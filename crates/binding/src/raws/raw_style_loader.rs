use std::{collections::HashMap, str::FromStr};

use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_loader_demo::{InjectType, StyleLoaderOpts, StyleLoaderPlugin};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawStyleLoaderPluginOpts {
  #[napi(js_name = "base")]
  pub base: Option<i64>,
  #[napi(js_name = "injectType")]
  pub inject_type: Option<String>,
  #[napi(js_name = "esModule")]
  pub es_module: Option<bool>,
  #[napi(js_name = "insert")]
  pub insert: Option<String>,
  #[napi(js_name = "output")]
  pub output: String,
  #[napi(js_name = "attributes")]
  pub attributes: Option<HashMap<String, String>>, // 使用 Option 让字段可选
}

impl From<RawStyleLoaderPluginOpts> for StyleLoaderOpts {
  fn from(value: RawStyleLoaderPluginOpts) -> Self {
    Self {
      base: value.base,
      inject_type: value.inject_type.map(|s| InjectType::from_str(&s).unwrap()),
      es_module: value.es_module,
      insert: value.insert,
      output: value.output,
      attributes: value.attributes,
    }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawStyleLoaderPluginOpts::from_unknown(options)?;
  Ok(Box::new(StyleLoaderPlugin::new(options.into())) as BoxPlugin)
}
