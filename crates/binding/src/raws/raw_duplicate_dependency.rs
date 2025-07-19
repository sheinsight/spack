use std::sync::Arc;

use derive_more::Debug;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::Status;
use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_duplicate_dependency::{
  CompilationHookFn, DuplicateDependencyPlugin, DuplicateDependencyPluginOpts,
  DuplicateDependencyPluginResp, Library, LibraryGroup,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDuplicateDependencyPluginOpts {
  #[napi(
    ts_type = "(error?: Error, response?: JsDuplicateDependencyPluginResp) => void|Promise<void>"
  )]
  #[debug(skip)]
  pub on_detected: Option<
    ThreadsafeFunction<
      JsDuplicateDependencyPluginResp,
      (),
      JsDuplicateDependencyPluginResp,
      Status,
      true,
      false,
      0,
    >,
  >,
}

impl Into<DuplicateDependencyPluginOpts> for RawDuplicateDependencyPluginOpts {
  fn into(self) -> DuplicateDependencyPluginOpts {
    let on_detected: Option<CompilationHookFn> = match self.on_detected {
      Some(callback) => {
        let callback = Arc::new(callback);
        Some(Box::new(move |response| {
          let callback = callback.clone();
          let response = JsDuplicateDependencyPluginResp::from(response);
          Box::pin(async move {
            callback.call(Ok(response), ThreadsafeFunctionCallMode::Blocking);
          })
        }))
      }
      _ => None,
    };
    DuplicateDependencyPluginOpts { on_detected }
  }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsLibraryGroup {
  pub name: String,
  pub libraries: Vec<JsLibrary>,
}

impl From<LibraryGroup> for JsLibraryGroup {
  fn from(value: LibraryGroup) -> Self {
    Self {
      name: value.name.clone(),
      libraries: value.libraries.into_iter().map(|l| l.into()).collect(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsDuplicateDependencyPluginResp {
  pub library_groups: Vec<JsLibraryGroup>,
  pub duration: f64,
}

impl From<DuplicateDependencyPluginResp> for JsDuplicateDependencyPluginResp {
  fn from(value: DuplicateDependencyPluginResp) -> Self {
    Self {
      library_groups: value
        .library_groups
        .into_iter()
        .map(|lg| lg.into())
        .collect(),
      duration: value.duration,
    }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDuplicateDependencyPluginOpts::from_unknown(options)?;
  Ok(Box::new(DuplicateDependencyPlugin::new(options.into())) as BoxPlugin)
}
