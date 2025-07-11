#![feature(let_chains)]

mod raws;

// use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
// use rspack_binding_builder_macros::register_plugin;
use spack_binding_builder_macros::register_plugin;
use strum_macros::{Display, EnumString};

#[napi(string_enum)]
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum CustomPluginNames {
  DuplicateDependencyPlugin,
  CaseSensitivePathsPlugin,
  BundleAnalyzerPlugin,
  DeadcodePlugin,
}

register_plugin!(
  CustomPluginNames::DuplicateDependencyPlugin,
  raws::raw_duplicate_dependency::binding
);

register_plugin!(
  CustomPluginNames::CaseSensitivePathsPlugin,
  raws::raw_case_sensitive_paths::binding
);

register_plugin!(
  CustomPluginNames::BundleAnalyzerPlugin,
  raws::raw_bundle_analyzer::binding
);

register_plugin!(
  CustomPluginNames::DeadcodePlugin,
  raws::raw_deadcode::binding
);
