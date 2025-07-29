#![feature(let_chains)]

mod raws;

// use napi::bindgen_prelude::*;
use napi_derive::napi;
// use serde::{Deserialize, Serialize};
// use rspack_binding_builder_macros::register_plugin;
use spack_binding_builder_macros::plugin_registry;
// use strum_macros::{Display, EnumString};

#[napi(string_enum)]
// #[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[plugin_registry]
pub enum CustomPluginNames {
  #[register(raws::raw_duplicate_dependency::binding)]
  DuplicateDependencyPlugin,
  #[register(raws::raw_case_sensitive_paths::binding)]
  CaseSensitivePathsPlugin,
  #[register(raws::raw_bundle_analyzer::binding)]
  BundleAnalyzerPlugin,
  #[register(raws::raw_deadcode::binding)]
  DeadcodePlugin,
}
