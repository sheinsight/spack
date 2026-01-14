use derive_more::Debug;
use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use spack_macros::ThreadsafeCallback;
use spack_plugin_bundle_analyzer::{
  Asset, BundleAnalyzerPlugin, BundleAnalyzerPluginOpts, Chunk, Module, Package, Report, Summary,
};

#[derive(Debug, ThreadsafeCallback)]
#[napi(object, object_to_js = false)]
pub struct RawBundleAnalyzerPluginOpts {
  #[napi(ts_type = "(response: JsBundleAnalyzerPluginResp) => void|Promise<void>")]
  #[debug(skip)]
  #[threadsafe_callback]
  pub on_analyzed: Option<ThreadsafeFunction<JsBundleAnalyzerPluginResp, ()>>,
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
}

impl From<Module> for JsModule {
  fn from(value: Module) -> Self {
    Self {
      id: value.id,
      name: value.name,
      size: value.size as u32,
      chunks: value.chunks,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunk {
  pub id: String,
  pub names: Vec<String>,
  pub size: u32,
  pub modules: Vec<String>,
  pub entry: bool,
  pub initial: bool,
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
}

impl From<Package> for JsPackage {
  fn from(value: Package) -> Self {
    Self {
      name: value.name,
      version: value.version,
      size: value.size as u32,
      module_count: value.module_count as u32,
      modules: value.modules,
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
