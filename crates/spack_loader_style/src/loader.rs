use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext, contextify};
use rspack_error::Result;
use rspack_loader_runner::DisplayWithSuffix;
use serde::Serialize;
use strum_macros::{Display, EnumString};

use crate::code_template::CodeTemplate;

// use crate::template;

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct StyleLoaderOpts {
  pub base: Option<i64>,
  pub inject_type: Option<InjectType>,
  pub es_module: Option<bool>,
  pub insert: Option<String>,
  pub output: String,
  pub attributes: Option<HashMap<String, String>>,
}

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

impl InjectType {
  pub fn get_link_hmr_code(&self, request: &str, es_module: bool) -> String {
    let content = if es_module {
      format!(
        r##"
update(content);
"##
      )
    } else {
      format!(
        r##"
content = require("{request}");
content = content.__esModule ? content.default : content;
update(content);
"##
      )
    };

    return format!(
      r##"
if (module.hot) {{
  module.hot.accept(
  "{request}",
  function(){{
    {content}
  }}
  )
  module.hot.dispose(function() {{
    update();
  }});
}}"##
    );
  }

  pub fn get_import_insert_by_selector_code(
    &self,
    loader_context: &mut LoaderContext<RunnerContext>,
    es_module: bool,
    insert: &Option<String>,
  ) -> String {
    let context = &loader_context.context.options.context;
    match &insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        let path = contextify(context, &insert);
        loader_context
          .build_dependencies
          .insert(PathBuf::from(insert));
        if es_module {
          format!(r##"import insertFn from "{path}";"##)
        } else {
          format!(r##"var insertFn = require("{path}");"##)
        }
      }
      Some(_) => {
        if es_module {
          format!(r##"import insertFn from "!@/.lego/runtime/insertBySelector.js";"##)
        } else {
          format!(r##"var insertFn = require("!@/.lego/runtime/insertBySelector.js");"##)
        }
      }
      None => "".to_string(),
    }
  }

  pub fn get_import_link_content_code(&self, request: &str, es_module: bool) -> String {
    if es_module {
      format!(r##"import content from "!!{request}";"##)
    } else {
      format!(
        r##"
var content = require("!!{request}");
content = content.__esModule ? content.default : content;"##
      )
    }
  }

  pub fn get_insert_option_code(&self, insert: &Option<String>) -> String {
    match &insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        format!(r##"options.insert = insertFn;"##)
      }
      Some(insert) => {
        format!(r##"options.insert = insertFn.bind(null, "{insert}");"##)
      }
      None => format!(r##"options.insert = insertFn.bind(null, "head");"##),
    }
  }

  pub fn get_import_style_api_code(&self, es_module: bool) -> String {
    if es_module {
      format!(r##"import API from "!@/.lego/runtime/injectStylesIntoStyleTag.js";"##)
    } else {
      format!(r##"var API = require("!@/.lego/runtime/injectStylesIntoStyleTag.js");"##)
    }
  }

  pub fn get_export_lazy_style_code(&self, es_module: bool, request: &str) -> String {
    if es_module {
      format!(
        r##"
export * from "{request}";
export default exported;
"##
      )
    } else {
      format!(
        r##"
module.exports = exported;
"##
      )
    }
  }

  pub fn get_export_style_code(&self, es_module: bool, request: &str) -> String {
    if es_module {
      format!(
        r##"
      export * from "{request}";
      export default content && content.locals ? content.locals : undefined;"##
      )
    } else {
      format!(r##"module.exports = content && content.locals || {{}};"##)
    }
  }

  pub fn get_style_hmr_code(&self, es_module: bool, request: &str, lazy: bool) -> String {
    let is_named_export = if es_module {
      "!content.locals"
    } else {
      "false"
    };

    let dispose_content = if lazy {
      format!(
        r##"
      if (update) {{
        update();
      }}"##
      )
    } else {
      format!(r##"update();"##)
    };

    let hmr_code = match (es_module, lazy) {
      (true, true) => format!(
        r##"
        if (!isEqualLocals(oldLocals, isNamedExport ? namedExport : content.locals, isNamedExport)) {{
          module.hot.invalidate();
          return;
        }}
        oldLocals = isNamedExport ? namedExport : content.locals;
        if (update && refs > 0) {{
          update(content);
        }}"##
      ),
      (true, false) => format!(
        r##"
        if (!isEqualLocals(oldLocals, isNamedExport ? namedExport : content.locals, isNamedExport)) {{
          module.hot.invalidate();
          return;
        }}
        oldLocals = isNamedExport ? namedExport : content.locals;
        update(content);"##
      ),
      (false, true) => format!(
        r##"
        content = require("{request}");
        content = content.__esModule ? content.default : content;
        if (!isEqualLocals(oldLocals, content.locals)) {{
            module.hot.invalidate();
            return;
        }}
        oldLocals = content.locals;
        if (update && refs > 0) {{
          update(content);
        }}"##
      ),
      (false, false) => format!(
        r##"
        content = require("{request}");
        content = content.__esModule ? content.default : content;
        if (typeof content === 'string') {{
          content = [[module.id, content, '']];
        }}
        if (!isEqualLocals(oldLocals, content.locals)) {{
            module.hot.invalidate();
            return;
        }}
        oldLocals = content.locals;
        update(content);
      "##
      ),
    };

    format!(
      r##"
if (module.hot) {{
    if (!content.locals || module.hot.invalidate) {{
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

      var isNamedExport = {is_named_export};
      var oldLocals = isNamedExport ? namedExport : content.locals;


      module.hot.accept(
        "{request}",
        function() {{
          {hmr_code}
        }}
      );
    }}

    module.hot.dispose(function() {{
      {dispose_content}
    }});
}}
"##
    )
  }

  pub fn get_import_insert_style_element_code(&self, es_module: bool) -> String {
    if es_module {
      format!(r##"import insertStyleElement from "!@/.lego/runtime/insertStyleElement.js";"##)
    } else {
      format!(r##"var insertStyleElement = require("!@/.lego/runtime/insertStyleElement.js");"##)
    }
  }

  // TODO
  pub fn get_style_tag_transform_fn_code(&self) -> String {
    format!("")
  }

  pub fn get_import_is_old_ie_code(&self, es_module: bool, is_auto: bool) -> String {
    if is_auto {
      if es_module {
        format!(r##"import isOldIE from "!@/.lego/runtime/isOldIE.js";"##)
      } else {
        format!(r##"var isOldIE = require("!@/.lego/runtime/isOldIE.js");"##)
      }
    } else {
      format!("")
    }
  }

  pub fn get_import_style_content_code(&self, request: &str, es_module: bool) -> String {
    if es_module {
      format!(r##"import content, * as namedExport from "!!{request}";"##)
    } else {
      format!(r##"var content = require("!!{request}");"##)
    }
  }

  pub fn get_dom_api(&self, is_auto: bool) -> String {
    if is_auto {
      format!("isOldIE() ? domAPISingleton : domAPI;")
    } else {
      format!("domAPI;")
    }
  }

  pub fn get_style_tag_transform_fn(&self, is_singleton: bool) -> String {
    if is_singleton {
      format!("")
    } else {
      format!("options.styleTagTransform = styleTagTransformFn;")
    }
  }

  pub fn get_set_attributes_code(&self, loader_options: &StyleLoaderOpts) -> String {
    let es_module = loader_options.es_module.unwrap_or(false);
    let modules = match &loader_options.attributes {
      Some(attributes) if attributes.contains_key("nonce") => {
        format!(r##"!@/.lego/runtime/setAttributesWithAttributesAndNonce"##)
      }
      Some(_) => {
        format!(r##"!@/.lego/runtime/setAttributesWithAttributes"##)
      }
      None => format!(r##"!@/.lego/runtime/setAttributesWithoutAttributes"##),
    };

    if es_module {
      format!(r##"import setAttributes from "{modules}";"##)
    } else {
      format!(r##"var setAttributes = require("{modules}");"##)
    }
  }
}

impl InjectType {
  pub fn get_link_tag_code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
    runtime_options: &str,
  ) -> String {
    let es_module = loader_options.es_module.unwrap_or(false);

    let import_link_api_code = CodeTemplate::new(
      r#"import API from "@/.lego/runtime/injectStylesIntoLinkTag.js";"#,
      r#"var API = require("@/.lego/runtime/injectStylesIntoLinkTag.js");"#,
    )
    .of_es_module(es_module);

    let hmr_code = self.get_link_hmr_code(&request, es_module);
    let import_insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, es_module, &loader_options.insert);
    let import_link_content_code = self.get_import_link_content_code(&request, es_module);
    let insert_option_code = self.get_insert_option_code(&loader_options.insert);
    let export_code = if es_module {
      format!(r##"export default {{}};"##)
    } else {
      format!(r##""##)
    };

    let source = format!(
      r##"
      {import_link_api_code}
      {import_insert_by_selector_code}
      {import_link_content_code}
      
      var options = {runtime_options};
      {insert_option_code}
      var update = API(content, options);
      
      {hmr_code}
      {export_code}
"##
    );
    source
  }

  pub fn get_import_style_dom_api_code(
    &self,
    es_module: bool,
    is_auto: bool,
    is_singleton: bool,
  ) -> String {
    if is_auto {
      if es_module {
        return format!(
          r##"
        import domAPI from "!@/.lego/runtime/styleDomAPI.js";
        import domAPISingleton from "!@/.lego/runtime/singletonStyleDomAPI.js";"##
        );
      } else {
        return format!(
          r##"
        var domAPI = require("!@/.lego/runtime/styleDomAPI.js");
        var domAPISingleton = require("!@/.lego/runtime/singletonStyleDomAPI.js");"##
        );
      }
    }

    if es_module {
      if is_singleton {
        return format!(r##"import domAPI from "!@/.lego/runtime/singletonStyleDomAPI.js";"##);
      } else {
        return format!(r##"import domAPI from "!@/.lego/runtime/styleDomAPI.js";"##);
      }
    } else {
      if is_singleton {
        return format!(r##"var domAPI = require("!@/.lego/runtime/singletonStyleDomAPI.js");"##);
      } else {
        return format!(r##"var domAPI = require("!@/.lego/runtime/styleDomAPI.js");"##);
      }
    }
  }

  pub fn get_lazy_style_tag_code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
    runtime_options: &str,
    is_singleton: bool,
    is_auto: bool,
  ) -> String {
    let es_module = loader_options.es_module.unwrap_or(false);

    let style_api_code = self.get_import_style_api_code(es_module);
    let style_dom_api_code = self.get_import_style_dom_api_code(es_module, is_auto, is_singleton);
    let insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, es_module, &loader_options.insert);
    let set_attributes_code = self.get_set_attributes_code(&loader_options);
    let insert_style_element_code = self.get_import_insert_style_element_code(es_module);
    let style_tag_transform_fn_code = self.get_style_tag_transform_fn_code();
    let import_style_content_code = self.get_import_style_content_code(&request, es_module);
    let insert_option_code = self.get_insert_option_code(&loader_options.insert);

    let is_old_ie_code = self.get_import_is_old_ie_code(es_module, is_auto);

    let exported = CodeTemplate::new(
      r##"
      if (content && content.locals) {{
        exported.locals = content.locals;
      }}"##,
      r##"
      content = content.__esModule ? content.default : content;
      exported.locals = content.locals || {{}};"##,
    )
    .of_es_module(es_module);

    let style_tag_transform_fn = self.get_style_tag_transform_fn(is_singleton);

    let dom_api = self.get_dom_api(is_auto);

    let hmr_code = self.get_style_hmr_code(es_module, &request, true);

    let export_code = self.get_export_lazy_style_code(es_module, &request);

    let source = format!(
      r##"
var exported = {{}};

{style_api_code}
{style_dom_api_code}
{insert_by_selector_code}
{set_attributes_code}
{insert_style_element_code}
{style_tag_transform_fn_code}
{import_style_content_code}
{is_old_ie_code}
{exported}

var refs = 0;
var update;
var options = {runtime_options};

{style_tag_transform_fn}
options.setAttributes = setAttributes;
{insert_option_code}
options.domAPI = {dom_api}
options.insertStyleElement = insertStyleElement;

exported.use = function(insertOptions) {{
  options.options = insertOptions || {{}};

  if (!(refs++)) {{
    update = API(content, options);
  }}

  return exported;
}};

exported.unuse = function() {{
  if (refs > 0 && !--refs) {{
    update();
    update = null;
  }}
}};

{hmr_code}

{export_code}
"##
    );

    println!("source: {}", source.clone());

    source
  }

  pub fn get_style_tag_code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
    runtime_options: &str,
    is_singleton: bool,
    is_auto: bool,
  ) -> String {
    let es_module = loader_options.es_module.unwrap_or(false);

    let style_api_code = self.get_import_style_api_code(es_module);
    let style_dom_api_code = self.get_import_style_dom_api_code(es_module, is_auto, is_singleton);
    let insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, es_module, &loader_options.insert);
    let set_attributes_code = self.get_set_attributes_code(&loader_options);
    let insert_style_element_code = self.get_import_insert_style_element_code(es_module);
    let style_tag_transform_fn_code = self.get_style_tag_transform_fn_code();
    let import_style_content_code = self.get_import_style_content_code(&request, es_module);
    let insert_option_code = self.get_insert_option_code(&loader_options.insert);

    let is_old_ie_code = self.get_import_is_old_ie_code(es_module, is_auto);

    let exported = CodeTemplate::new(
      r##""##,
      r##"content = content.__esModule ? content.default : content;"##,
    )
    .of_es_module(es_module);

    let style_tag_transform_fn = self.get_style_tag_transform_fn(is_singleton);

    let dom_api = self.get_dom_api(is_auto);

    let hmr_code = self.get_style_hmr_code(es_module, &request, false);

    let export_code = self.get_export_style_code(es_module, &request);

    let source = format!(
      r##"
      {style_api_code}
      {style_dom_api_code}
      {insert_by_selector_code}
      {set_attributes_code}
      {insert_style_element_code}
      {style_tag_transform_fn_code}
      {import_style_content_code}
      {is_old_ie_code}
      {exported}

      var options = {runtime_options};

      {style_tag_transform_fn}
      options.setAttributes = setAttributes;
      {insert_option_code}
      options.domAPI = {dom_api};
      options.insertStyleElement = insertStyleElement;

      var update = API(content, options);

      {hmr_code}

      {export_code}
      "##
    );

    source
  }
}

#[cacheable]
pub struct StyleLoader {
  pub options: StyleLoaderOpts,
}

pub const STYLE_LOADER_IDENTIFIER: &str = "builtin:style-loader";

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for StyleLoader {
  fn identifier(&self) -> Identifier {
    STYLE_LOADER_IDENTIFIER.into()
  }
  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source_map = loader_context.take_source_map();

    let resource = loader_context.resource();

    let request = loader_context.remaining_request();

    let request = request.display_with_suffix(resource);

    let inject_type = self.options.inject_type.unwrap_or(InjectType::StyleTag);

    let mut runtime_options = HashMap::new();
    if let Some(attributes) = &self.options.attributes {
      runtime_options.insert("attributes".to_string(), serde_json::json!(attributes));
    }

    if let Some(base) = &self.options.base {
      runtime_options.insert("base".to_string(), serde_json::json!(base));
    }
    let runtime_options = serde_json::to_string_pretty(&runtime_options).unwrap();

    let is_lazy_singleton = matches!(inject_type, InjectType::LazySingletonStyleTag);

    let is_lazy_auto = matches!(inject_type, InjectType::LazyAutoStyleTag);

    let is_singleton = matches!(inject_type, InjectType::SingletonStyleTag);

    let is_auto = matches!(inject_type, InjectType::AutoStyleTag);

    match inject_type {
      InjectType::LinkTag => {
        let source =
          inject_type.get_link_tag_code(&request, loader_context, &self.options, &runtime_options);
        loader_context.finish_with((source, source_map));
      }
      InjectType::LazyStyleTag
      | InjectType::LazySingletonStyleTag
      | InjectType::LazyAutoStyleTag => {
        let source = inject_type.get_lazy_style_tag_code(
          &request,
          loader_context,
          &self.options,
          &runtime_options,
          is_lazy_singleton,
          is_lazy_auto,
        );
        loader_context.finish_with((source, source_map));
      }
      InjectType::StyleTag | InjectType::SingletonStyleTag | InjectType::AutoStyleTag => {
        let source = inject_type.get_style_tag_code(
          &request,
          loader_context,
          &self.options,
          &runtime_options,
          is_singleton,
          is_auto,
        );
        loader_context.finish_with((source, source_map));
      }
    }

    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source = loader_context.take_content();
    let sm = loader_context.take_source_map();
    let request = loader_context.resource_query();

    loader_context.finish_with((source, sm));
    Ok(())
  }
}
