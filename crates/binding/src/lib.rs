#![feature(let_chains)]

use napi_derive::napi;
use rspack_binding_builder_macros::register_plugin;
use rspack_napi::napi::bindgen_prelude::*;

register_plugin!(
  "DuplicateDependencyPlugin",
  spack_plugin_duplicate_dependency::get_binding_plugin
);

register_plugin!(
  "CaseSensitivePathsPlugin",
  spack_plugin_case_sensitive_paths::get_binding_plugin
);
