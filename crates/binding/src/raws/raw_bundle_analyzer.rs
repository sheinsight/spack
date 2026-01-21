use derive_more::Debug;
use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use spack_macros::ThreadsafeCallback;
use spack_plugin_bundle_analyzer::{
  Asset, BundleAnalyzerPlugin, BundleAnalyzerPluginOpts, Chunk, ChunkModuleBreakdown,
  ChunkOverlapAnalysis, ChunkPairOverlap, ConcatenatedModuleInfo, DuplicatePackage, Module,
  ModuleSizeInfo, NodeModulesBreakdown, OverlappedModule, Package, PackageBreakdown,
  PackageVersion, PerformanceTimings, Report, SourceBreakdown, Summary,
};

#[derive(Debug, ThreadsafeCallback)]
#[napi(object, object_to_js = false)]
pub struct RawBundleAnalyzerPluginOpts {
  #[napi(ts_type = "(response: JsBundleAnalyzerPluginResp) => void|Promise<void>")]
  #[debug(skip)]
  #[threadsafe_callback]
  pub on_analyzed: Option<ThreadsafeFunction<JsBundleAnalyzerPluginResp, ()>>,
  /// 是否计算 gzip 压缩后的大小（默认：false）
  pub gzip_assets: Option<bool>,
}

// JavaScript 可用的数据结构

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsAsset {
  pub name: String,
  pub size: u32,
  pub gzip_size: Option<u32>,
  pub chunks: Vec<String>,
  pub emitted: bool,
}

impl From<Asset> for JsAsset {
  fn from(value: Asset) -> Self {
    Self {
      name: value.name,
      size: value.size as u32,
      gzip_size: value.gzip_size.map(|s| s as u32),
      chunks: value.chunks,
      emitted: value.emitted,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsModule {
  pub id: String,
  pub name: String,
  pub size: u32,
  pub chunks: Vec<String>,
  pub module_kind: String,
  pub module_type: String,
  pub is_node_module: bool,
  pub name_for_condition: String,
}

impl From<Module> for JsModule {
  fn from(value: Module) -> Self {
    Self {
      id: value.id,
      name: value.name,
      size: value.size as u32,
      chunks: value.chunks,
      module_kind: value.module_kind.as_str().to_string(),
      module_type: value.module_type.as_str().to_string(),
      is_node_module: value.is_node_module,
      name_for_condition: value.name_for_condition,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunk {
  // chunk ID
  pub id: String,
  // chunk 名称
  pub names: Vec<String>,
  // chunk 大小
  pub size: u32,
  // 包含的模块 ID 列表
  pub modules: Vec<String>,
  // 是否入口 chunk
  pub entry: bool,
  // 是否初始 chunk
  pub initial: bool,
  // 是否包含异步 chunk
  pub async_chunks: bool,
  // 是否包含运行时代码
  pub runtime: bool,
  // chunk 创建的原因(如 entry、import()、splitChunks 等)
  pub reason: String,
  // chunk 生成的输出文件列表
  pub files: Vec<String>,
}

impl From<Chunk> for JsChunk {
  fn from(value: Chunk) -> Self {
    Self {
      id: value.id,
      names: value.names,
      size: value.size as u32,
      modules: value.modules,
      entry: value.entry,
      initial: value.initial,
      reason: value.reason,
      files: value.files,
      async_chunks: value.async_chunks,
      runtime: value.runtime,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsPackage {
  pub name: String,
  pub version: String,
  pub size: u32,
  pub module_count: u32,
  pub modules: Vec<String>,
  pub package_json_path: String,
}

impl From<Package> for JsPackage {
  fn from(value: Package) -> Self {
    Self {
      name: value.name,
      version: value.version,
      size: value.size as u32,
      module_count: value.module_count as u32,
      modules: value.modules,
      package_json_path: value.package_json_path,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsPackageVersion {
  pub version: String,
  pub size: u32,
  pub module_count: u32,
  pub package_json_path: String,
}

impl From<PackageVersion> for JsPackageVersion {
  fn from(value: PackageVersion) -> Self {
    Self {
      version: value.version,
      size: value.size as u32,
      module_count: value.module_count as u32,
      package_json_path: value.package_json_path,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsDuplicatePackage {
  pub name: String,
  pub versions: Vec<JsPackageVersion>,
  pub total_size: u32,
  pub wasted_size: u32,
}

impl From<DuplicatePackage> for JsDuplicatePackage {
  fn from(value: DuplicatePackage) -> Self {
    Self {
      name: value.name,
      versions: value.versions.into_iter().map(|v| v.into()).collect(),
      total_size: value.total_size as u32,
      wasted_size: value.wasted_size as u32,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsPerformanceTimings {
  pub collect_assets_ms: f64,
  pub collect_modules_ms: f64,
  pub collect_chunks_ms: f64,
  pub analyze_packages_ms: f64,
  pub compress_gzip_ms: f64,
  pub analyze_overlap_ms: f64,
  pub total_ms: f64,
}

impl From<PerformanceTimings> for JsPerformanceTimings {
  fn from(value: PerformanceTimings) -> Self {
    Self {
      collect_assets_ms: value.collect_assets_ms,
      collect_modules_ms: value.collect_modules_ms,
      collect_chunks_ms: value.collect_chunks_ms,
      analyze_packages_ms: value.analyze_packages_ms,
      compress_gzip_ms: value.compress_gzip_ms,
      analyze_overlap_ms: value.analyze_overlap_ms,
      total_ms: value.total_ms,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsSummary {
  pub total_size: u32,
  pub total_gzip_size: u32,
  pub total_assets: u32,
  pub total_modules: u32,
  pub total_chunks: u32,
  pub build_time: f64,
  pub timings: JsPerformanceTimings,
}

impl From<Summary> for JsSummary {
  fn from(value: Summary) -> Self {
    Self {
      total_size: value.total_size as u32,
      total_gzip_size: value.total_gzip_size as u32,
      total_assets: value.total_assets as u32,
      total_modules: value.total_modules as u32,
      total_chunks: value.total_chunks as u32,
      build_time: value.build_time,
      timings: value.timings.into(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsOverlappedModule {
  pub module_id: String,
  pub module_name: String,
  pub module_size: u32,
  pub chunks: Vec<String>,
  pub duplication_count: u32,
  pub wasted_size: u32,
  pub package_name: Option<String>,
}

impl From<OverlappedModule> for JsOverlappedModule {
  fn from(value: OverlappedModule) -> Self {
    Self {
      module_id: value.module_id,
      module_name: value.module_name,
      module_size: value.module_size as u32,
      chunks: value.chunks,
      duplication_count: value.duplication_count as u32,
      wasted_size: value.wasted_size as u32,
      package_name: value.package_name,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunkPairOverlap {
  pub chunk_a: String,
  pub chunk_b: String,
  pub shared_modules: Vec<String>,
  pub shared_size: u32,
  pub overlap_ratio_a: f64,
  pub overlap_ratio_b: f64,
}

impl From<ChunkPairOverlap> for JsChunkPairOverlap {
  fn from(value: ChunkPairOverlap) -> Self {
    Self {
      chunk_a: value.chunk_a,
      chunk_b: value.chunk_b,
      shared_modules: value.shared_modules,
      shared_size: value.shared_size as u32,
      overlap_ratio_a: value.overlap_ratio_a,
      overlap_ratio_b: value.overlap_ratio_b,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunkOverlapAnalysis {
  pub overlapped_modules: Vec<JsOverlappedModule>,
  pub chunk_pair_overlaps: Vec<JsChunkPairOverlap>,
  pub total_wasted_size: u32,
  pub recommendations: Vec<String>,
}

impl From<ChunkOverlapAnalysis> for JsChunkOverlapAnalysis {
  fn from(value: ChunkOverlapAnalysis) -> Self {
    Self {
      overlapped_modules: value
        .overlapped_modules
        .into_iter()
        .map(|m| m.into())
        .collect(),
      chunk_pair_overlaps: value
        .chunk_pair_overlaps
        .into_iter()
        .map(|p| p.into())
        .collect(),
      total_wasted_size: value.total_wasted_size as u32,
      recommendations: value.recommendations,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsConcatenatedModuleInfo {
  pub id: String,
  pub name: String,
  pub size: u32,
}

impl From<ConcatenatedModuleInfo> for JsConcatenatedModuleInfo {
  fn from(value: ConcatenatedModuleInfo) -> Self {
    Self {
      id: value.id,
      name: value.name,
      size: value.size as u32,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsModuleSizeInfo {
  pub module_id: String,
  pub module_name: String,
  pub size: u32,
  pub module_type: String,
  pub concatenated_modules: Option<Vec<JsConcatenatedModuleInfo>>,
}

impl From<ModuleSizeInfo> for JsModuleSizeInfo {
  fn from(value: ModuleSizeInfo) -> Self {
    Self {
      module_id: value.module_id,
      module_name: value.module_name,
      size: value.size as u32,
      module_type: value.module_type,
      concatenated_modules: value
        .concatenated_modules
        .map(|modules| modules.into_iter().map(|m| m.into()).collect()),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsSourceBreakdown {
  pub total_size: u32,
  pub module_count: u32,
  pub modules: Vec<JsModuleSizeInfo>,
}

impl From<SourceBreakdown> for JsSourceBreakdown {
  fn from(value: SourceBreakdown) -> Self {
    Self {
      total_size: value.total_size as u32,
      module_count: value.module_count as u32,
      modules: value.modules.into_iter().map(|m| m.into()).collect(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsPackageBreakdown {
  pub package_name: String,
  pub total_size: u32,
  pub module_count: u32,
  pub modules: Vec<JsModuleSizeInfo>,
}

impl From<PackageBreakdown> for JsPackageBreakdown {
  fn from(value: PackageBreakdown) -> Self {
    Self {
      package_name: value.package_name,
      total_size: value.total_size as u32,
      module_count: value.module_count as u32,
      modules: value.modules.into_iter().map(|m| m.into()).collect(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsNodeModulesBreakdown {
  pub total_size: u32,
  pub package_count: u32,
  pub packages: Vec<JsPackageBreakdown>,
}

impl From<NodeModulesBreakdown> for JsNodeModulesBreakdown {
  fn from(value: NodeModulesBreakdown) -> Self {
    Self {
      total_size: value.total_size as u32,
      package_count: value.package_count as u32,
      packages: value.packages.into_iter().map(|p| p.into()).collect(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunkModuleBreakdown {
  pub chunk_id: String,
  pub chunk_size: u32,
  pub source: JsSourceBreakdown,
  pub node_modules: JsNodeModulesBreakdown,
}

impl From<ChunkModuleBreakdown> for JsChunkModuleBreakdown {
  fn from(value: ChunkModuleBreakdown) -> Self {
    Self {
      chunk_id: value.chunk_id,
      chunk_size: value.chunk_size as u32,
      source: value.source.into(),
      node_modules: value.node_modules.into(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsBundleAnalyzerPluginResp {
  pub timestamp: u32,
  pub summary: JsSummary,
  pub assets: Vec<JsAsset>,
  pub modules: Vec<JsModule>,
  pub chunks: Vec<JsChunk>,
  pub packages: Vec<JsPackage>,
  pub duplicate_packages: Vec<JsDuplicatePackage>,
  pub chunk_overlap: JsChunkOverlapAnalysis,
  pub chunk_module_breakdowns: Vec<JsChunkModuleBreakdown>,
}

impl From<Report> for JsBundleAnalyzerPluginResp {
  fn from(value: Report) -> Self {
    Self {
      timestamp: value.timestamp as u32,
      summary: value.summary.into(),
      assets: value.assets.into_iter().map(|a| a.into()).collect(),
      modules: value.modules.into_iter().map(|m| m.into()).collect(),
      chunks: value.chunks.into_iter().map(|c| c.into()).collect(),
      packages: value.packages.into_iter().map(|p| p.into()).collect(),
      duplicate_packages: value.duplicate_packages.into_iter().map(|d| d.into()).collect(),
      chunk_overlap: value.chunk_overlap.into(),
      chunk_module_breakdowns: value
        .chunk_module_breakdowns
        .into_iter()
        .map(|b| b.into())
        .collect(),
    }
  }
}

// impl Into<BundleAnalyzerPluginOpts> for RawBundleAnalyzerPluginOpts {
//   fn into(self) -> BundleAnalyzerPluginOpts {
//     BundleAnalyzerPluginOpts {
//       on_analyzed: todo!(),
//     }
//   }
// }

#[allow(unused)]
pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawBundleAnalyzerPluginOpts::from_unknown(options)?;
  Ok(Box::new(BundleAnalyzerPlugin::new(options.into())) as BoxPlugin)
}
