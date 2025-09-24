use std::{collections::HashMap, str::FromStr};

use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_loader_style::{InjectType, StyleLoaderOpts, StyleLoaderPlugin};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawStyleLoaderPluginOpts {
  #[napi(js_name = "base")]
  pub base: Option<i64>,
  // #[napi(
  //   js_name = "injectType",
  //   ts_type = "'linkTag' | 'styleTag' | 'singletonStyleTag' | 'autoStyleTag' | 'lazyStyleTag' | 'lazySingletonStyleTag' | 'lazyAutoStyleTag'"
  // )]
  // 暂时只开放 styleTag
  #[napi(js_name = "injectType", ts_type = "'styleTag'")]
  pub inject_type: Option<String>,
  #[napi(js_name = "insert")]
  pub insert: Option<String>,
  #[napi(js_name = "output")]
  pub output: String,
  #[napi(js_name = "styleTagTransform")]
  pub style_tag_transform: Option<String>,
  #[napi(js_name = "attributes")]
  pub attributes: Option<HashMap<String, String>>, // 使用 Option 让字段可选
}

impl From<RawStyleLoaderPluginOpts> for StyleLoaderOpts {
  fn from(value: RawStyleLoaderPluginOpts) -> Self {
    Self {
      base: value.base,
      inject_type: value.inject_type.map(|s| InjectType::from_str(&s).unwrap()),
      insert: value.insert,
      output: value.output,
      style_tag_transform: value.style_tag_transform,
      attributes: value.attributes,
    }
  }
}

#[allow(unused)]
pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawStyleLoaderPluginOpts::from_unknown(options)?;
  Ok(Box::new(StyleLoaderPlugin::new(options.into())) as BoxPlugin)
}
