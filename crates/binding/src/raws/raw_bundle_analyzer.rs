use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_bundle_analyzer::{BundleAnalyzerPlugin, BundleAnalyzerPluginOpts};
// use napi_derive::napi;
// use spack_plugin_bundle_analyzer::{BundleAnalyzerPlugin, BundleAnalyzerPluginOpts, ModuleInfo};

#[derive(Debug)]
#[napi(object)]
pub struct RawBundleAnalyzerPluginOpts {
  // #[napi(ts_type = "async (response: JsBundleAnalyzerPluginResp) => void")]
  // #[debug(skip)]
  // pub on_analyzed: Option<ThreadsafeFunction<JsBundleAnalyzerPluginResp, ()>>,
}

impl From<RawBundleAnalyzerPluginOpts> for BundleAnalyzerPluginOpts {
  fn from(_value: RawBundleAnalyzerPluginOpts) -> Self {
    Self {}
  }
}

// #[derive(Debug)]
// #[napi(object)]
// pub struct JsModule {
//   pub name: String,
//   pub size: u64,
//   pub path: String,
//   pub dependencies: Vec<String>,
// }

// #[derive(Debug)]
// #[napi(object)]
// pub struct JsBundleAnalyzerPluginResp {
//   pub modules: Vec<JsModule>,
//   pub duration: f64,
// }

// impl From<ModuleInfo> for JsModule {
//   fn from(value: ModuleInfo) -> Self {
//     let ModuleInfo {
//       name,
//       size,
//       path,
//       dependencies,
//     } = value;
//     Self {
//       name,
//       size,
//       path,
//       dependencies,
//     }
//   }
// }

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawBundleAnalyzerPluginOpts::from_unknown(options)?;
  Ok(Box::new(BundleAnalyzerPlugin::new(options.into())) as BoxPlugin)
}
