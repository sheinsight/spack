#![feature(let_chains)]

use derive_more::Debug;
use napi::tokio::time::Instant;
use rspack_core::{
  ApplyContext, Compilation, CompilerAfterEmit, CompilerOptions, EntrypointsStatsOption,
  ExtendedStatsOptions, ModuleIdentifier, Plugin, PluginContext,
};
use rspack_hook::{plugin, plugin_hook};

#[derive(Debug, Clone)]
pub struct ModuleReasonInfo {
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: Option<String>,
  pub dependency_type: Option<String>,
  pub user_request: Option<String>,
  pub active: bool,
  pub location: Option<String>,
}

#[derive(Debug)]
pub struct BundleAnalyzerPluginOpts {
  // pub on_analyzed: Option<CompilationHookFn>,
}

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

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    println!("BundleAnalyzerPlugin apply");
    ctx
      .context
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilerAfterEmit for BundleAnalyzerPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let start_time = Instant::now();

  let stats = compilation.get_stats();

  stats
    .get_chunks(
      &ExtendedStatsOptions {
        chunks: true,
        chunk_modules: true,
        assets: false,
        cached_modules: false,
        chunk_group_auxiliary: false,
        chunk_group_children: false,
        chunk_groups: false,
        chunk_relations: false,
        depth: false,
        entrypoints: EntrypointsStatsOption::Bool(false),
        errors: false,
        hash: false,
        ids: false,
        modules: false,
        module_assets: false,
        nested_modules: false,
        optimization_bailout: false,
        provided_exports: false,
        reasons: false,
        source: false,
        used_exports: false,
        warnings: false,
      },
      |chunks| {
        for chunk in chunks {
          println!("--->> {:#?}", chunk);
        }
      },
    )
    .unwrap();

  println!(
    "BundleAnalyzerPlugin -> duration -> {:?}",
    start_time.elapsed().as_millis()
  );

  Ok(())
}
