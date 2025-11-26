use derive_more::Debug;
use napi::{
  Env, Unknown,
  bindgen_prelude::{FromNapiValue, Promise},
};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use spack_plugin_demo::{CycleHandlerFn, DemoPluginOpts, DemoResponse, DemoRspackPlugin};

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawDemoResponse {
  pub name: String,
  pub age: i32,
}

impl From<DemoResponse> for RawDemoResponse {
  fn from(value: DemoResponse) -> Self {
    Self {
      name: value.name,
      age: value.age,
    }
  }
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDemoPluginOpts {
  #[debug(skip)]
  pub on_detected: Option<ThreadsafeFunction<RawDemoResponse, Promise<()>>>,
}

impl From<RawDemoPluginOpts> for DemoPluginOpts {
  fn from(value: RawDemoPluginOpts) -> Self {
    let on_detected: Option<CycleHandlerFn> = match value.on_detected {
      Some(callback) => Some(Box::new(move |response| {
        let callback = callback.clone();
        Box::pin(async move {
          callback.call_with_promise(response.into()).await?;
          Ok(())
        })
      })),
      _ => None,
    };
    Self { on_detected }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDemoPluginOpts::from_unknown(options)?;
  Ok(Box::new(DemoRspackPlugin::new(options.into())) as BoxPlugin)
}
