use rspack_core::Plugin;
use rspack_error::Result;
use rspack_hook::plugin;
use spack_builtin_loader::{
  OxlintLoaderOpts, StyleLoaderOpts, UnifiedLoaderPlugin, UnifiedLoaderPluginOpts,
};
use spack_plugin_case_sensitive_paths::{CaseSensitivePathsPlugin, CaseSensitivePathsPluginOpts};

#[derive(Debug)]
pub struct UnifiedPluginOpts {
  #[allow(unused)]
  pub style_loader: Option<StyleLoaderOpts>,
  #[allow(unused)]
  pub case_sensitive: Option<CaseSensitivePathsPluginOpts>,
  #[allow(unused)]
  pub oxlint_loader: Option<OxlintLoaderOpts>,
}

pub const UNIFIED_PLUGIN_IDENTIFIER: &str = "Spack.UnifiedPlugin";

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
    UnifiedLoaderPlugin::new(UnifiedLoaderPluginOpts {
      style_loader: self.options.style_loader.clone(),
      oxlint_loader: self.options.oxlint_loader.clone(),
    })
    .apply(ctx)?;

    if let Some(case_sensitive) = self.options.case_sensitive.clone() {
      CaseSensitivePathsPlugin::new(case_sensitive).apply(ctx)?;
    }

    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {}
}
