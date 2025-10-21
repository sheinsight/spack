use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_unified::{UnifiedPlugin, UnifiedPluginOpts};

use crate::raws::{
  raw_case_sensitive_paths::RawCaseSensitivePathsPluginOpts,
  raw_oxlint_loader::RawOxLintLoaderPluginOpts, raw_style_loader::RawStyleLoaderPluginOpts,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawUnifiedPluginOpts {
  /// style-loader 的配置
  #[napi(js_name = "styleLoader")]
  pub style_loader: Option<RawStyleLoaderPluginOpts>,
  /// oxlint-loader 的配置
  #[napi(js_name = "oxlintLoader")]
  pub oxlint_loader: Option<RawOxLintLoaderPluginOpts>,
  /// case-sensitive-paths 的配置
  #[napi(js_name = "caseSensitive")]
  pub case_sensitive: Option<RawCaseSensitivePathsPluginOpts>,
}

impl From<RawUnifiedPluginOpts> for UnifiedPluginOpts {
  fn from(value: RawUnifiedPluginOpts) -> Self {
    Self {
      style_loader: value.style_loader.map(From::from),
      oxlint_loader: value.oxlint_loader.map(From::from),
      case_sensitive: value.case_sensitive.map(From::from),
    }
  }
}

#[allow(unused)]
pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawUnifiedPluginOpts::from_unknown(options)?;
  Ok(Box::new(UnifiedPlugin::new(options.into())) as BoxPlugin)
}
