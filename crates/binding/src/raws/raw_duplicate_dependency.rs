use std::sync::Arc;

use derive_more::Debug;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use rustc_hash::FxHashMap;
use spack_plugin_duplicate_dependency::{
  CompilationHookFn, DuplicateDependencyPlugin, DuplicateDependencyPluginOpts,
  DuplicateDependencyPluginResp, Library,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDuplicateDependencyPluginOpts {
  #[napi(ts_type = "(error: null, response: JsDuplicateDependencyPluginResp) => Promise<void>")]
  #[debug(skip)]
  pub on_detected: Option<ThreadsafeFunction<JsDuplicateDependencyPluginResp, ()>>,
}

impl From<RawDuplicateDependencyPluginOpts> for DuplicateDependencyPluginOpts {
  fn from(value: RawDuplicateDependencyPluginOpts) -> Self {
    let on_detected: Option<CompilationHookFn> = match value.on_detected {
      Some(callback) => {
        let callback = Arc::new(callback);
        Some(Box::new(move |libraries| {
          let callback = callback.clone();
          let response = JsDuplicateDependencyPluginResp::from(libraries);
          Box::pin({
            async move {
              // TODO: handle error
              // callback
              //   .call_async(Ok(response))
              //   .await
              //   .expect("callback error");
              callback
                .call_async(Ok(response)) // 移除 Ok() 包装
                .await
                .map_err(|e| rspack_error::Error::msg(format!("callback error: {}", e)))?;
              Ok(())
            }
          })
        }))
      }
      _ => None,
    };
    Self { on_detected }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct JsLibrary {
  pub dir: String,
  pub name: String,
  pub version: String,
}

impl From<Library> for JsLibrary {
  fn from(value: Library) -> Self {
    Self {
      dir: value.dir.clone(),
      name: value.name.clone(),
      version: value.version.clone(),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct JsDuplicateDependencyPluginResp {
  pub libraries: FxHashMap<String, Vec<JsLibrary>>,
  pub duration: f64,
}

impl From<DuplicateDependencyPluginResp> for JsDuplicateDependencyPluginResp {
  fn from(value: DuplicateDependencyPluginResp) -> Self {
    Self {
      libraries: value
        .libraries
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().map(|l| l.into()).collect()))
        .collect(),
      duration: value.duration,
    }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDuplicateDependencyPluginOpts::from_unknown(options)?;

  Ok(Box::new(DuplicateDependencyPlugin::new(options.into())) as BoxPlugin)
}
