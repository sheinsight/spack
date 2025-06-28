use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use spack_plugin_duplicate_dependencies::{
  CompilationHookFn, DuplicateDependenciesPluginOptions, Library,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDuplicateDependenciesPluginOptions {
  #[napi(ts_type = "(libraries: Library[]) => void")]
  pub on_detected: Option<ThreadsafeFunction<Vec<JsLibrary>, ()>>,
}

impl From<RawDuplicateDependenciesPluginOptions> for DuplicateDependenciesPluginOptions {
  fn from(value: RawDuplicateDependenciesPluginOptions) -> Self {
    let on_detected: Option<CompilationHookFn> = match value.on_detected {
      Some(callback) => Some(Box::new(move |libraries| {
        let callback = callback.clone();
        let libraries = libraries.into_iter().map(|l| l.into()).collect();
        Box::pin({
          async move {
            callback.call_with_sync(libraries).await?;
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
