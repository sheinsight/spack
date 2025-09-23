use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_case_sensitive_paths::{CaseSensitivePathsPlugin, CaseSensitivePathsPluginOpts};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCaseSensitivePathsPluginOpts {}

impl From<RawCaseSensitivePathsPluginOpts> for CaseSensitivePathsPluginOpts {
  fn from(_value: RawCaseSensitivePathsPluginOpts) -> Self {
    Self {}
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawCaseSensitivePathsPluginOpts::from_unknown(options)?;
  Ok(Box::new(CaseSensitivePathsPlugin::new(options.into())) as BoxPlugin)
}
