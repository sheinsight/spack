use napi_derive::napi;
use spack_builtin_loader::{OxLintLoaderOpts, restricted::Restricted};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOxLintLoaderPluginOpts {
  /// runtime 文件的生成目录 , 请保证存在 @@ 的 alias 配置
  #[napi(js_name = "outputDir")]
  pub output_dir: String,

  #[napi(js_name = "showWarning")]
  pub show_warning: bool,

  #[napi(js_name = "restrictedImports")]
  pub restricted_imports: Vec<RawRestricted>,

  #[napi(js_name = "restrictedGlobals")]
  pub restricted_globals: Vec<RawRestricted>,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRestricted {
  #[napi(js_name = "name")]
  pub name: String,

  #[napi(js_name = "message")]
  pub message: String,
}

impl From<RawRestricted> for Restricted {
  fn from(value: RawRestricted) -> Self {
    Self {
      name: value.name,
      message: value.message,
    }
  }
}

impl From<RawOxLintLoaderPluginOpts> for OxLintLoaderOpts {
  fn from(value: RawOxLintLoaderPluginOpts) -> Self {
    Self {
      output_dir: value.output_dir,
      show_warning: value.show_warning,
      restricted_imports: value
        .restricted_imports
        .into_iter()
        .map(From::from)
        .collect(),
      restricted_globals: value
        .restricted_globals
        .into_iter()
        .map(From::from)
        .collect(),
    }
  }
}
