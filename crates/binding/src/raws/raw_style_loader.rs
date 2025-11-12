use std::collections::HashMap;

use napi_derive::napi;
use spack_builtin_loader::StyleLoaderOpts;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawStyleLoaderPluginOpts {
  #[napi(js_name = "base")]
  pub base: Option<i64>,
  #[napi(js_name = "insert")]
  pub insert: Option<String>,
  /// runtime 文件的生成目录 , 请保证存在 @@ 的 alias 配置
  #[napi(js_name = "outputDir")]
  pub output_dir: String,
  /// 模块引用时的前缀路径，例如 "@@/runtime"
  #[napi(js_name = "importPrefix")]
  pub import_prefix: String,
  #[napi(js_name = "styleTagTransform")]
  pub style_tag_transform: Option<String>,
  /// 为 style 标签添加的属性
  #[napi(js_name = "attributes")]
  pub attributes: Option<HashMap<String, String>>, // 使用 Option 让字段可选
}

impl From<RawStyleLoaderPluginOpts> for StyleLoaderOpts {
  fn from(value: RawStyleLoaderPluginOpts) -> Self {
    Self {
      base: value.base,
      insert: value.insert,
      output_dir: value.output_dir,
      import_prefix: value.import_prefix,
      style_tag_transform: value.style_tag_transform,
      attributes: value.attributes,
    }
  }
}
