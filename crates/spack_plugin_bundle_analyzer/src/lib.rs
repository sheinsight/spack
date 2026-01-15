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

use derive_more::Debug;
use napi::tokio::time::Instant;
pub use opts::{BundleAnalyzerPluginOpts, CompilationHookFn};
use rspack_core::{ApplyContext, Compilation, CompilerAfterEmit, Plugin};
use rspack_hook::{plugin, plugin_hook};

pub use crate::{
  asset::Asset, chunk::Chunk, module::Module, module_type::ModuleType, package::Package,
  performance_timings::PerformanceTimings, report::Report, summary::Summary,
};
use crate::{asset::Assets, module::Modules, package::Packages};

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
  let assets = Assets::from(&mut *compilation);

  let collect_assets_ms = assets_start.elapsed().as_millis_f64();

  // 2. 收集 Modules（源文件）
  let modules_start = Instant::now();
  // let modules = collect_modules(compilation);
  let modules = Modules::from(&mut *compilation);
  let collect_modules_ms = modules_start.elapsed().as_millis_f64();

  // 3. 收集 Chunks（代码块）
  let chunks_start = Instant::now();
  let chunks = chunk::Chunks::from(&mut *compilation);
  let collect_chunks_ms = chunks_start.elapsed().as_millis_f64();

  // 4. 分析 Packages（按包名聚合）
  let packages_start = Instant::now();
  let packages = Packages::from(&modules);
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
    assets: assets.into(),
    modules: modules.into(),
    chunks: chunks.into(),
    packages: packages.into(),
  };

  // 调用回调函数
  if let Some(on_analyzed) = &self.options.on_analyzed {
    if let Err(e) = on_analyzed(report).await {
      tracing::error!("BundleAnalyzerPlugin callback failed: {:?}", e);
    }
  }

  Ok(())
}
