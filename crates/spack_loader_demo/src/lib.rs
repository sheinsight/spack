use std::sync::Arc;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  ApplyContext, BoxLoader, Context, Loader, LoaderContext, ModuleRuleUseLoader, ModuleType,
  NormalModuleFactoryResolveLoader, Plugin, Resolver, RunnerContext,
};
use rspack_error::Diagnostic;
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::{Identifiable, Identifier};
use rspack_cacheable::__private::rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[cacheable]
pub struct DemoLoader {
  pub inject_type: InjectType,
}
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for DemoLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_DEMO_LOADER_IDENTIFIER.into()
  }

  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    // 获取模块引用
    let module = unsafe { loader_context.context.module.as_ref() };
    // 获取模块类型
    let module_type = module.module_type();

    if module_type.is_css_like() {
      loader_context.emit_diagnostic(Diagnostic::warn(
        SIMPLE_DEMO_LOADER_IDENTIFIER.into(),
        "You can't use `experiments.css` (`experiments.futureDefaults` enable built-in CSS support by default) and `style-loader` together, please set `experiments.css` to `false` or set `{ type: \"javascript/auto\" }` for rules with `style-loader` in your webpack config (now `style-loader` does nothing).".to_string(),
      ));
      return Ok(());
    }

    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    let mut source = content.try_into_string()?;
    
    // 根据 inject_type 参数决定注入内容
    match self.inject_type {
      InjectType::StyleTag => {
        source += r#"
    function injectStyleTag(){
      console.error("injecting style tag");
    }
    "#;
      }
    }
    
    let sm = loader_context.take_source_map();
    println!("---> inject_type: {:?}, source: {}", self.inject_type, source);
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

#[derive(Debug, Clone, Archive, RkyvDeserialize, RkyvSerialize)]
pub enum InjectType {
  StyleTag,
}

#[derive(Debug)]
pub struct DemoLoaderPluginOpts {
  pub inject_type: InjectType,
}

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
    return Ok(get_builtin_test_loader(loader_request, self.options.inject_type.clone()));
  }
  Ok(None)
}

pub fn get_builtin_test_loader(builtin: &str, inject_type: InjectType) -> Option<BoxLoader> {
  if builtin.starts_with(SIMPLE_DEMO_LOADER_IDENTIFIER) {
    return Some(Arc::new(DemoLoader { inject_type }));
  }
  None
}
