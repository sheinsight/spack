use std::str::FromStr;

use napi_derive::napi;
use spack_builtin_loader::css_modules_ts_loader::{CssModulesTsLoaderOpts, Mode};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCssModulesTsLoaderPluginOpts {
  #[napi(js_name = "mode", ts_type = "'verify' | 'emit'")]
  pub mode: Option<String>,
}

impl From<RawCssModulesTsLoaderPluginOpts> for CssModulesTsLoaderOpts {
  fn from(value: RawCssModulesTsLoaderPluginOpts) -> Self {
    Self {
      mode: match value.mode {
        Some(m) => {
          Mode::from_str(&m).expect(format!("mode value only supported ['verify','emit']").as_str())
        }
        _ => Mode::Emit,
      },
    }
  }
}
