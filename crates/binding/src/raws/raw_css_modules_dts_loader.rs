use napi_derive::napi;
use spack_builtin_loader::css_modules_dts_loader::{CssModulesDtsLoaderOpts, Mode};

#[derive(Debug, Clone)]
#[napi(string_enum)]
pub enum RawMode {
  VERIFY,
  EMIT,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCssModulesDtsLoaderPluginOpts {
  #[napi(js_name = "mode")]
  pub mode: Option<RawMode>,
}

impl From<RawCssModulesDtsLoaderPluginOpts> for CssModulesDtsLoaderOpts {
  fn from(value: RawCssModulesDtsLoaderPluginOpts) -> Self {
    Self {
      mode: match value.mode {
        Some(RawMode::VERIFY) => Mode::VERIFY,
        Some(RawMode::EMIT) => Mode::EMIT,
        None => Mode::VERIFY, // or some default
      },
    }
  }
}
