use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  contextify, ApplyContext, BoxLoader, Context, Loader, LoaderContext, ModuleRuleUseLoader,
  ModuleType, NormalModuleFactoryResolveLoader, Plugin, Resolver, RunnerContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::{Identifiable, Identifier};

#[cacheable]
pub struct DemoLoader {
  options: DemoLoaderPluginOpts,
}

impl DemoLoader {
  fn write_inject_styles_into_style_tag(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/injectStylesIntoStyleTag.js");
    let dir = context.as_path().join(self.options.output.clone());
    let file = dir.join("injectStylesIntoStyleTag.js");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(&file, runtime_context).unwrap();
    Ok(file.to_string())
  }

  fn write_style_dom_api(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/styleDomAPI.js");
    let dir = context.as_path().join(self.options.output.clone());
    let file = dir.join("styleDomAPI.js");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(&file, runtime_context).unwrap();
    Ok(file.to_string())
  }

  fn write_insert_style_element(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/insertStyleElement.js");
    let dir = context.as_path().join(self.options.output.clone());
    let file = dir.join("insertStyleElement.js");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(&file, runtime_context).unwrap();
    Ok(file.to_string())
  }

  fn write_insert_by_selector(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/insertBySelector.js");
    let dir = context.as_path().join(self.options.output.clone());
    let file = dir.join("insertBySelector.js");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(&file, runtime_context).unwrap();
    Ok(file.to_string())
  }
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

    // 获取当前资源的绝对路径
    let resource_path = loader_context
      .resource_path()
      .map(|p| p.to_string())
      .unwrap_or_default();

    // 获取项目根目录 (context_path)
    let root_context = &loader_context.context.options.context;

    // 使用 contextify 将绝对路径转换为相对路径

    let abs = self.write_inject_styles_into_style_tag(&loader_context.context.options.context)?;

    let abs_dom = self.write_style_dom_api(&loader_context.context.options.context)?;

    let abs_insert = self.write_insert_style_element(&loader_context.context.options.context)?;

    let abs_insert_by_selector =
      self.write_insert_by_selector(&loader_context.context.options.context)?;

    let relative_path = contextify(root_context, &abs);
    let relative_path_insert = contextify(root_context, &abs_insert);
    let relative_path_dom = contextify(root_context, &abs_dom);
    let relative_path_insert_by_selector = contextify(root_context, &abs_insert_by_selector);
    let css_resource_path = contextify(root_context, &resource_path);

    println!(
      "---> css_resource_path: {:#?} , {:?}",
      css_resource_path, &resource_path
    );

    let insert_type = self.options.insert.clone();
    
    // 判断 insert 是否为绝对路径
    let insert_value = if PathBuf::from(&insert_type).is_absolute() {
      "module-path"
    } else {
      "selector"
    };

    let source = format!(
      r#"
      import API from "!{relative_path}";
      import domAPI from "!{relative_path_dom}";
      import insertStyleElement from "!{relative_path_insert}";
      import insertFn from "!{relative_path_insert_by_selector}";
      import content, * as namedExport from "!!{css_resource_path}";
    "#,
    );
    let sm = loader_context.take_source_map();
    println!("---> {}", source);
    loader_context.finish_with((source, sm));

    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    let mut source = content.try_into_string()?;
    source += r#"
   
    "#;
    let sm = loader_context.take_source_map();
    println!("---> {}", source);
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

#[derive(Debug, Clone)]
#[cacheable]
pub enum InjectType {
  StyleTag,
}

#[derive(Debug, Clone)]
#[cacheable]
pub struct DemoLoaderPluginOpts {
  pub inject_type: InjectType,
  pub es_module: bool,
  pub insert: String,
  pub output: String,
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
    return Ok(get_builtin_test_loader(
      loader_request,
      self.options.clone(),
    ));
  }
  Ok(None)
}

pub fn get_builtin_test_loader(builtin: &str, options: DemoLoaderPluginOpts) -> Option<BoxLoader> {
  if builtin.starts_with(SIMPLE_DEMO_LOADER_IDENTIFIER) {
    return Some(Arc::new(DemoLoader { options }));
  }
  None
}
