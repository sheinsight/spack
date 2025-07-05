use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_case_sensitive_paths::{
  CaseSensitivePathsPlugin, CaseSensitivePathsPluginOptions,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCaseSensitivePathsPluginOptions {
  pub debug: bool,
  pub use_cache: bool,
}

impl From<RawCaseSensitivePathsPluginOptions> for CaseSensitivePathsPluginOptions {
  fn from(value: RawCaseSensitivePathsPluginOptions) -> Self {
    Self {
      debug: value.debug,
      use_cache: value.use_cache,
    }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawCaseSensitivePathsPluginOptions::from_unknown(options)?;
  Ok(Box::new(CaseSensitivePathsPlugin::new(options.into())) as BoxPlugin)
}
