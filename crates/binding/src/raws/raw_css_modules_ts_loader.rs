use napi_derive::napi;
use spack_builtin_loader::css_modules_ts_loader::{CssModulesTsLoaderOpts, Mode};

#[derive(Debug, Clone)]
#[napi(string_enum)]
pub enum RawMode {
  VERIFY,
  EMIT,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCssModulesTsLoaderPluginOpts {
  #[napi(js_name = "mode")]
  pub mode: Option<RawMode>,
}

impl From<RawCssModulesTsLoaderPluginOpts> for CssModulesTsLoaderOpts {
  fn from(value: RawCssModulesTsLoaderPluginOpts) -> Self {
    Self {
      mode: match value.mode {
        Some(RawMode::VERIFY) => Mode::VERIFY,
        Some(RawMode::EMIT) => Mode::EMIT,
        None => Mode::VERIFY, // or some default
      },
    }
  }
}
