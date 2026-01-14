mod asset;
mod chunk;
mod module;
mod opts;
mod package;
mod report;
// mod resp;
mod summary;
// mod types;

use derive_more::Debug;
use napi::tokio::time::Instant;
pub use opts::{BundleAnalyzerPluginOpts, CompilationHookFn};
// pub use resp::*;
use rspack_collections::Identifier;
use rspack_core::{ApplyContext, ChunkGraph, Compilation, CompilerAfterEmit, Plugin};
use rspack_hook::{plugin, plugin_hook};

// pub use types::*;
pub use crate::{
  asset::Asset, chunk::Chunk, module::Module, package::Package, report::Report, summary::Summary,
};

#[plugin]
#[derive(Debug)]
pub struct BundleAnalyzerPlugin {
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

  // println!("assets--> {:#?}", assets);

  // println!("modules--> {:#?}", modules);

  // println!("chunks---> {:#?}", chunks);

  // 计算总大小：累加所有 assets 的大小
  let total_size: u64 = assets.iter().map(|a| a.size as u64).sum();

  let summary = Summary {
    total_size,
    total_assets: assets.len(),
    total_modules: modules.len(),
    total_chunks: chunks.len(),
    build_time: millis as f64,
  };

  // 获取当前 Unix 时间戳（毫秒）
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis() as u64;

  let report = Report {
    timestamp,
    summary,
    assets,
    modules,
    chunks,
    packages,
  };

  // println!("report--> {:#?}", report);

  // 调用回调函数
  if let Some(on_analyzed) = &self.options.on_analyzed {
    if let Err(e) = on_analyzed(report).await {
      tracing::error!("BundleAnalyzerPlugin callback failed: {:?}", e);
    }
  }

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

  // key 是包名, value 是 (版本号, 模块列表)
  let mut package_map: HashMap<String, (String, Vec<&Module>)> = HashMap::new();

  // 1. 遍历所有模块,按包名分组
  for module in modules {
    if let Some((package_name, version)) = parse_package_info(&module.name) {
      package_map
        .entry(package_name)
        .or_insert_with(|| (version.clone(), Vec::new()))
        .1
        .push(module);
    }
  }

  // 2. 为每个包生成统计信息
  let mut packages: Vec<Package> = package_map
    .into_iter()
    .map(|(name, (version, mods))| {
      let size: u64 = mods.iter().map(|m| m.size).sum();
      let modules: Vec<String> = mods.iter().map(|m| m.id.clone()).collect();

      Package {
        name,
        version,
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

/// 从模块路径中解析包名和版本号
/// 支持 npm/yarn 和 pnpm 两种路径格式
///
/// 返回: Some((包名, 版本号))
///
/// 例如:
///   npm/yarn 格式:
///     "node_modules/react/index.js" -> Some(("react", "unknown"))
///     "node_modules/@babel/core/lib.js" -> Some(("@babel/core", "unknown"))
///
///   pnpm 格式:
///     "node_modules/.pnpm/react@18.2.0/node_modules/react/index.js"
///       -> Some(("react", "18.2.0"))
///     "node_modules/.pnpm/@babel+core@7.22.0/node_modules/@babel/core/lib.js"
///       -> Some(("@babel/core", "7.22.0"))
///
///   非 node_modules:
///     "./src/index.js" -> None
fn parse_package_info(module_path: &str) -> Option<(String, String)> {
  // 只处理 node_modules 中的模块
  if !module_path.contains("node_modules/") {
    return None;
  }

  // 优先检查是否是 pnpm 格式 (包含 .pnpm/)
  if module_path.contains("node_modules/.pnpm/") {
    return parse_pnpm_package_info(module_path);
  }

  // 处理标准 npm/yarn 格式
  parse_npm_package_info(module_path)
}

/// 解析 pnpm 格式的包路径
/// 例如:
///   node_modules/.pnpm/react@18.2.0/node_modules/react/index.js
///   node_modules/.pnpm/@babel+core@7.22.0/node_modules/@babel/core/lib.js
///   node_modules/.pnpm/@visactor+react-vchart@2.0.9_react-dom@19.2.0_react@19.2.0/...
fn parse_pnpm_package_info(module_path: &str) -> Option<(String, String)> {
  // 找到 .pnpm/ 后面的部分
  let parts: Vec<&str> = module_path.split("node_modules/.pnpm/").collect();
  if parts.len() < 2 {
    return None;
  }

  let after_pnpm = parts[1];
  let segments: Vec<&str> = after_pnpm.split('/').collect();
  if segments.is_empty() {
    return None;
  }

  // 第一个 segment 格式:
  // - 普通包: "包名@版本" 或 "包名@版本_peer依赖信息"
  // - scoped: "@scope+包名@版本" 或 "@scope+包名@版本_peer依赖信息"
  let pkg_with_version = segments[0];

  // 先去掉 peer dependencies 后缀 (如果有的话)
  // 例如: react@18.2.0_some_peer -> react@18.2.0
  let pkg_without_peers = if let Some(underscore_pos) = pkg_with_version.find('_') {
    &pkg_with_version[..underscore_pos]
  } else {
    pkg_with_version
  };

  // 处理 scoped package: @babel+core@7.22.0
  if pkg_without_peers.starts_with('@') {
    // 去掉开头的 @
    let without_at = &pkg_without_peers[1..];

    // 找到最后一个 @ (版本号前的)
    if let Some(last_at_pos) = without_at.rfind('@') {
      let name_part = &without_at[..last_at_pos]; // 例如: babel+core
      let version = &without_at[last_at_pos + 1..]; // 例如: 7.22.0

      // 将 + 替换回 /
      let package_name = format!("@{}", name_part.replace('+', "/"));

      return Some((package_name, version.to_string()));
    }
  } else {
    // 处理普通 package: react@18.2.0
    if let Some(at_pos) = pkg_without_peers.rfind('@') {
      let package_name = pkg_without_peers[..at_pos].to_string();
      let version = pkg_without_peers[at_pos + 1..].to_string();
      return Some((package_name, version));
    }
  }

  None
}

/// 解析标准 npm/yarn 格式的包路径
/// 例如: node_modules/react/index.js
///       node_modules/@babel/core/lib.js
fn parse_npm_package_info(module_path: &str) -> Option<(String, String)> {
  // 找到最后一个 node_modules/ (可能有嵌套依赖)
  let parts: Vec<&str> = module_path.split("node_modules/").collect();
  if parts.len() < 2 {
    return None;
  }

  let after_nm = parts[parts.len() - 1];
  let segments: Vec<&str> = after_nm.split('/').collect();

  // 处理 scoped package (如 @babel/core)
  if segments[0].starts_with('@') {
    if segments.len() < 2 {
      return None;
    }
    let package_name = format!("{}/{}", segments[0], segments[1]);
    Some((package_name, "unknown".to_string()))
  } else {
    // 普通 package (如 react)
    let package_name = segments[0].to_string();
    Some((package_name, "unknown".to_string()))
  }
}
