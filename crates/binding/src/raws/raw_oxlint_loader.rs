use napi_derive::napi;
use spack_builtin_loader::OxLintLoaderOpts;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOxLintLoaderPluginOpts {
  /// runtime 文件的生成目录 , 请保证存在 @@ 的 alias 配置
  #[napi(js_name = "output")]
  pub output: String,
}

impl From<RawOxLintLoaderPluginOpts> for OxLintLoaderOpts {
  fn from(value: RawOxLintLoaderPluginOpts) -> Self {
    Self {
      output: value.output,
    }
  }
}
