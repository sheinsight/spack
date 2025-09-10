use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_loader_demo::{DemoLoaderPlugin, DemoLoaderPluginOpts, InjectType};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDemoLoaderPluginOpts {
  #[napi(js_name = "injectType")]
  pub inject_type: String,
}

impl From<RawDemoLoaderPluginOpts> for DemoLoaderPluginOpts {
  fn from(value: RawDemoLoaderPluginOpts) -> Self {
    let inject_type = match value.inject_type.as_str() {
      "style-tag" => InjectType::StyleTag,
      _ => InjectType::StyleTag, // 默认值
    };
    Self { inject_type }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDemoLoaderPluginOpts::from_unknown(options)?;
  Ok(Box::new(DemoLoaderPlugin::new(options.into())) as BoxPlugin)
}
