use std::{collections::HashMap, ops::Not, path::PathBuf};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext, contextify};
use rspack_error::Result;
use rspack_loader_runner::DisplayWithSuffix;
use rspack_paths::Utf8PathBuf;
use serde::Serialize;

use crate::ModuleHelper;

pub const STYLE_LOADER_IDENTIFIER: &str = "builtin:style-loader";

lazy_static::lazy_static! {
  static ref STYLE_LOADER_RUNTIME: Vec<(&'static str, &'static str)> = vec![
    (
      "injectStylesIntoLinkTag.js",
      include_str!("runtime/injectStylesIntoLinkTag.js"),
    ),
    (
      "injectStylesIntoStyleTag.js",
      include_str!("runtime/injectStylesIntoStyleTag.js"),
    ),
    (
      "insertStyleElement.js",
      include_str!("runtime/insertStyleElement.js"),
    ),
    (
      "insertBySelector.js",
      include_str!("runtime/insertBySelector.js"),
    ),
    (
      "setAttributesWithoutAttributes.js",
      include_str!("runtime/setAttributesWithoutAttributes.js"),
    ),
    (
      "setAttributesWithAttributes.js",
      include_str!("runtime/setAttributesWithAttributes.js"),
    ),
    (
      "setAttributesWithAttributesAndNonce.js",
      include_str!("runtime/setAttributesWithAttributesAndNonce.js"),
    ),
    (
      "setAttributesWithAttributesAndNonce.js",
      include_str!("runtime/setAttributesWithAttributesAndNonce.js"),
    ),
    (
      "styleTagTransform.js",
      include_str!("runtime/styleTagTransform.js"),
    ),
    ("styleDomAPI.js", include_str!("runtime/styleDomAPI.js")),
    (
      "singletonStyleDomAPI.js",
      include_str!("runtime/singletonStyleDomAPI.js"),
    ),
    ("isOldIE.js", include_str!("runtime/isOldIE.js")),
  ];
}

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct StyleLoaderOpts {
  pub base: Option<i64>,
  pub insert: Option<String>,
  pub output: String,
  pub style_tag_transform: Option<String>,
  pub attributes: Option<HashMap<String, String>>,
}

#[cacheable]
pub struct StyleLoader {
  options: StyleLoaderOpts,
  module_helper: ModuleHelper,
}

impl StyleLoader {
  pub fn write_runtime(dir: &Utf8PathBuf) -> Result<()> {
    if dir.exists().not() {
      std::fs::create_dir_all(dir)?;
    }
    for (file_name, runtime) in STYLE_LOADER_RUNTIME.iter() {
      let file = dir.join(file_name);
      if file.exists().not() {
        std::fs::write(file, runtime)?;
      }
    }

    Ok(())
  }
}

impl StyleLoader {
  pub fn new(options: StyleLoaderOpts) -> Self {
    let module_helper = ModuleHelper::new(&options.output);
    Self {
      options,
      module_helper,
    }
  }

  fn get_runtime_options_str(&self) -> String {
    let mut runtime_options = HashMap::new();
    if let Some(attributes) = &self.options.attributes {
      runtime_options.insert("attributes".to_string(), serde_json::json!(attributes));
    }
    if let Some(base) = &self.options.base {
      runtime_options.insert("base".to_string(), serde_json::json!(base));
    }
    let runtime_options = serde_json::to_string_pretty(&runtime_options).unwrap();
    runtime_options
  }

  fn get_insert_option(&self) -> String {
    match &self.options.insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        format!(r##"options.insert = insertFn;"##)
      }
      Some(insert) => {
        format!(r##"options.insert = insertFn.bind(null, "{insert}");"##)
      }
      None => format!(r##"options.insert = insertFn.bind(null, "head");"##),
    }
  }

  fn get_remaining_request(&self, loader_context: &mut LoaderContext<RunnerContext>) -> String {
    let resource = loader_context.resource();
    let request = loader_context.remaining_request();
    let request = request.display_with_suffix(resource);
    format!("!!{request}")
  }

  fn get_attributes_module(&self) -> String {
    match &self.options.attributes {
      Some(attributes) if attributes.contains_key("nonce") => self
        .module_helper
        .file_name("setAttributesWithAttributesAndNonce.js"),
      Some(_) => self
        .module_helper
        .file_name("setAttributesWithAttributes.js"),
      None => self
        .module_helper
        .file_name("setAttributesWithoutAttributes.js"),
    }
  }

  fn get_insert_fn_module(&self, loader_context: &mut LoaderContext<RunnerContext>) -> String {
    match &self.options.insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        let path = contextify(&loader_context.context.options.context, &insert);
        loader_context
          .build_dependencies
          .insert(PathBuf::from(&path));
        path
      }
      _ => self
        .module_helper
        .file_name_with_bang("insertBySelector.js"),
    }
  }

  fn get_style_tag_transform_fn_module(
    &self,
    loader_context: &mut LoaderContext<RunnerContext>,
  ) -> String {
    if let Some(style_tag_transform) = &self.options.style_tag_transform {
      loader_context
        .build_dependencies
        .insert(PathBuf::from(style_tag_transform));
      style_tag_transform.to_string()
    } else {
      self
        .module_helper
        .file_name_with_bang("styleTagTransform.js")
    }
  }

  fn get_hmr(&self, request: &str) -> String {
    format!(
      r##"
function isEqualLocals(a, b, isNamedExport) {{
  if ((!a && b) || (a && !b)) {{
    return false;
  }}

  let property;

  for (property in a) {{
    if (isNamedExport && property === "default") {{
      continue;
    }}

    if (a[property] !== b[property]) {{
      return false;
    }}
  }}

  for (property in b) {{
    if (isNamedExport && property === "default") {{
      continue;
    }}

    if (!a[property]) {{
      return false;
    }}
  }}

  return true;
}};

if (module.hot) {{
  if (!content.locals || module.hot.invalidate) {{

    var isNamedExport = !content.locals;
    var oldLocals = isNamedExport ? namedExport : content.locals;


    module.hot.accept(
      "{request}",
      function accept() {{
        if (!isEqualLocals(oldLocals, isNamedExport ? namedExport : content.locals, isNamedExport)) {{
          module.hot.invalidate();
          return;
        }}
        oldLocals = isNamedExport ? namedExport : content.locals;
        update(content);
      }}
    );
  }}

  module.hot.dispose(function dispose() {{
    update();
  }});
}}
  "##
    )
  }

  fn source(&self, loader_context: &mut LoaderContext<RunnerContext>) -> String {
    let request = self.get_remaining_request(loader_context);

    let runtime_options = self.get_runtime_options_str();

    let api_module = self
      .module_helper
      .file_name_with_bang("injectStylesIntoStyleTag.js");

    let dom_api_module = self.module_helper.file_name_with_bang("styleDomAPI.js");

    let insert_fn_module = self.get_insert_fn_module(loader_context);

    let attributes_module = self.get_attributes_module();

    let insert_style_element_module = self
      .module_helper
      .file_name_with_bang("insertStyleElement.js");

    let style_tag_transform_fn_module = self.get_style_tag_transform_fn_module(loader_context);

    let insert_option = self.get_insert_option();

    let hmr_code = self.get_hmr(&request);

    let source = format!(
      r##"
import API from "{api_module}";
import domAPI from "{dom_api_module}";
import insertFn from "{insert_fn_module}";
import setAttributes from "{attributes_module}";
import insertStyleElement from "{insert_style_element_module}";
import styleTagTransformFn from "{style_tag_transform_fn_module}";
import content, * as namedExport from "{request}";

var options = {runtime_options};

options.styleTagTransform = styleTagTransformFn;
options.setAttributes = setAttributes;
{insert_option}
options.domAPI = domAPI;
options.insertStyleElement = insertStyleElement;

var update = API(content, options);

{hmr_code}

export * from "{request}";
export default content && content.locals ? content.locals : undefined;"##
    );

    source
  }
}

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for StyleLoader {
  fn identifier(&self) -> Identifier {
    STYLE_LOADER_IDENTIFIER.into()
  }
  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source_map = loader_context.take_source_map();

    let source = self.source(loader_context);

    loader_context.finish_with((source, source_map));
    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source = loader_context.take_content();
    let sm = loader_context.take_source_map();

    loader_context.finish_with((source, sm));
    Ok(())
  }
}
