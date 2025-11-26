use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::{
  ApplyContext, BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin,
  Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoResponse {
  pub name: String,
  pub age: i32,
}

pub type CycleHandlerFn = Box<dyn Fn(DemoResponse) -> BoxFuture<'static, Result<()>> + Sync + Send>;

#[derive(Debug)]
pub struct DemoPluginOpts {
  #[debug(skip)]
  pub on_detected: Option<CycleHandlerFn>,
}

#[plugin]
#[derive(Debug)]
pub struct DemoRspackPlugin {
  #[allow(unused)]
  options: DemoPluginOpts,
}

impl DemoRspackPlugin {
  pub fn new(options: DemoPluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for DemoRspackPlugin {
  fn name(&self) -> &'static str {
    "spack.JsLoaderRspackPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    println!("apply start >>>>");

    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));

    println!("apply end >>>>");
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for DemoRspackPlugin,stage = -1)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  if let Some(on_detected) = &self.options.on_detected {
    let future = on_detected(DemoResponse {
      name: "test".to_string(),
      age: 18,
    });
    future.await?;
  };
  Ok(None)
}
