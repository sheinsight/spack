use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_unified::{UnifiedPlugin, UnifiedPluginOpts};

use crate::raws::{
  raw_case_sensitive_paths::RawCaseSensitivePathsPluginOpts, raw_oxlint::RawOxlintPluginOpts,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawUnifiedPluginOpts {
  /// oxlint-loader 的配置
  #[napi(js_name = "oxlint")]
  pub oxlint: Option<RawOxlintPluginOpts>,
  /// case-sensitive-paths 的配置
  #[napi(js_name = "caseSensitive")]
  pub case_sensitive: Option<RawCaseSensitivePathsPluginOpts>,
}

impl From<RawUnifiedPluginOpts> for UnifiedPluginOpts {
  fn from(value: RawUnifiedPluginOpts) -> Self {
    Self {
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
