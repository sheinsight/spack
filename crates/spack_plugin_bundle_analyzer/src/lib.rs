#![feature(duration_millis_float)]
mod asset;
mod chunk;
mod module;
mod module_type;
mod opts;
mod package;
mod package_version_resolver;
mod performance_timings;
mod report;
mod summary;

use std::io::Write;

use derive_more::Debug;
use flate2::Compression;
use flate2::write::GzEncoder;
use napi::tokio::time::Instant;
pub use opts::{BundleAnalyzerPluginOpts, CompilationHookFn};
use rayon::prelude::*;
use rspack_collections::Identifier;
use rspack_core::{ApplyContext, ChunkGraph, Compilation, CompilerAfterEmit, Plugin};
use rspack_hook::{plugin, plugin_hook};

pub use crate::{
  asset::Asset, chunk::Chunk, module::Module, module_type::ModuleType, package::Package,
  performance_timings::PerformanceTimings, report::Report, summary::Summary,
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
    ctx.compiler_hooks.after_emit.tap(after_emit::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilerAfterEmit for BundleAnalyzerPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let start_time = Instant::now();

  // 1. 收集 Assets（输出文件）
  let assets_start = Instant::now();
  let assets = collect_assets(compilation);
  let collect_assets_ms = assets_start.elapsed().as_millis_f64();

  // 2. 收集 Modules（源文件）
  let modules_start = Instant::now();
  let modules = collect_modules(compilation);
  let collect_modules_ms = modules_start.elapsed().as_millis_f64();

  // 3. 收集 Chunks（代码块）
  let chunks_start = Instant::now();
  let chunks = collect_chunks(compilation);
  // let collect_chunks_ms = chunks_start.elapsed().as_micros() as f64 / 1000.0;
  let collect_chunks_ms = chunks_start.elapsed().as_millis_f64();

  // 4. 分析 Packages（按包名聚合）
  let packages_start = Instant::now();
  let packages = analyze_packages(&modules);
  let analyze_packages_ms = packages_start.elapsed().as_millis_f64();

  // 计算总耗时
  let total_ms = start_time.elapsed().as_millis_f64();

  // Gzip 压缩耗时已经在 collect_assets 中并行计算，这里统计的是总耗时中用于压缩的部分
  // 实际压缩时间已包含在 collect_assets_ms 中
  let compress_gzip_ms = collect_assets_ms; // 压缩主要发生在 collect_assets 阶段

  // 计算总大小：累加所有 assets 的大小
  let total_size: u64 = assets.iter().map(|a| a.size as u64).sum();

  // 计算 gzip 压缩后总大小
  let total_gzip_size: u64 = assets
    .iter()
    .filter_map(|a| a.gzip_size.map(|s| s as u64))
    .sum();

  // 构建性能计时信息
  let timings = PerformanceTimings::new(
    collect_assets_ms,
    collect_modules_ms,
    collect_chunks_ms,
    analyze_packages_ms,
    compress_gzip_ms,
    total_ms,
  );

  let summary = Summary {
    total_size,
    total_gzip_size,
    total_assets: assets.len(),
    total_modules: modules.len(),
    total_chunks: chunks.len(),
    build_time: total_ms, // 保持向后兼容
    timings,
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

  // 调用回调函数
  if let Some(on_analyzed) = &self.options.on_analyzed {
    if let Err(e) = on_analyzed(report).await {
      tracing::error!("BundleAnalyzerPlugin callback failed: {:?}", e);
    }
  }

  Ok(())
}

fn collect_assets(compilation: &Compilation) -> Vec<Asset> {
  // 先收集所有 assets 的基本信息到 Vec,然后并行计算压缩大小
  let assets_data: Vec<_> = compilation
    .assets()
    .iter()
    .map(|(name, asset)| {
      let buffer = asset.source.as_ref().map(|s| s.buffer());
      let size = asset.source.as_ref().map(|s| s.size()).unwrap_or(0);
      (name.to_string(), size, buffer)
    })
    .collect();

  // 使用 rayon 并行计算每个 asset 的压缩大小
  assets_data
    .par_iter()
    .map(|(name, size, buffer_opt)| {
      let gzip_size = if let Some(buffer) = buffer_opt {
        // 并行计算 gzip 压缩大小
        calculate_gzip_size(buffer)
      } else {
        None
      };

      Asset {
        name: name.clone(),
        size: *size,
        gzip_size,
        chunks: get_asset_chunks(name, compilation),
        emitted: true,
      }
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
    .map(|(id, module)| {
      let name = module
        .readable_identifier(&compilation.options.context)
        .to_string();

      // 识别模块类型
      let module_type = ModuleType::from_path(&name);

      // 判断是否来自 node_modules
      let is_node_module = name.contains("node_modules/");

      Module {
        id: id.to_string(),
        name,
        size: get_module_size(module.as_ref()),
        chunks: get_module_chunks(&id, chunk_graph),
        module_type,
        is_node_module,
      }
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

  // 创建包信息解析器
  let mut resolver = package_version_resolver::PackageVersionResolver::new();

  // 1. 遍历所有模块,按包名分组
  for module in modules {
    // 从 package.json 解析包名和版本
    if let Some((package_name, version)) = resolver.resolve(&module.name) {
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

/// 计算 gzip 压缩后的大小
///
/// 参数:
/// - data: 原始数据字节
///
/// 返回: 压缩后的字节数,如果压缩失败返回 None
fn calculate_gzip_size(data: &[u8]) -> Option<usize> {
  let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

  // 写入数据
  if encoder.write_all(data).is_err() {
    return None;
  }

  // 完成压缩
  match encoder.finish() {
    Ok(compressed) => Some(compressed.len()),
    Err(e) => {
      tracing::error!("{}", e);
      None
    }
  }
}
