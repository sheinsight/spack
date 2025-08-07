use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_demo::{DemoPlugin, DemoPluginOpts};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDemoPluginOpts {}

impl From<RawDemoPluginOpts> for DemoPluginOpts {
  fn from(_value: RawDemoPluginOpts) -> Self {
    Self {}
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDemoPluginOpts::from_unknown(options)?;
  Ok(Box::new(DemoPlugin::new(options.into())) as BoxPlugin)
}
