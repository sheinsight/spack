use napi_derive::napi;
use spack_plugin_case_sensitive_paths::CaseSensitivePathsPluginOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCaseSensitivePathsPluginOptions {
  pub debug: bool,
  pub use_cache: bool,
}

impl From<RawCaseSensitivePathsPluginOptions> for CaseSensitivePathsPluginOptions {
  fn from(value: RawCaseSensitivePathsPluginOptions) -> Self {
    Self {
      debug: value.debug,
      use_cache: value.use_cache,
    }
  }
}
