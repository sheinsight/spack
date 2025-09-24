use rspack_core::Plugin;
use rspack_error::Result;
use rspack_hook::plugin;
use spack_loader_style::{StyleLoaderOpts, StyleLoaderPlugin};
use spack_plugin_case_sensitive_paths::{CaseSensitivePathsPlugin, CaseSensitivePathsPluginOpts};

#[derive(Debug)]
pub struct UnifiedPluginOpts {
  #[allow(unused)]
  pub style_loader: Option<StyleLoaderOpts>,
  #[allow(unused)]
  pub case_sensitive: Option<CaseSensitivePathsPluginOpts>,
}

pub const UNIFIED_PLUGIN_IDENTIFIER: &str = "spack.UnifiedPlugin";

#[plugin]
#[derive(Debug)]
pub struct UnifiedPlugin {
  #[allow(unused)]
  options: UnifiedPluginOpts,
}

impl UnifiedPlugin {
  pub fn new(options: UnifiedPluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for UnifiedPlugin {
  fn name(&self) -> &'static str {
    UNIFIED_PLUGIN_IDENTIFIER
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext) -> Result<()> {
    if let Some(style_loader) = self.options.style_loader.clone() {
      StyleLoaderPlugin::new(style_loader).apply(ctx)?;
    }

    if let Some(case_sensitive) = self.options.case_sensitive.clone() {
      CaseSensitivePathsPlugin::new(case_sensitive).apply(ctx)?;
    }

    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {}
}
