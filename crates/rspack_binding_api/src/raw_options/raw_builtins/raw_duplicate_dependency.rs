use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use spack_plugin_duplicate_dependency::{
  CompilationHookFn, DuplicateDependencyPluginOptions, DuplicateDependencyPluginResponse, Library,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDuplicateDependencyPluginOptions {
  #[napi(ts_type = "(response: DuplicateDependencyPluginResponse) => void")]
  pub on_detected: Option<ThreadsafeFunction<JsDuplicateDependencyPluginResponse, ()>>,
}

impl From<RawDuplicateDependencyPluginOptions> for DuplicateDependencyPluginOptions {
  fn from(value: RawDuplicateDependencyPluginOptions) -> Self {
    let on_detected: Option<CompilationHookFn> = match value.on_detected {
      Some(callback) => Some(Box::new(move |libraries| {
        let callback = callback.clone();
        let response = JsDuplicateDependencyPluginResponse::from(libraries);
        Box::pin({
          async move {
            callback.call_with_sync(response).await?;
            Ok(())
          }
        })
      })),
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
pub struct JsDuplicateDependencyPluginResponse {
  pub libraries: Vec<JsLibrary>,
  pub duration_millis: f64,
}

impl From<DuplicateDependencyPluginResponse> for JsDuplicateDependencyPluginResponse {
  fn from(value: DuplicateDependencyPluginResponse) -> Self {
    Self {
      libraries: value.libraries.into_iter().map(|l| l.into()).collect(),
      duration_millis: value.duration_millis,
    }
  }
}
