#![feature(let_chains)]

mod raws;

use napi_derive::napi;
use spack_macros::plugin_registry;

#[napi(string_enum)]
#[plugin_registry]
pub enum CustomPluginNames {
  #[register(raws::raw_duplicate_dependency::binding)]
  DuplicateDependencyPlugin,
  #[register(raws::raw_case_sensitive_paths::binding)]
  CaseSensitivePathsPlugin,
  #[register(raws::raw_bundle_analyzer::binding)]
  BundleAnalyzerPlugin,
  #[register(raws::raw_demo::binding)]
  DemoPlugin,
}
