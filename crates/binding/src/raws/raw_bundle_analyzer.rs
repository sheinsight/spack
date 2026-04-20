use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_bundle_analyzer::{BundleAnalyzerPlugin, BundleAnalyzerPluginOpts};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawBundleAnalyzerPluginOpts {
  /// 报告输出目录（默认：当前工作目录）
  #[napi(js_name = "outputDir")]
  pub output_dir: Option<String>,
  /// 是否计算 gzip 压缩后的大小（默认：false）
  pub gzip_assets: Option<bool>,
  /// 是否计算 brotli 压缩后的大小（默认：false）
  pub brotli_assets: Option<bool>,
}

impl From<RawBundleAnalyzerPluginOpts> for BundleAnalyzerPluginOpts {
  fn from(value: RawBundleAnalyzerPluginOpts) -> Self {
    Self {
      output_dir: value.output_dir,
      gzip_assets: value.gzip_assets,
      brotli_assets: value.brotli_assets,
    }
  }
}

#[allow(unused)]
pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawBundleAnalyzerPluginOpts::from_unknown(options)?;
  Ok(Box::new(BundleAnalyzerPlugin::new(options.into())) as BoxPlugin)
}
