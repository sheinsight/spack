use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_loader_demo::{DemoLoaderPlugin, DemoLoaderPluginOpts};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDemoLoaderPluginOpts {}

impl From<RawDemoLoaderPluginOpts> for DemoLoaderPluginOpts {
  fn from(_value: RawDemoLoaderPluginOpts) -> Self {
    Self {}
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDemoLoaderPluginOpts::from_unknown(options)?;
  Ok(Box::new(DemoLoaderPlugin::new(options.into())) as BoxPlugin)
}
