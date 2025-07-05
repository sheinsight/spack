#![feature(let_chains)]

use napi_derive::napi;
use rspack_binding_builder::CustomPluginBuilder;
use rspack_binding_builder_macros::register_plugin;
use rspack_core::{ApplyContext, BoxPlugin, CompilerOptions, Plugin, PluginContext};
use rspack_napi::{napi, napi::bindgen_prelude::*};

register_plugin!(
    "DuplicateDependencyPlugin",
    spack_plugin_duplicate_dependency::get_binding_plugin
);
