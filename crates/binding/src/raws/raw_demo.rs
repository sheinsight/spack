use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_demo::{JsLoaderRspackPlugin, JsLoaderRspackPluginOpts};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDemoPluginOpts {}

impl From<RawDemoPluginOpts> for JsLoaderRspackPluginOpts {
  fn from(_value: RawDemoPluginOpts) -> Self {
    Self {}
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDemoPluginOpts::from_unknown(options)?;
  Ok(Box::new(JsLoaderRspackPlugin::new(options.into())) as BoxPlugin)
}
