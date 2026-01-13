mod asset;
// mod chunk;
mod module;
mod opts;
// mod report;
mod resp;
// mod summary;
mod types;

use derive_more::Debug;
use napi::tokio::time::Instant;
pub use opts::{BundleAnalyzerPluginOpts, CompilationHookFn};
pub use resp::*;
use rspack_collections::Identifier;
use rspack_core::{
  ApplyContext, ChunkGraph, Compilation, CompilerAfterEmit, ModuleIdentifier, Plugin,
};
use rspack_hook::{plugin, plugin_hook};
pub use types::*;

use crate::{asset::Asset, module::Module};

#[plugin]
#[derive(Debug)]
pub struct BundleAnalyzerPlugin {
  #[allow(unused)]
  options: BundleAnalyzerPluginOpts,
}

impl BundleAnalyzerPlugin {
  pub fn new(options: BundleAnalyzerPluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for BundleAnalyzerPlugin {
  fn name(&self) -> &'static str {
    "spack.BundleAnalyzerPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    println!("coming bundle analyzer");
    ctx.compiler_hooks.after_emit.tap(after_emit::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilerAfterEmit for BundleAnalyzerPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let start_time = Instant::now();

  // 1. 收集 Assets（输出文件）
  let assets = collect_assets(compilation);

  // 2. 收集 Modules（源文件）
  let modules = collect_modules(compilation);

  let millis = start_time.elapsed().as_millis();

  assets.iter().for_each(|item| println!("--> {:#?}", item));

  println!("millis {}", millis);

  Ok(())
}

fn collect_assets(compilation: &Compilation) -> Vec<Asset> {
  compilation
    .assets()
    .iter()
    .map(|(name, asset)| {
      let size = if let Some(source) = &asset.source {
        source.size()
      } else {
        0
      };

      return Asset {
        name: name.to_string(),
        size: size,
        chunks: get_asset_chunks(name, compilation),
        emitted: true,
      };
    })
    .collect()
}

fn get_asset_chunks(asset_name: &str, compilation: &Compilation) -> Vec<String> {
  compilation
    .chunk_by_ukey
    .values()
    .filter(|chunk| chunk.files().contains(asset_name))
    .map(|chunk| {
      let id = if let Some(id) = chunk.id() {
        id.to_string()
      } else {
        "".to_string()
      };
      return id;
    })
    .collect()
}

fn collect_modules(compilation: &Compilation) -> Vec<Module> {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;

  module_graph
    .modules()
    .into_iter()
    .map(|(id, module)| Module {
      id: id.to_string(),
      name: module
        .readable_identifier(&compilation.options.context)
        .to_string(),
      size: get_module_size(module.as_ref()),
      chunks: get_module_chunks(&id, chunk_graph),
    })
    .collect()
}

fn get_module_size(module: &dyn rspack_core::Module) -> u64 {
  module
    .original_source()
    .map(|s| s.size() as u64)
    .unwrap_or(0)
}

fn get_module_chunks(module_id: &Identifier, chunk_graph: &ChunkGraph) -> Vec<String> {
  chunk_graph
    .get_module_chunks(*module_id)
    .iter()
    .map(|chunk_ukey| chunk_ukey.as_u32().to_string())
    .collect()
}
