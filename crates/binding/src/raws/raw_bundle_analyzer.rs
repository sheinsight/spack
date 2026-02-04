use std::collections::HashMap;

use derive_more::Debug;
use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use spack_macros::ThreadsafeCallback;
use spack_plugin_bundle_analyzer::{
  Asset, BundleAnalyzerPlugin, BundleAnalyzerPluginOpts, Chunk, ConcatenatedModuleInfo, Module,
  Package, PerformanceTimings, Report, Summary,
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
  /// 是否计算 brotli 压缩后的大小（默认：false）
  pub brotli_assets: Option<bool>,
}

// JavaScript 可用的数据结构

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsAsset {
  pub name: String,
  pub size: u32,
  pub gzip_size: Option<u32>,
  pub brotli_size: Option<u32>,
  pub chunks: Vec<String>,
  pub emitted: bool,
  // pub asset_type: String,
}

impl From<Asset> for JsAsset {
  fn from(value: Asset) -> Self {
    Self {
      name: value.name,
      size: value.size as u32,
      gzip_size: value.gzip_size.map(|s| s as u32),
      brotli_size: value.brotli_size.map(|s| s as u32),
      chunks: value.chunks,
      emitted: value.emitted,
      // asset_type: value.asset_type.as_str().to_string(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsModule {
  pub id: u32,
  // pub name: String,
  pub size: u32,
  pub chunks: Vec<String>,
  pub module_kind: String,
  // pub module_type: String,
  pub is_node_module: bool,
  pub name_for_condition: String,
  pub concatenated_modules: Option<Vec<JsConcatenatedModuleInfo>>,
  /// 关联的 Package 的 package.json 路径（唯一标识）
  /// 仅三方包模块有值，用于精确匹配对应的 Package
  pub package_json_path: Option<String>,
  // /// 用户请求路径（如 require('lodash') 中的 'lodash'）
  // pub user_request: Option<String>,
  /// 原始请求路径（如 loader 链中的完整请求）
  pub raw_request: Option<String>,
  // /// 当前模块的出站依赖列表（当前模块依赖哪些模块的 ID）
  // pub dependencies: Option<Vec<String>>,
  /// 当前模块的入站依赖列表（哪些模块依赖当前模块的数字 ID）
  pub reasons: Option<Vec<u32>>,
}

impl From<Module> for JsModule {
  fn from(value: Module) -> Self {
    Self {
      id: value.id,
      // name: value.name,
      size: value.size as u32,
      chunks: value.chunks,
      module_kind: value.module_kind.as_str().to_string(),
      // module_type: value.module_type.as_str().to_string(),
      is_node_module: value.is_node_module,
      name_for_condition: value.name_for_condition,
      concatenated_modules: value
        .concatenated_modules
        .map(|modules| modules.into_iter().map(|m| m.into()).collect()),
      package_json_path: value.package_json_path,
      // user_request: value.user_request,
      raw_request: value.raw_request,
      // dependencies: value.dependencies,
      reasons: value.reasons,
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
  // 包含的模块数字 ID 列表
  pub modules: Vec<u32>,
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
  // 父 chunk ID 列表（哪些 chunk 引用了当前 chunk）
  pub parents: Vec<String>,
  // 子 chunk ID 列表（当前 chunk 引用了哪些 chunk）
  pub children: Vec<String>,
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
      parents: value.parents,
      children: value.children,
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
  pub modules: Vec<u32>,
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
pub struct JsPerformanceTimings {
  pub collect_assets_ms: f64,
  pub collect_modules_ms: f64,
  pub collect_chunks_ms: f64,
  pub analyze_packages_ms: f64,
  pub total_ms: f64,
}

impl From<PerformanceTimings> for JsPerformanceTimings {
  fn from(value: PerformanceTimings) -> Self {
    Self {
      collect_assets_ms: value.collect_assets_ms,
      collect_modules_ms: value.collect_modules_ms,
      collect_chunks_ms: value.collect_chunks_ms,
      analyze_packages_ms: value.analyze_packages_ms,
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
pub struct JsConcatenatedModuleInfo {
  pub id: u32,
  // pub name: String,
  pub size: u32,
  // /// 模块文件类型
  // pub module_type: String,
  /// 是否来自 node_modules
  pub is_node_module: bool,
  /// 模块条件名称
  pub name_for_condition: String,
  /// 关联的 Package 的 package.json 路径
  pub package_json_path: Option<String>,
}

impl From<ConcatenatedModuleInfo> for JsConcatenatedModuleInfo {
  fn from(value: ConcatenatedModuleInfo) -> Self {
    Self {
      id: value.id,
      // name: value.name,
      size: value.size as u32,
      // module_type: value.module_type.as_str().to_string(),
      is_node_module: value.is_node_module,
      name_for_condition: value.name_for_condition,
      package_json_path: value.package_json_path,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsBundleAnalyzerPluginResp {
  pub timestamp: u32,
  pub summary: JsSummary,
  pub module_id_map: HashMap<String, String>,
  pub assets: Vec<JsAsset>,
  pub modules: Vec<JsModule>,
  pub chunks: Vec<JsChunk>,
  pub packages: Vec<JsPackage>,
}

impl From<Report> for JsBundleAnalyzerPluginResp {
  fn from(value: Report) -> Self {
    Self {
      timestamp: value.timestamp as u32,
      summary: value.summary.into(),
      module_id_map: value.module_id_map,
      assets: value.assets.into_iter().map(|a| a.into()).collect(),
      modules: value.modules.into_iter().map(|m| m.into()).collect(),
      chunks: value.chunks.into_iter().map(|c| c.into()).collect(),
      packages: value.packages.into_iter().map(|p| p.into()).collect(),
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
