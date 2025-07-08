#![feature(let_chains)]

mod raws;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_binding_builder_macros::register_plugin;
// use rspack_napi::napi::bindgen_prelude::*;

register_plugin!(
  "DuplicateDependencyPlugin",
  raws::raw_duplicate_dependency::binding
);

register_plugin!(
  "CaseSensitivePathsPlugin",
  raws::raw_case_sensitive_paths::binding
);

register_plugin!("BundleAnalyzerPlugin", raws::raw_bundle_analyzer::binding);
