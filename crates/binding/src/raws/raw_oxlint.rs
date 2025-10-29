use std::collections::HashMap;

use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use spack_plugin_oxlint::{OxlintPlugin, OxlintPluginOpts};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOxlintPluginOpts {
  /// runtime 文件的生成目录 , 请保证存在 @@ 的 alias 配置
  #[napi(js_name = "outputDir")]
  pub output_dir: String,

  #[napi(js_name = "showWarning")]
  pub show_warning: Option<bool>,

  #[napi(js_name = "restrictedImports")]
  pub restricted_imports: Option<Vec<RawRestricted>>,

  #[napi(js_name = "restrictedGlobals")]
  pub restricted_globals: Option<Vec<RawRestricted>>,

  #[napi(js_name = "globals")]
  pub globals: Option<HashMap<String, bool>>,

  #[napi(js_name = "environments")]
  pub environments: Option<RawEnvironment>,

  #[napi(js_name = "oxlintConfigFilePath")]
  pub oxlint_config_file_path: Option<String>,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawEnvironment {
  #[napi(js_name = "browser")]
  pub browser: Option<bool>,
  #[napi(js_name = "node")]
  pub node: Option<bool>,
  #[napi(js_name = "commonjs")]
  pub commonjs: Option<bool>,
  #[napi(js_name = "es2024")]
  pub es2024: Option<bool>,
  #[napi(js_name = "amd")]
  pub amd: Option<bool>,
  #[napi(js_name = "sharedNodeBrowser")]
  pub shared_node_browser: Option<bool>,
}

impl From<RawEnvironment> for spack_plugin_oxlint::Environment {
  fn from(value: RawEnvironment) -> Self {
    Self {
      browser: value.browser.unwrap_or(true),
      node: value.node.unwrap_or(true),
      commonjs: value.commonjs.unwrap_or(false),
      es2024: value.es2024.unwrap_or(true),
      amd: value.amd.unwrap_or(false),
      shared_node_browser: value.shared_node_browser.unwrap_or(false),
    }
  }
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRestricted {
  #[napi(js_name = "name")]
  pub name: String,

  #[napi(js_name = "message")]
  pub message: String,
}

impl From<RawRestricted> for spack_plugin_oxlint::Restricted {
  fn from(value: RawRestricted) -> Self {
    Self {
      name: value.name,
      message: value.message,
    }
  }
}

impl From<RawOxlintPluginOpts> for OxlintPluginOpts {
  fn from(value: RawOxlintPluginOpts) -> Self {
    let environments = value
      .environments
      .map(|e| e.into())
      .unwrap_or(spack_plugin_oxlint::Environment::default())
      .into();

    let output_dir = value.output_dir;

    let show_warning = value.show_warning.unwrap_or(true);

    let restricted_imports = value
      .restricted_imports
      .unwrap_or_default()
      .into_iter()
      .map(From::from)
      .collect();

    let restricted_globals = value
      .restricted_globals
      .unwrap_or_default()
      .into_iter()
      .map(From::from)
      .collect();

    let globals = value.globals.unwrap_or_default();

    let oxlint_config_file_path = value.oxlint_config_file_path;

    Self {
      base_dir: "src".to_string(),
      output_dir,
      show_warning,
      restricted_imports,
      restricted_globals,
      globals,
      environments,
      oxlint_config_file_path,
    }
  }
}

#[allow(unused)]
pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawOxlintPluginOpts::from_unknown(options)?;
  Ok(Box::new(OxlintPlugin::new(options.into())) as BoxPlugin)
}
