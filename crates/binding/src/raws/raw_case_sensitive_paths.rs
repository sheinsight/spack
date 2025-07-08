use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_case_sensitive_paths::{CaseSensitivePathsPlugin, CaseSensitivePathsPluginOpts};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCaseSensitivePathsPluginOpts {
  pub debug: bool,
  pub use_cache: bool,
}

impl From<RawCaseSensitivePathsPluginOpts> for CaseSensitivePathsPluginOpts {
  fn from(value: RawCaseSensitivePathsPluginOpts) -> Self {
    Self {
      debug: value.debug,
      use_cache: value.use_cache,
    }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawCaseSensitivePathsPluginOpts::from_unknown(options)?;
  Ok(Box::new(CaseSensitivePathsPlugin::new(options.into())) as BoxPlugin)
}
