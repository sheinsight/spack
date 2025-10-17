use std::sync::Arc;

use rspack_core::{
  Alias, ApplyContext, BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader,
  Plugin, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;
use rspack_resolver::AliasValue;

use crate::{
  OXLINT_LOADER_IDENTIFIER, OxlintLoader, OxlintLoaderOpts,
  style_loader::{STYLE_LOADER_IDENTIFIER, StyleLoader, StyleLoaderOpts},
};

pub const STYLE_LOADER_PLUGIN_IDENTIFIER: &str = "Spack.StyleLoaderPlugin";

const ALIAS_NAME: &str = "@@";

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

  pub fn write_runtime_by_alias(&self, alias_config: &Option<Alias>) -> Result<()> {
    let err_msg = "StyleLoaderPlugin requires the alias '@@' to be configured.”";

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

    for alias in aliases {
      if let AliasValue::Path(path) = alias {
        let path = Utf8PathBuf::from(path.to_string()).join(&self.options.output);
        StyleLoader::write_runtime(&path)?;
      }
    }

    Ok(())
  }
}

impl Plugin for StyleLoaderPlugin {
  fn name(&self) -> &'static str {
    STYLE_LOADER_PLUGIN_IDENTIFIER
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

#[plugin_hook(NormalModuleFactoryResolveLoader for StyleLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;

  if loader_request.starts_with(STYLE_LOADER_IDENTIFIER) {
    return Ok(Some(Arc::new(StyleLoader::new(self.options.clone()))));
  }
  if loader_request.starts_with(OXLINT_LOADER_IDENTIFIER) {
    return Ok(Some(Arc::new(OxlintLoader::new(OxlintLoaderOpts {}))));
  }
  // if loader_request.starts_with(CSS_LOADER_IDENTIFIER) {
  //   return Ok(Some(Arc::new(CssLoader::new(CssLoaderOpts {}))));
  // }
  Ok(None)
}
