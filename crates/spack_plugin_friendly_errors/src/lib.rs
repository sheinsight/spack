use rspack_core::{ApplyContext, Compilation, CompilerAfterEmit, Plugin};
use rspack_hook::{plugin, plugin_hook};

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

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    ctx.compiler_hooks.after_emit.tap(after_emit::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilerAfterEmit for FriendlyErrorsPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let _diagnostics = compilation.diagnostics();

  // for diagnostic in diagnostics.iter() {}
  Ok(())
}
