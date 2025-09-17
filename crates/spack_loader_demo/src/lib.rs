#![feature(let_chains)]

use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  contextify, ApplyContext, BoxLoader, Context, Loader, LoaderContext, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, Plugin, Resolver, RunnerContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::{Identifiable, Identifier};
use serde::Serialize;
use strum_macros::{Display, EnumString};

mod runtime_module;
mod virtual_modules;
mod vp;

pub use vp::VirtualModulesPlugin;

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
        if self.options.es_module.unwrap_or(false) {
          format!(r##"import insertFn from "virtualModules:insertBySelector.js""##)
        } else {
          format!(r##"var insertFn = require("virtualModules:insertBySelector.js")"##)
        }
      }
    };
    Ok(code)
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
    let query = loader_context
      .resource_path()
      .map(|p| p.to_string())
      .unwrap_or_default();
    let module_path = contextify(context, &format!("!!{query}"));
    let code = if self.options.es_module.unwrap_or(false) {
      format!(r##"import content from "{module_path}""##)
    } else {
      format!(r##"var content = require("{module_path}")"##)
    };
    Ok(code)
  }

  fn get_link_hmr_code(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<String> {
    if !loader_context.hot {
      return Ok(String::new());
    }

    let update_code = if self.options.es_module.unwrap_or(false) {
      "update(content);"
    } else {
      r##"
content = require(modulePath);
content = content.__esModule ? content.default : content;
update(content);
"##
    };

    let module_path = self.get_resource_query(loader_context)?;

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

  fn get_resource_query(&self, loader_context: &LoaderContext<RunnerContext>) -> Result<String> {
    if let Some(query) = loader_context.resource_query() {
      return Ok(format!("!!{query}"));
    }
    Ok("".to_string())
  }

  fn get_style_hmr_code(&self, loader_context: &LoaderContext<RunnerContext>) -> Result<String> {
    if !loader_context.hot {
      return Ok(String::new());
    }

    let es_module = self.options.es_module.unwrap_or(false);

    let module_path = self.get_resource_query(loader_context)?;

    let code = format!(
      r##"
if (module.hot) {{
if (!content.locals || module.hot.invalidate) {{
  var isEqualLocals = {is_equal_locals};
  var isNamedExport = {is_named_export};
  var oldLocals = isNamedExport ? namedExport : content.locals;

  module.hot.accept(
    {module_path},
    function () {{
      {inner_code}
    }}
  );
}}

module.hot.dispose(function() {{
  if (update) {{
    update();
  }}
}})
}}
"##,
      module_path = module_path,
      is_equal_locals = r##"
function isEqualLocals(a, b, isNamedExport) {
if ((!a && b) || (a && !b)) {
  return false;
}

let property;

for (property in a) {
  if (isNamedExport && property === "default") {
    continue;
  }

  if (a[property] !== b[property]) {
    return false;
  }
}

for (property in b) {
  if (isNamedExport && property === "default") {
    continue;
  }

  if (!a[property]) {
    return false;
  }
}

return true;
}"##,
      is_named_export = if es_module {
        "!content.locals"
      } else {
        "false"
      },
      inner_code = if es_module {
        format!(
          r##"
if (!isEqualLocals(oldLocals, isNamedExport ? namedExport : content.locals, isNamedExport)) {{
  module.hot.invalidate();
  return;
}}
oldLocals = isNamedExport ? namedExport : content.locals;
if (update && refs > 0) {{
  update(content);
}}
        "##
        )
      } else {
        format!(
          r##"
content = require({module_path});
content = content.__esModule ? content.default : content;
if (!isEqualLocals(oldLocals, content.locals)) {{
  module.hot.invalidate();
  return;
}}
oldLocals = content.locals;
if (update && refs > 0) {{
  update(content);
}}
        "##
        )
      }
    );
    return Ok(code);
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

    let context = &loader_context.context.options.context;

    let module_path = self.get_resource_query(loader_context)?;

    let runtime_code = serde_json::to_string_pretty(&runtimeOptions).unwrap();

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

        let insert_option_code = self.get_insert_option_code(loader_context)?;

        let export_code = if es_module {
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

      InjectType::LazyStyleTag
      | InjectType::LazySingletonStyleTag
      | InjectType::LazyAutoStyleTag => {
        let is_singleton = match inject_type {
          InjectType::LazySingletonStyleTag => true,
          _ => false,
        };

        let is_auto = match inject_type {
          InjectType::LazyAutoStyleTag => true,
          _ => false,
        };

        let hmr_code = self.get_style_hmr_code(loader_context)?;

        let api_code = if es_module {
          format!(r##"import API from "virtualModules:injectStylesIntoStyleTag.js""##)
        } else {
          format!(r##"var API = require("virtualModules:injectStylesIntoStyleTag.js")"##)
        };

        let dom_api_code = match (is_auto, es_module, is_singleton) {
          (true, true, _) => format!(
            r##"
import domAPI from "virtualModules:styleDomAPI.js";
import domAPISingleton from "virtualModules:singletonStyleDomAPI.js";
"##
          ),
          (true, false, _) => format!(
            r##"
var domAPI = require("virtualModules:styleDomAPI.js");
var domAPISingleton = require("virtualModules:singletonStyleDomAPI.js");
"##
          ),
          (false, true, true) => {
            format!(r##"import domAPI from "virtualModules:singletonStyleDomAPI.js";"##)
          }
          (false, true, false) => {
            format!(r##"import domAPI from "virtualModules:styleDomAPI.js";"##)
          }
          (false, false, true) => {
            format!(r##"var domAPI = require("virtualModules:singletonStyleDomAPI.js");"##)
          }
          (false, false, false) => {
            format!(r##"var domAPI = require("virtualModules:styleDomAPI.js");"##)
          }
        };

        let insert_fn_code = match &self.options.insert {
          Some(insert) if PathBuf::from(insert).is_absolute() => {
            let insert_path = contextify(context, &insert);
            loader_context
              .build_dependencies
              .insert(insert_path.clone().into());
            format!(r##"import insertFn from "{insert_path}""##)
          }
          _ => {
            format!(r##"import insertFn from "virtualModules:insertStyleElement.js""##)
          }
        };

        let attributes_module = match (&self.options.attributes) {
          Some(attributes) if attributes.contains_key("nonce") => {
            "virtualModules:setAttributesWithAttributesAndNonce.js"
          }
          Some(_) => "virtualModules:setAttributesWithAttributes.js",
          None => "virtualModules:setAttributesWithoutAttributes.js",
        };

        let set_attributes_code = if es_module {
          format!(r##"import setAttributes from "{attributes_module}""##)
        } else {
          format!(r##"var setAttributes = require("{attributes_module}")"##)
        };

        let insert_style_element_code = if es_module {
          format!(r##"import insertStyleElement from "virtualModules:insertStyleElement.js""##)
        } else {
          format!(r##"var insertStyleElement = require("virtualModules:insertStyleElement.js")"##)
        };

        // let style_tag_transform_fn_code = match (is_singleton) {};

        // let content_code = if es_module {
        //   format!(r##"import content from "virtualModules:injectStylesIntoStyleTag.js""##)
        // } else {
        //   format!(r##"var content = require("virtualModules:injectStylesIntoStyleTag.js")"##)
        // };

        let content_code = if es_module {
          format!(r##"import content from "{module_path}""##)
        } else {
          format!(r##"var content = require("{module_path}")"##)
        };

        let is_old_ie_code = match (is_auto, es_module) {
          (true, true) => format!(r##"import isOldIE from "virtualModules:isOldIE.js""##),
          (true, false) => format!(r##"var isOldIE = require("virtualModules:isOldIE.js")"##),
          _ => "".to_string(),
        };

        let locals = if es_module {
          format!(
            r##"
if (content && content.locals) {{
  exported.locals = content.locals;
}}"##
          )
        } else {
          format!(
            r##"
content = content.__esModule ? content.default : content;
exported.locals = content.locals || {{}};
"##
          )
        };

        let transform_fn = if is_singleton {
          format!(r##""##)
        } else {
          format!(r##"options.styleTagTransform = styleTagTransformFn"##)
        };

        format!(
          r##"
var exported = {{}};

{api_code}
{dom_api_code}
{insert_fn_code}
{set_attributes_code}
{insert_style_element_code}
{content_code}
{is_old_ie_code}
{locals}

var refs = 0;
var update;

var options = {runtime_code};

{transform_fn}
options.setAttributes = setAttributes;

{hmr_code}"##
        )
      }

      InjectType::StyleTag | InjectType::SingletonStyleTag | InjectType::AutoStyleTag => {
        format!(r##""##)
      }
    };

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
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));

    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {}
}

// impl Default for DemoLoaderPlugin {
//   fn default() -> Self {
//     Self::with_default(Identifier::from("webpack/runtime/css_loading"), None)
//   }
// }

// #[plugin_hook(CompilationRuntimeRequirementInTree for DemoLoaderPlugin)]
// async fn runtime_requirements_in_tree(
//   &self,
//   compilation: &mut Compilation,
//   chunk_ukey: &ChunkUkey,
//   _all_runtime_requirements: &RuntimeGlobals,
//   runtime_requirements: &RuntimeGlobals,
//   runtime_requirements_mut: &mut RuntimeGlobals,
// ) -> Result<Option<()>> {
//   compilation.add_runtime_module(chunk_ukey, Box::<StyleLoaderRuntimeModule>::default())?;

//   Ok(None)
// }

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
