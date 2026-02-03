#![feature(duration_millis_float)]
mod asset;
mod chunk;
mod context;
mod module;
mod opts;
mod package;
mod reporting;

use std::env::current_dir;

use derive_more::Debug;
use napi::tokio::{fs, time::Instant};
pub use opts::{BundleAnalyzerPluginOpts, CompilationHookFn};
use rspack_core::{ApplyContext, Compilation, CompilerAfterEmit, Plugin};
use rspack_hook::{plugin, plugin_hook};

use crate::{asset::Assets, context::ModuleChunkContext, module::Modules, package::Packages};
pub use crate::{
  asset::{Asset, AssetType},
  chunk::Chunk,
  module::{ConcatenatedModuleInfo, Module, ModuleDependency, ModuleKind, ModuleReason, ModuleType},
  package::Package,
  reporting::{PerformanceTimings, Report, Summary},
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

  // 1. 预构建 module ↔ chunk 映射关系（性能优化）
  let context_start = Instant::now();
  let module_chunk_context = ModuleChunkContext::from(&*compilation);
  let _build_context_ms = context_start.elapsed().as_millis_f64();

  // 2. 收集 Assets（输出文件）
  let assets_start = Instant::now();
  let enable_gzip = self.options.gzip_assets.unwrap_or(false);
  let enable_brotli = self.options.brotli_assets.unwrap_or(false);
  let assets = Assets::from_with_compression(&mut *compilation, enable_gzip, enable_brotli);
  let collect_assets_ms = assets_start.elapsed().as_millis_f64();

  // 3. 收集 Modules（源文件，使用预构建的映射）
  let modules_start = Instant::now();
  let mut modules = Modules::from_with_context(&mut *compilation, &module_chunk_context);
  let collect_modules_ms = modules_start.elapsed().as_millis_f64();

  // 4. 收集 Chunks（代码块，使用预构建的映射）
  let chunks_start = Instant::now();
  let chunks = chunk::Chunks::from_with_context(&mut *compilation, &module_chunk_context);
  let collect_chunks_ms = chunks_start.elapsed().as_millis_f64();

  // 5. 创建包版本解析器（在多个分析阶段复用，避免重复创建和缓存失效）
  let mut resolver = package::PackageVersionResolver::new();

  // 6. 分析 Packages（按包名聚合，复用 resolver）
  let packages_start = Instant::now();
  let packages = Packages::from_with_resolver(&modules, &mut resolver);
  let analyze_packages_ms = packages_start.elapsed().as_millis_f64();

  // 7. 关联 Module 和 Package（填充 package_json_path）
  modules.associate_packages(&packages);

  // 计算总耗时
  let total_ms = start_time.elapsed().as_millis_f64();

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

  let dir = current_dir().unwrap();

  // 1. 写出 JSON 数据文件（用于调试）
  let json_file = dir.join("bundle-analyzer.json");
  let json_data = serde_json::to_string_pretty(&report)
    .map_err(|e| rspack_error::error!("Failed to serialize report: {}", e))?;
  fs::write(&json_file, &json_data).await?;

  // 2. 生成 HTML 报告
  let html_file = dir.join("bundle-analyzer.html");

  // 读取 HTML 模板（编译时嵌入）
  let template = include_str!("index.html");

  // 替换数据注入点
  let html_content = template.replace(
    "window.__bundle_viewer_data__ = null;",
    &format!("window.__bundle_viewer_data__ = {};", json_data)
  );

  fs::write(&html_file, html_content).await?;

  tracing::info!(
    "Bundle analysis complete:\n  - JSON: {}\n  - HTML: {}",
    json_file.display(),
    html_file.display()
  );

  // 调用回调函数
  if let Some(on_analyzed) = &self.options.on_analyzed {
    if let Err(e) = on_analyzed(report).await {
      tracing::error!("BundleAnalyzerPlugin callback failed: {:?}", e);
    }
  }

  Ok(())
}
