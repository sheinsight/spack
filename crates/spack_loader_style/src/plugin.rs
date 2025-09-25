use std::{ops::Not, sync::Arc};

use rspack_core::{
  ApplyContext, BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin,
  Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;

use crate::loader::{STYLE_LOADER_IDENTIFIER, StyleLoader, StyleLoaderOpts};

pub const STYLE_LOADER_PLUGIN_IDENTIFIER: &str = "Spack.StyleLoaderPlugin";

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

  pub fn write_runtime(dir: &Utf8PathBuf) -> Result<()> {
    if dir.exists().not() {
      std::fs::create_dir_all(dir)?;
    }

    let runtimes = vec![
      (
        "injectStylesIntoLinkTag.js",
        include_str!("runtime/injectStylesIntoLinkTag.js").to_string(),
      ),
      (
        "injectStylesIntoStyleTag.js",
        include_str!("runtime/injectStylesIntoStyleTag.js").to_string(),
      ),
      (
        "insertStyleElement.js",
        include_str!("runtime/insertStyleElement.js").to_string(),
      ),
      (
        "insertBySelector.js",
        include_str!("runtime/insertBySelector.js").to_string(),
      ),
      (
        "setAttributesWithoutAttributes.js",
        include_str!("runtime/setAttributesWithoutAttributes.js").to_string(),
      ),
      (
        "setAttributesWithAttributes.js",
        include_str!("runtime/setAttributesWithAttributes.js").to_string(),
      ),
      (
        "setAttributesWithAttributesAndNonce.js",
        include_str!("runtime/setAttributesWithAttributesAndNonce.js").to_string(),
      ),
      (
        "setAttributesWithAttributesAndNonce.js",
        include_str!("runtime/setAttributesWithAttributesAndNonce.js").to_string(),
      ),
      (
        "styleTagTransform.js",
        include_str!("runtime/styleTagTransform.js").to_string(),
      ),
      (
        "styleDomAPI.js",
        include_str!("runtime/styleDomAPI.js").to_string(),
      ),
      (
        "singletonStyleDomAPI.js",
        include_str!("runtime/singletonStyleDomAPI.js").to_string(),
      ),
      ("isOldIE.js", include_str!("runtime/isOldIE.js").to_string()),
    ];

    for (file_name, runtime) in runtimes {
      let file = dir.join(file_name);

      if file.exists().not() {
        std::fs::write(file, runtime)?;
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
    if let Some(alias) = &ctx.compiler_options.resolve.alias {
      let value = match alias {
        rspack_core::Alias::OverwriteToNoAlias => None,
        rspack_core::Alias::MergeAlias(items) => {
          let alias_values = items
            .iter()
            .find_map(|(k, v)| if k == "@@" { Some(v) } else { None });

          if let Some(alias) = alias_values {
            alias.get(0)
          } else {
            None
          }
        }
      };

      if let Some(value) = value {
        match value {
          rspack_resolver::AliasValue::Path(path) => {
            let path = path.to_string();
            let path = Utf8PathBuf::from(path).join(&self.options.output);
            Self::write_runtime(&path)?;
          }
          rspack_resolver::AliasValue::Ignore => {
            return Err(rspack_error::error!(
              "StyleLoaderPlugin requires alias to be configured with '@@'"
            ));
          }
        }
      } else {
        return Err(rspack_error::error!(
          "StyleLoaderPlugin requires alias to be configured with '@@'"
        ));
      }
    }

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
  Ok(None)
}
