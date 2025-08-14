use rspack_core::{
  ApplyContext, Compilation, CompilerAfterEmit, CompilerOptions, EntrypointsStatsOption,
  ExtendedStatsOptions, Plugin, PluginContext,
};
use rspack_hook::{plugin, plugin_hook};

#[derive(Debug, Clone)]
pub struct DeadcodePluginOpts {
  // pub on_analyzed: Option<CompilationHookFn>,
}

#[plugin]
#[derive(Debug)]
pub struct DeadcodePlugin {
  #[allow(unused)]
  options: DeadcodePluginOpts,
}

impl DeadcodePlugin {
  pub fn new(options: DeadcodePluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for DeadcodePlugin {
  fn name(&self) -> &'static str {
    "spack.DeadcodePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>) -> rspack_error::Result<()> {
    ctx
      .context
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilerAfterEmit for DeadcodePlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let stats = compilation.get_stats();

  stats.get_modules(
    &ExtendedStatsOptions {
      assets: false,
      cached_modules: false,
      chunks: false,
      chunk_group_auxiliary: false,
      chunk_group_children: false,
      chunk_groups: false,
      chunk_modules: false,
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
      provided_exports: true,
      reasons: false,
      source: false,
      used_exports: true,
      warnings: false,
    },
    |modules| {
      for module in modules {
        if module
          .name
          .as_ref()
          .map(|name| name.contains("node_modules"))
          .unwrap_or(false)
        {
          continue;
        }

        if let (Some(provided), Some(used)) = (module.provided_exports, module.used_exports) {
          let unused: Vec<_> = provided
            .iter()
            .filter(|exp| match &used {
              rspack_core::StatsUsedExports::Vec(atoms) => !atoms.contains(exp),
              rspack_core::StatsUsedExports::Bool(all_used) => !all_used,
              rspack_core::StatsUsedExports::Null => true,
            })
            .collect();
          if !unused.is_empty() {
            println!(
              "name: {}, unused: {:#?}",
              module.name.as_ref().unwrap_or(&"none".into()),
              unused
            );
          }
        }
      }
    },
  )?;
  Ok(())
}
