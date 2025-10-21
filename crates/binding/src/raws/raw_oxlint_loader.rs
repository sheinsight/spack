use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_builtin_loader::{OxLintLoaderOpts, UnifiedLoaderPlugin, UnifiedLoaderPluginOpts};

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

#[allow(unused)]
pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawOxLintLoaderPluginOpts::from_unknown(options)?;
  Ok(Box::new(UnifiedLoaderPlugin::new(UnifiedLoaderPluginOpts {
    style_loader: None,
    oxlint_loader: Some(options.into()),
  })) as BoxPlugin)
}
