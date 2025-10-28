use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_unified::{UnifiedPlugin, UnifiedPluginOpts};

use crate::raws::{
  raw_case_sensitive_paths::RawCaseSensitivePathsPluginOpts, raw_oxlint::RawOxLintPluginOpts,
  raw_style_loader::RawStyleLoaderPluginOpts,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawUnifiedPluginOpts {
  /// 输出目录
  #[napi(js_name = "baseDir")]
  pub base_dir: String,
  /// style-loader 的配置
  #[napi(js_name = "styleLoader")]
  pub style_loader: Option<RawStyleLoaderPluginOpts>,
  /// oxlint-loader 的配置
  #[napi(js_name = "oxlintLoader")]
  pub oxlint: Option<RawOxLintPluginOpts>,
  /// case-sensitive-paths 的配置
  #[napi(js_name = "caseSensitive")]
  pub case_sensitive: Option<RawCaseSensitivePathsPluginOpts>,
}

impl From<RawUnifiedPluginOpts> for UnifiedPluginOpts {
  fn from(value: RawUnifiedPluginOpts) -> Self {
    Self {
      base_dir: value.base_dir,
      style_loader: value.style_loader.map(From::from),
      oxlint: value.oxlint.map(From::from),
      case_sensitive: value.case_sensitive.map(From::from),
    }
  }
}

#[allow(unused)]
pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawUnifiedPluginOpts::from_unknown(options)?;
  Ok(Box::new(UnifiedPlugin::new(options.into())) as BoxPlugin)
}
