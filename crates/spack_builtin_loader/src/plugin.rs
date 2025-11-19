use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_core::{
  ApplyContext, BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin,
  Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;
use serde::Serialize;

use crate::{
  css_modules_dts_loader::{
    CSS_MODULES_DTS_LOADER_IDENTIFIER, CssModulesDtsLoader, CssModulesDtsLoaderOpts,
  },
  style_loader::{STYLE_LOADER_IDENTIFIER, StyleLoader, StyleLoaderOpts},
};

pub const UNIFIED_LOADER_PLUGIN_IDENTIFIER: &str = "Spack.UnifiedLoaderPlugin";

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedLoaderPluginOpts {
  pub style_loader: Option<StyleLoaderOpts>,
  pub css_modules_dts_loader: Option<CssModulesDtsLoaderOpts>,
}

#[plugin]
#[derive(Debug)]
pub struct UnifiedLoaderPlugin {
  #[allow(unused)]
  options: UnifiedLoaderPluginOpts,
}

impl UnifiedLoaderPlugin {
  pub fn new(options: UnifiedLoaderPluginOpts) -> Self {
    Self::new_inner(options)
  }

  pub fn write_runtime_by_alias(&self) -> Result<()> {
    if let Some(style_loader) = &self.options.style_loader {
      StyleLoader::write_runtime(&Utf8PathBuf::from(&style_loader.output_dir))?;
    }

    Ok(())
  }
}

impl Plugin for UnifiedLoaderPlugin {
  fn name(&self) -> &'static str {
    UNIFIED_LOADER_PLUGIN_IDENTIFIER
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    self.write_runtime_by_alias()?;

    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));

    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {}
}

#[plugin_hook(NormalModuleFactoryResolveLoader for UnifiedLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;

  if let Some(style_loader) = &self.options.style_loader
    && loader_request.starts_with(STYLE_LOADER_IDENTIFIER)
  {
    return Ok(Some(Arc::new(StyleLoader::new(style_loader.clone()))));
  }

  if let Some(css_modules_dts_loader_opts) = &self.options.css_modules_dts_loader
    && loader_request.starts_with(CSS_MODULES_DTS_LOADER_IDENTIFIER)
  {
    return Ok(Some(Arc::new(CssModulesDtsLoader::new(
      css_modules_dts_loader_opts.clone(),
    ))));
  }

  Ok(None)
}
