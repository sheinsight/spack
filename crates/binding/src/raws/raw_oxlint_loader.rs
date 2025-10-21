use napi_derive::napi;
use spack_builtin_loader::OxLintLoaderOpts;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOxLintLoaderPluginOpts {
  /// runtime 文件的生成目录 , 请保证存在 @@ 的 alias 配置
  #[napi(js_name = "outputDir")]
  pub output_dir: String,

  #[napi(js_name = "showWarning")]
  pub show_warning: bool,
}

impl From<RawOxLintLoaderPluginOpts> for OxLintLoaderOpts {
  fn from(value: RawOxLintLoaderPluginOpts) -> Self {
    Self {
      output_dir: value.output_dir,
      show_warning: value.show_warning,
    }
  }
}
