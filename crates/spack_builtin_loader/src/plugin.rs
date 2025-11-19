use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_core::{
  Alias, ApplyContext, BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader,
  Plugin, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;
use serde::Serialize;

use crate::{
  css_modules_dts_loader::{
    CSS_MODULES_TS_LOADER_IDENTIFIER, CssModulesDtsLoader, CssModulesDtsLoaderOpts,
  },
  style_loader::{STYLE_LOADER_IDENTIFIER, StyleLoader, StyleLoaderOpts},
};

pub const UNIFIED_LOADER_PLUGIN_IDENTIFIER: &str = "Spack.UnifiedLoaderPlugin";

const ALIAS_NAME: &str = "@@";

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedLoaderPluginOpts {
  // pub base_dir: String,
  pub style_loader: Option<StyleLoaderOpts>,

  pub css_modules_ts: Option<CssModulesDtsLoaderOpts>,
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

  pub fn write_runtime_by_alias(&self, alias_config: &Option<Alias>) -> Result<()> {
    let err_msg = "UnifiedLoaderPlugin requires the alias '@@' to be configured.”";

    let Some(alias) = alias_config else {
      return Err(rspack_error::error!(err_msg.to_string()));
    };

    let tuple_aliases = match alias {
      Alias::MergeAlias(items) => items,
      Alias::OverwriteToNoAlias => {
        return Err(rspack_error::error!(err_msg.to_string()));
      }
    };

    let Some((_, aliases)) = tuple_aliases.iter().find(|(k, _v)| k == ALIAS_NAME) else {
      return Err(rspack_error::error!(err_msg.to_string()));
    };

    if aliases.is_empty() {
      return Err(rspack_error::error!(err_msg.to_string()));
    }

    // let base_dir = Utf8PathBuf::from(self.options.base_dir.clone());

    if let Some(style_loader) = &self.options.style_loader {
      // let path = base_dir.join(&style_loader.output_dir);
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
    self.write_runtime_by_alias(&ctx.compiler_options.resolve.alias)?;

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

  if let Some(css_modules_ts) = &self.options.css_modules_ts
    && loader_request.starts_with(CSS_MODULES_TS_LOADER_IDENTIFIER)
  {
    return Ok(Some(Arc::new(CssModulesDtsLoader::new(
      css_modules_ts.clone(),
    ))));
  }

  Ok(None)
}
