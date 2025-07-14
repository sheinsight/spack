use derive_more::Debug;
use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_deadcode::{DeadcodePlugin, DeadcodePluginOpts};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDeadcodePluginOpts {
  // #[napi(ts_type = "(response: JsDeadcodePluginResp) => Promise<void>")]
  // #[debug(skip)]
  // pub on_detected: Option<ThreadsafeFunction<JsDeadcodePluginResp, ()>>,
}

impl From<RawDeadcodePluginOpts> for DeadcodePluginOpts {
  fn from(_value: RawDeadcodePluginOpts) -> Self {
    Self {}
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDeadcodePluginOpts::from_unknown(options)?;
  Ok(Box::new(DeadcodePlugin::new(options.into())) as BoxPlugin)
}
