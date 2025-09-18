use std::sync::Arc;

use rspack_core::{
  ApplyContext, BoxLoader, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin, Resolver, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{
  loader::{StyleLoader, StyleLoaderOpts, STYLE_LOADER_IDENTIFIER},
  runtime_module::StyleLoaderRuntimeModule,
};

pub const STYLE_LOADER_PLUGIN_IDENTIFIER: &str = "spack.StyleLoaderPlugin";

#[plugin]
#[derive(Debug)]
pub struct StyleLoaderPlugin {
  #[allow(unused)]
  options: StyleLoaderOpts,
}

impl StyleLoaderPlugin {
  pub fn new(options: StyleLoaderOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for StyleLoaderPlugin {
  fn name(&self) -> &'static str {
    STYLE_LOADER_PLUGIN_IDENTIFIER
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));

    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));

    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {}
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for StyleLoaderPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey, // ✅ 这里有 chunk_ukey
  _runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  // runtime_requirements.insert(RuntimeGlobals::MODULE);
  // ✅ 这里可以添加 RuntimeModule

  let es_module = self.options.es_module.unwrap_or(false);

  compilation.add_runtime_module(
    chunk_ukey,
    Box::new(StyleLoaderRuntimeModule::new(Some(*chunk_ukey), es_module)),
  )?;

  Ok(())
}

#[plugin_hook(NormalModuleFactoryResolveLoader for StyleLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;

  if loader_request.starts_with(STYLE_LOADER_IDENTIFIER) {
    return Ok(Some(Arc::new(StyleLoader {
      options: self.options.clone(),
    })));
  }
  Ok(None)
}
