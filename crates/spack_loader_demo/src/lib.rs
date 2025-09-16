#![feature(let_chains)]

use std::ops::Not;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  contextify, ApplyContext, BoxLoader, Context, Loader, LoaderContext, ModuleFactoryCreateData,
  ModuleRuleUseLoader, ModuleType, NormalModuleFactoryBeforeResolve,
  NormalModuleFactoryResolveLoader, Plugin, Resolver, RunnerContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::{Identifiable, Identifier};
use rspack_paths::Utf8PathBuf;
use serde::Serialize;
use strum_macros::{Display, EnumString};

mod virtual_modules;
mod vp;

pub use vp::VirtualModulesPlugin;

use crate::vp::VirtualModulesPluginOptions;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeOptions {
  pub attributes: Option<HashMap<String, String>>,
  pub base: Option<i64>,
}

#[cacheable]
pub struct DemoLoader {
  options: DemoLoaderPluginOpts,
}

impl DemoLoader {
  fn write_runtime_file(
    &self,
    context: &Context,
    runtime_content: &str,
    filename: &str,
  ) -> Result<String> {
    let dir = context.as_path().join(self.options.output.clone());
    if dir.exists().not() {
      std::fs::create_dir_all(&dir).unwrap();
    }
    let file = dir.join(filename);
    if file.exists().not() {
      std::fs::write(&file, runtime_content).unwrap();
    }
    Ok(file.to_string())
  }

  fn write_inject_styles_into_style_tag(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/injectStylesIntoStyleTag.js");
    self.write_runtime_file(context, runtime_context, "injectStylesIntoStyleTag.js")
  }

  fn write_style_dom_api(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/styleDomAPI.js");
    self.write_runtime_file(context, runtime_context, "styleDomAPI.js")
  }

  fn write_insert_style_element(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/insertStyleElement.js");
    self.write_runtime_file(context, runtime_context, "insertStyleElement.js")
  }

  fn get_import_insert_by_selector_code(
    &self,
    loader_context: &mut LoaderContext<RunnerContext>,
  ) -> Result<String> {
    let context = &loader_context.context.options.context;
    let code = match &self.options.insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        let p = contextify(context, &insert);
        if self.options.es_module.unwrap_or(false) {
          format!(r##"import insertFn from "{p}""##)
        } else {
          format!(r##"var insertFn = require("{p}")"##)
        }
      }
      _ => {
        let runtime_context = include_str!("./runtimes/insertBySelector.js");
        let abs = self.write_runtime_file(context, runtime_context, "insertBySelector.js")?;
        let p = contextify(context, &abs);
        if self.options.es_module.unwrap_or(false) {
          format!(r##"import insertFn from "{p}""##)
        } else {
          format!(r##"var insertFn = require("{p}")"##)
        }
      }
    };
    Ok(code)
  }

  fn write_set_attributes_with_attributes_and_nonce(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/setAttributesWithAttributesAndNonce.js");
    self.write_runtime_file(
      context,
      runtime_context,
      "setAttributesWithAttributesAndNonce.js",
    )
  }

  fn get_import_link_api_code(
    &self,
    loader_context: &mut LoaderContext<RunnerContext>,
  ) -> Result<String> {
    // let context = &loader_context.context.options.context;
    // let runtime_context = include_str!("./runtimes/injectStylesIntoLinkTag.js");
    // let abs = self.write_runtime_file(context, runtime_context, "injectStylesIntoLinkTag.js")?;
    // let p = contextify(context, &abs);
    let code = if self.options.es_module.unwrap_or(false) {
      format!(r##"import API from "virtualModules:injectStylesIntoLinkTag.js""##)
    } else {
      format!(r##"var API = require("virtualModules:injectStylesIntoLinkTag.js")"##)
    };
    Ok(code)
  }

  fn get_insert_option_code(
    &self,
    loader_context: &mut LoaderContext<RunnerContext>,
  ) -> Result<String> {
    let context = &loader_context.context.options.context;
    let code = match &self.options.insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        "options.insert = insertFn;".to_string()
      }
      Some(insert) => {
        format!(r##"options.insert = insertFn.bind(null, "{insert}");"##)
      }
      _ => {
        format!(r##"options.insert = insertFn.bind(null, "head");"##)
      }
    };
    Ok(code)
  }

  fn get_import_link_content_code(
    &self,
    loader_context: &mut LoaderContext<RunnerContext>,
  ) -> Result<String> {
    let context = &loader_context.context.options.context;
    let query = loader_context.resource_query().unwrap_or_default();
    let module_path = contextify(context, &format!("!!{query}"));
    let code = if self.options.es_module.unwrap_or(false) {
      format!(r##"import content from "{module_path}""##)
    } else {
      format!(r##"var content = require("{module_path}")"##)
    };
    Ok(code)
  }

  fn write_set_attributes_with_attributes(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/setAttributesWithAttributes.js");
    self.write_runtime_file(context, runtime_context, "setAttributesWithAttributes.js")
  }

  fn write_set_attributes_without_attributes(&self, context: &Context) -> Result<String> {
    let runtime_context = include_str!("./runtimes/setAttributesWithoutAttributes.js");
    self.write_runtime_file(
      context,
      runtime_context,
      "setAttributesWithoutAttributes.js",
    )
  }

  fn get_link_hmr_code(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<String> {
    if !loader_context.hot {
      return Ok(String::new());
    }

    let context = &loader_context.context.options.context;

    let update_code = if self.options.es_module.unwrap_or(false) {
      "update(content);"
    } else {
      r##"
content = require(modulePath);
content = content.__esModule ? content.default : content;
update(content);
"##
    };

    if let Some(query) = loader_context.resource_query() {
      let module_path = contextify(context, &format!("!!{query}"));
      let code = format!(
        r##"
if (module.hot) {{
  module.hot.accept(
      {module_path},
      function() {{
      {update_code}
      }}
  );
  module.hot.dispose(function() {{
    update();
  }});
}}"##
      );
      return Ok(code);
    }
    Ok(String::new())
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

    println!(
      "---> module: {:#?} {:#?} {:#?}",
      loader_context.resource_query(),
      loader_context.resource_path(),
      loader_context.resource()
    );

    let es_module = self.options.es_module.unwrap_or(false);
    let inject_type = self.options.inject_type.unwrap_or(InjectType::StyleTag);
    let mut runtimeOptions = RuntimeOptions {
      attributes: self.options.attributes.clone(),
      base: self.options.base,
    };

    let insert_type = match &self.options.insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => "module-path",
      _ => "selector",
    };

    // let context = &loader_context.context.options.context;

    let source = match inject_type {
      InjectType::LinkTag => {
        let hmr_code = self.get_link_hmr_code(loader_context)?;

        let api_code = self.get_import_link_api_code(loader_context)?;
        let insert_code = self.get_import_insert_by_selector_code(loader_context)?;
        let link_content_code = self.get_import_link_content_code(loader_context)?;

        let content = if self.options.es_module.unwrap_or(false) {
          format!(r##""##)
        } else {
          format!(r##"content = content.__esModule ? content.default : content;"##)
        };

        let runtime_code = serde_json::to_string_pretty(&runtimeOptions).unwrap();

        let insert_option_code = self.get_insert_option_code(loader_context)?;

        let export_code = if self.options.es_module.unwrap_or(false) {
          format!(r##"export default {{}}"##)
        } else {
          format!(r##""##)
        };

        format!(
          r#"
{api_code}
{insert_code}
{link_content_code}
{content}

var options = {runtime_code}

{insert_option_code}

var update = API(content, options);

{hmr_code}
{export_code}
        "#
        )
      }
      InjectType::StyleTag => todo!(),
      InjectType::SingletonStyleTag => todo!(),
      InjectType::AutoStyleTag => todo!(),
      InjectType::LazyStyleTag => todo!(),
      InjectType::LazySingletonStyleTag => todo!(),
      InjectType::LazyAutoStyleTag => todo!(),
    };

    let sm = loader_context.take_source_map();

    println!("---> {}", source);

    loader_context.finish_with((source, sm));

    ////////

    //     let resource_path = loader_context
    //       .resource_path()
    //       .map(|p| p.to_string())
    //       .unwrap_or_default();

    //     let root_context = &loader_context.context.options.context;

    //     let abs = self.write_inject_styles_into_style_tag(&loader_context.context.options.context)?;

    //     let abs_dom = self.write_style_dom_api(&loader_context.context.options.context)?;

    //     let abs_insert = self.write_insert_style_element(&loader_context.context.options.context)?;

    //     let abs_insert_by_selector =
    //       self.write_insert_by_selector(&loader_context.context.options.context)?;

    //     let relative_path = contextify(&resource_path, &abs);
    //     let relative_path_insert = contextify(&resource_path, &abs_insert);
    //     let relative_path_dom = contextify(&resource_path, &abs_dom);
    //     let relative_path_insert_by_selector = contextify(&resource_path, &abs_insert_by_selector);

    //     let css_resource_path = contextify(&resource_path, &resource_path);

    //     println!(
    //       "---> css_resource_path: {:#?} , {:?}",
    //       css_resource_path, &resource_path
    //     );

    //     // 判断 insert 是否为绝对路径
    //     let selector_fn_module = if PathBuf::from(&self.options.insert).is_absolute() {
    //       let module_path = contextify(root_context, &self.options.insert);
    //       loader_context
    //         .build_dependencies
    //         .insert(module_path.clone().into());
    //       module_path
    //     } else {
    //       format!(r##"!{relative_path_insert_by_selector}"##)
    //     };

    //     let attributes_fn = match &self.options.attributes {
    //       Some(attributes) if attributes.contains_key("nonce") => {
    //         Self::write_set_attributes_with_attributes_and_nonce
    //       }
    //       Some(_) => Self::write_set_attributes_with_attributes,
    //       None => Self::write_set_attributes_without_attributes,
    //     };

    //     let attributes = attributes_fn(self, &loader_context.context.options.context)?;

    //     let opts = serde_json::to_string_pretty(&runtimeOptions).unwrap();

    //     let source = format!(
    //       r#"
    // import API from "!{relative_path}";
    // import domAPI from "!{relative_path_dom}";
    // import insertStyleElement from "!{relative_path_insert}";
    // import insertFn from "{selector_fn_module}";
    // import setAttributes from "{attributes}";
    // import content, * as namedExport from "!!{css_resource_path}";

    // var options = {opts};

    // options.setAttributes = setAttributes;
    // options.insertStyleElement = insertStyleElement;
    // var update = API(content, options);
    //     "#,
    //     );
    // let sm = loader_context.take_source_map();
    // println!("---> {}", source);
    // loader_context.finish_with((source, sm));

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

#[derive(Debug, Clone, Copy, Serialize, Display, EnumString)]
#[cacheable]
#[strum(serialize_all = "camelCase")]
pub enum InjectType {
  StyleTag,
  SingletonStyleTag,
  AutoStyleTag,
  LazyStyleTag,
  LazySingletonStyleTag,
  LazyAutoStyleTag,
  LinkTag,
}

#[derive(Debug, Clone, Serialize)]
#[cacheable]
pub struct DemoLoaderPluginOpts {
  pub base: Option<i64>,
  pub inject_type: Option<InjectType>,
  pub es_module: Option<bool>,
  pub insert: Option<String>,
  pub output: String,
  pub attributes: Option<HashMap<String, String>>,
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
    let mut modules = HashMap::new();
    modules.insert(
      "virtualModules:injectStylesIntoLinkTag.js".to_string(),
      include_str!("./runtimes/injectStylesIntoLinkTag.js"),
    );
    modules.insert(
      "virtualModules:injectStylesIntoStyleTag.js".to_string(),
      include_str!("./runtimes/injectStylesIntoStyleTag.js"),
    );
    modules.insert(
      "virtualModules:insertStyleElement.js".to_string(),
      include_str!("./runtimes/insertStyleElement.js"),
    );
    modules.insert(
      "virtualModules:insertBySelector.js".to_string(),
      include_str!("./runtimes/insertBySelector.js"),
    );
    modules.insert(
      "virtualModules:setAttributesWithAttributes.js".to_string(),
      include_str!("./runtimes/setAttributesWithAttributes.js"),
    );
    modules.insert(
      "virtualModules:setAttributesWithAttributesAndNonce.js".to_string(),
      include_str!("./runtimes/setAttributesWithAttributesAndNonce.js"),
    );
    modules.insert(
      "virtualModules:setAttributesWithoutAttributes.js".to_string(),
      include_str!("./runtimes/setAttributesWithoutAttributes.js"),
    );

    let v = VirtualModulesPluginOptions {
      modules: modules
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect(),
    };

    let v = VirtualModulesPlugin::new(v);

    v.apply(ctx)?;

    ctx
      .normal_module_factory_hooks
      .before_resolve
      .tap(before_resolve::new(self));

    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));

    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryBeforeResolve for DemoLoaderPlugin)]
async fn before_resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<bool>> {
  if data.request.starts_with("virtualModules:") {
    // 添加前导斜杠以匹配存储的路径
    let virtual_path = format!("/{}", data.request);
    println!(
      "Resolving virtual module: {} -> {}",
      data.request, virtual_path
    );
    data.request = virtual_path;
    return Ok(Some(true));
  }
  Ok(None)
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
