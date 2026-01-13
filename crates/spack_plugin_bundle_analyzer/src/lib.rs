mod asset;
mod chunk;
mod module;
mod opts;
mod package;
// mod report;
mod resp;
// mod summary;
mod types;

use derive_more::Debug;
use napi::tokio::time::Instant;
pub use opts::{BundleAnalyzerPluginOpts, CompilationHookFn};
pub use resp::*;
use rspack_collections::Identifier;
use rspack_core::{ApplyContext, ChunkGraph, Compilation, CompilerAfterEmit, Plugin};
use rspack_hook::{plugin, plugin_hook};
pub use types::*;

use crate::{asset::Asset, chunk::Chunk, module::Module, package::Package};

#[plugin]
#[derive(Debug)]
pub struct BundleAnalyzerPlugin {
  #[allow(unused)]
  options: BundleAnalyzerPluginOpts,
}

impl BundleAnalyzerPlugin {
  pub fn new(options: BundleAnalyzerPluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for BundleAnalyzerPlugin {
  fn name(&self) -> &'static str {
    "spack.BundleAnalyzerPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    println!("coming bundle analyzer");
    ctx.compiler_hooks.after_emit.tap(after_emit::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilerAfterEmit for BundleAnalyzerPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let start_time = Instant::now();

  // 1. 收集 Assets（输出文件）
  let assets = collect_assets(compilation);

  // 2. 收集 Modules（源文件）
  let modules = collect_modules(compilation);

  // 3. 收集 Chunks（代码块）
  let chunks = collect_chunks(compilation);

  // 4. 分析 Packages（按包名聚合）
  let packages = analyze_packages(&modules);

  let millis = start_time.elapsed().as_millis();

  println!("assets--> {:#?}", assets);

  println!("modules--> {:#?}", modules);

  println!("chunks---> {:#?}", chunks);

  println!("packages--> {:#?}", packages);

  println!("millis {}", millis);

  Ok(())
}

fn collect_assets(compilation: &Compilation) -> Vec<Asset> {
  compilation
    .assets()
    .iter()
    .map(|(name, asset)| {
      let size = if let Some(source) = &asset.source {
        source.size()
      } else {
        0
      };

      return Asset {
        name: name.to_string(),
        size: size,
        chunks: get_asset_chunks(name, compilation),
        emitted: true,
      };
    })
    .collect()
}

fn get_asset_chunks(asset_name: &str, compilation: &Compilation) -> Vec<String> {
  compilation
    .chunk_by_ukey
    .values()
    .filter(|chunk| chunk.files().contains(asset_name))
    .map(|chunk| {
      let id = if let Some(id) = chunk.id() {
        id.to_string()
      } else {
        "".to_string()
      };
      return id;
    })
    .collect()
}

fn collect_modules(compilation: &Compilation) -> Vec<Module> {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;

  module_graph
    .modules()
    .into_iter()
    .map(|(id, module)| Module {
      id: id.to_string(),
      name: module
        .readable_identifier(&compilation.options.context)
        .to_string(),
      size: get_module_size(module.as_ref()),
      chunks: get_module_chunks(&id, chunk_graph),
    })
    .collect()
}

fn get_module_size(module: &dyn rspack_core::Module) -> u64 {
  // 使用 Module trait 的 size 方法获取模块大小
  // source_type 参数为 None 表示获取所有类型的总大小
  // compilation 参数为 None 因为我们不需要编译上下文
  module.size(None, None) as u64
}

fn get_module_chunks(module_id: &Identifier, chunk_graph: &ChunkGraph) -> Vec<String> {
  chunk_graph
    .get_module_chunks(*module_id)
    .iter()
    .map(|chunk_ukey| chunk_ukey.as_u32().to_string())
    .collect()
}

/// 计算 chunk 的总大小
/// 通过累加该 chunk 包含的所有模块的大小得出
fn calculate_chunk_size(module_ids: &[String], module_graph: &rspack_core::ModuleGraph) -> u64 {
  module_ids
    .iter()
    .filter_map(|id_str| {
      // 将字符串 ID 转换为 ModuleIdentifier
      // 从 module_graph 中找到对应的模块并获取其大小
      module_graph
        .modules()
        .into_iter()
        .find(|(module_id, _)| module_id.to_string() == *id_str)
        .map(|(_, module)| get_module_size(module.as_ref()))
    })
    .sum()
}

fn collect_chunks(compilation: &Compilation) -> Vec<Chunk> {
  let chunk_graph = &compilation.chunk_graph;
  let module_graph = compilation.get_module_graph();

  compilation
    .chunk_by_ukey
    .iter()
    .map(|(ukey, chunk)| {
      let modules: Vec<String> = chunk_graph
        .get_chunk_modules(ukey, &module_graph)
        .iter()
        .map(|m| m.identifier().to_string())
        .collect();

      Chunk {
        id: ukey.as_u32().to_string(),
        names: chunk
          .name()
          .map(|n| vec![n.to_string()])
          .unwrap_or_default(),
        size: calculate_chunk_size(&modules, &module_graph),
        modules,
        entry: chunk.has_entry_module(chunk_graph),
        initial: chunk.can_be_initial(&compilation.chunk_group_by_ukey),
      }
    })
    .collect()
}

/// 分析包依赖,按包名聚合
fn analyze_packages(modules: &[Module]) -> Vec<Package> {
  use std::collections::HashMap;

  let mut package_map: HashMap<String, Vec<&Module>> = HashMap::new();

  // 1. 遍历所有模块,按包名分组
  for module in modules {
    if let Some(package_name) = parse_package_name(&module.name) {
      package_map.entry(package_name).or_default().push(module);
    }
  }

  // 2. 为每个包生成统计信息
  let mut packages: Vec<Package> = package_map
    .into_iter()
    .map(|(name, mods)| {
      let size: u64 = mods.iter().map(|m| m.size).sum();
      let modules: Vec<String> = mods.iter().map(|m| m.id.clone()).collect();

      Package {
        name,
        version: "unknown".to_string(), // 暂时固定为 unknown
        size,
        module_count: mods.len(),
        modules,
      }
    })
    .collect();

  // 3. 按大小降序排序
  packages.sort_by_key(|p| std::cmp::Reverse(p.size));

  packages
}

/// 从模块路径中解析包名
/// 例如:
///   "node_modules/react/index.js" -> Some("react")
///   "node_modules/@babel/core/lib.js" -> Some("@babel/core")
///   "./src/index.js" -> None (不是 node_modules)
fn parse_package_name(module_path: &str) -> Option<String> {
  // 只处理 node_modules 中的模块
  if !module_path.contains("node_modules/") {
    return None;
  }

  // 找到 node_modules/ 后面的部分
  let parts: Vec<&str> = module_path.split("node_modules/").collect();
  if parts.len() < 2 {
    return None;
  }

  let after_nm = parts[1];
  let segments: Vec<&str> = after_nm.split('/').collect();

  // 处理 scoped package (如 @babel/core)
  if segments[0].starts_with('@') {
    if segments.len() < 2 {
      return None;
    }
    Some(format!("{}/{}", segments[0], segments[1]))
  } else {
    // 普通 package (如 react)
    Some(segments[0].to_string())
  }
}
