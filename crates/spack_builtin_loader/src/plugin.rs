use once_cell::sync::Lazy;
use rspack_cacheable::cacheable;
use rspack_core::{
  ApplyContext, BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin,
  Resolver,
};
use rspack_error::{Result, SerdeResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;

use crate::{
  css_modules_ts_loader::{CSS_MODULES_TS_LOADER_IDENTIFIER, CssModulesTsLoader},
  lightningcss_loader::{LIGHTNINGCSS_LOADER_IDENTIFIER, LightningcssLoader},
  loader_cache::{LoaderCache, LoaderWithIdentifier},
  style_loader::{STYLE_LOADER_IDENTIFIER, StyleLoader},
};

pub const UNIFIED_LOADER_PLUGIN_IDENTIFIER: &str = "Spack.UnifiedLoaderPlugin";

static STYLE_LOADER_CACHE: Lazy<LoaderCache<StyleLoader>> = Lazy::new(LoaderCache::new);

static CSS_MODULES_TS_LOADER_CACHE: Lazy<LoaderCache<CssModulesTsLoader>> =
  Lazy::new(LoaderCache::new);

static LIGHTNINGCSS_LOADER_CACHE: Lazy<LoaderCache<LightningcssLoader>> =
  Lazy::new(LoaderCache::new);

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedLoaderPluginOpts {}

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
}

impl Plugin for UnifiedLoaderPlugin {
  fn name(&self) -> &'static str {
    UNIFIED_LOADER_PLUGIN_IDENTIFIER
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));

    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {
    tokio::task::block_in_place(|| {
      tokio::runtime::Handle::current().block_on(async {
        STYLE_LOADER_CACHE.clear().await;
        CSS_MODULES_TS_LOADER_CACHE.clear().await;
      })
    });
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for UnifiedLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;
  let options = l.options.as_deref().unwrap_or("{}");

  if loader_request.starts_with(STYLE_LOADER_IDENTIFIER) {
    let loader = STYLE_LOADER_CACHE
      .get_or_insert(loader_request, options, || {
        let options = serde_json::from_str(options).to_rspack_result_with_detail(
          options,
          format!("parse {} options error", STYLE_LOADER_IDENTIFIER).as_ref(),
        )?;
        Ok(StyleLoader::new(options).with_identifier(loader_request.as_str().into()))
      })
      .await?;

    return Ok(Some(loader));
  }

  if loader_request.starts_with(CSS_MODULES_TS_LOADER_IDENTIFIER) {
    let loader = CSS_MODULES_TS_LOADER_CACHE
      .get_or_insert(loader_request, options, || {
        let options = serde_json::from_str(options).to_rspack_result_with_detail(
          options,
          format!("parse {} options error", CSS_MODULES_TS_LOADER_IDENTIFIER).as_ref(),
        )?;
        Ok(CssModulesTsLoader::new(options).with_identifier(loader_request.as_str().into()))
      })
      .await?;

    return Ok(Some(loader));
  }

  if loader_request.starts_with(LIGHTNINGCSS_LOADER_IDENTIFIER) {
    let loader = LIGHTNINGCSS_LOADER_CACHE
      .get_or_insert(loader_request, options, || {
        let options = serde_json::from_str(options).to_rspack_result_with_detail(
          options,
          format!("parse {} options error", LIGHTNINGCSS_LOADER_IDENTIFIER).as_ref(),
        )?;
        Ok(LightningcssLoader::new(options).with_identifier(loader_request.as_str().into()))
      })
      .await?;

    return Ok(Some(loader));
  }

  Ok(None)
}
