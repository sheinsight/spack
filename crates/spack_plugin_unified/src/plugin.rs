use rspack_core::Plugin;
use rspack_error::Result;
use rspack_hook::plugin;
use spack_builtin_loader::{StyleLoaderOpts, UnifiedLoaderPlugin, UnifiedLoaderPluginOpts};
use spack_plugin_case_sensitive_paths::{CaseSensitivePathsPlugin, CaseSensitivePathsPluginOpts};
use spack_plugin_oxlint::{OxLintPlugin, OxLintPluginOpts};

#[derive(Debug)]
pub struct UnifiedPluginOpts {
  pub base_dir: String,
  #[allow(unused)]
  pub style_loader: Option<StyleLoaderOpts>,
  #[allow(unused)]
  pub case_sensitive: Option<CaseSensitivePathsPluginOpts>,
  #[allow(unused)]
  pub oxlint: Option<OxLintPluginOpts>,
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
      base_dir: self.options.base_dir.clone(),
    })
    .apply(ctx)?;

    if let Some(case_sensitive) = self.options.case_sensitive.clone() {
      CaseSensitivePathsPlugin::new(case_sensitive).apply(ctx)?;
    }

    if let Some(oxlint_ops) = self.options.oxlint.clone() {
      OxLintPlugin::new(oxlint_ops).apply(ctx)?;
    }
    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {}
}
