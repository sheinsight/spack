use rspack_core::{ApplyContext, Plugin};
use rspack_hook::plugin;

const FRIENDLY_ERRORS_PLUGIN_IDENTIFIER: &str = "Spack.FriendlyErrorsPlugin";

#[derive(Debug)]
pub struct FriendlyErrorsPluginOpts {}

#[plugin]
#[derive(Debug)]
pub struct FriendlyErrorsPlugin {
  #[allow(unused)]
  options: FriendlyErrorsPluginOpts,
}

impl FriendlyErrorsPlugin {
  pub fn new(options: FriendlyErrorsPluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for FriendlyErrorsPlugin {
  fn name(&self) -> &'static str {
    FRIENDLY_ERRORS_PLUGIN_IDENTIFIER
  }

  fn apply(&self, _ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    Ok(())
  }
}
