use derive_more::Debug;
use napi::{bindgen_prelude::FromNapiValue, Env, Unknown};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use spack_macros::ThreadsafeCallback;
use spack_plugin_duplicate_dependency::{
  DuplicateDependencyPlugin, DuplicateDependencyPluginOpts, DuplicateDependencyPluginResp, Library,
  LibraryGroup,
};

#[derive(Debug, ThreadsafeCallback)]
#[napi(object, object_to_js = false)]
pub struct RawDuplicateDependencyPluginOpts {
  #[napi(ts_type = "(response: JsDuplicateDependencyPluginResp) => void|Promise<void>")]
  #[debug(skip)]
  #[threadsafe_callback]
  pub on_detected: Option<ThreadsafeFunction<JsDuplicateDependencyPluginResp, ()>>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsLibrary {
  pub file: String,
  pub name: String,
  pub version: String,
}

impl From<Library> for JsLibrary {
  fn from(value: Library) -> Self {
    Self {
      file: value.file.clone(),
      name: value.name.clone(),
      version: value.version.clone(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsLibraryGroup {
  pub name: String,
  pub libs: Vec<JsLibrary>,
}

impl From<LibraryGroup> for JsLibraryGroup {
  fn from(value: LibraryGroup) -> Self {
    Self {
      name: value.name.clone(),
      libs: value.libs.into_iter().map(|l| l.into()).collect(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsDuplicateDependencyPluginResp {
  pub groups: Vec<JsLibraryGroup>,
  pub duration: f64,
}

impl From<DuplicateDependencyPluginResp> for JsDuplicateDependencyPluginResp {
  fn from(value: DuplicateDependencyPluginResp) -> Self {
    Self {
      groups: value.groups.into_iter().map(|lg| lg.into()).collect(),
      duration: value.duration,
    }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawDuplicateDependencyPluginOpts::from_unknown(options)?;
  Ok(Box::new(DuplicateDependencyPlugin::new(options.into())) as BoxPlugin)
}
