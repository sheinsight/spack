use std::sync::Arc;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  ApplyContext, BoxLoader, Context, Loader, LoaderContext, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, Plugin, Resolver, RunnerContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::{Identifiable, Identifier};

#[cacheable]
pub struct DemoLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for DemoLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_DEMO_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    let mut source = content.try_into_string()?;
    source += r#"
    function hello(){
      console.error("hello");
    }
    "#;
    let sm = loader_context.take_source_map();
    loader_context.finish_with((source, sm));

    Ok(())
  }
}
impl Identifiable for DemoLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_DEMO_LOADER_IDENTIFIER.into()
  }
}
pub const SIMPLE_DEMO_LOADER_IDENTIFIER: &str = "builtin:test-demo-loader";

#[derive(Debug)]
pub struct DemoLoaderPluginOpts {}

#[plugin]
#[derive(Debug)]
pub struct DemoLoaderPlugin {
  #[allow(unused)]
  options: DemoLoaderPluginOpts,
}

impl DemoLoaderPlugin {
  pub fn new(options: DemoLoaderPluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for DemoLoaderPlugin {
  fn name(&self) -> &'static str {
    "spack.DemoLoaderPlugin"
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

#[plugin_hook(NormalModuleFactoryResolveLoader for DemoLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;

  if loader_request.starts_with("builtin:test") {
    return Ok(get_builtin_test_loader(loader_request));
  }
  Ok(None)
}

pub fn get_builtin_test_loader(builtin: &str) -> Option<BoxLoader> {
  if builtin.starts_with(SIMPLE_DEMO_LOADER_IDENTIFIER) {
    return Some(Arc::new(DemoLoader));
  }
  None
}
