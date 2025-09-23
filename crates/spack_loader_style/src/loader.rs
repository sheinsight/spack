use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext, contextify};
use rspack_error::Result;
use rspack_loader_runner::DisplayWithSuffix;
use serde::Serialize;
use strum_macros::{Display, EnumString};

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct StyleLoaderOpts {
  pub base: Option<i64>,
  pub inject_type: Option<InjectType>,
  pub insert: Option<String>,
  pub output: String,
  pub style_tag_transform: Option<String>,
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
  pub fn get_link_hmr_code(&self, request: &str) -> String {
    return format!(
      r##"
if (module.hot) {{
  module.hot.accept(
  "!!{request}",
  function(){{
    update(content);
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
    insert: &Option<String>,
  ) -> String {
    let context = &loader_context.context.options.context;
    match &insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        let path = contextify(context, &insert);
        loader_context
          .build_dependencies
          .insert(PathBuf::from(insert));
        format!(r##"import insertFn from "{path}";"##)
      }
      _ => {
        format!(r##"import insertFn from "!@@/runtime/insertBySelector.js";"##)
      } // None => "".to_string(),
    }
  }

  pub fn get_import_link_content_code(&self, request: &str) -> String {
    format!(r##"import content from "!!{request}";"##)
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

  pub fn get_import_style_api_code(&self) -> String {
    format!(r##"import API from "!@@/runtime/injectStylesIntoStyleTag.js";"##)
  }

  pub fn get_export_lazy_style_code(&self, request: &str) -> String {
    format!(
      r##"
export * from "!!{request}";
export default exported;
"##
    )
  }

  pub fn get_export_style_code(&self, request: &str) -> String {
    format!(
      r##"
    export * from "!!{request}";
    export default content && content.locals ? content.locals : undefined;"##
    )
  }

  pub fn get_style_hmr_code(&self, request: &str, lazy: bool) -> String {
    let is_named_export = "!content.locals";

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

    let hmr_code = match lazy {
      true => format!(
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
      false => format!(
        r##"
        if (!isEqualLocals(oldLocals, isNamedExport ? namedExport : content.locals, isNamedExport)) {{
          module.hot.invalidate();
          return;
        }}
        oldLocals = isNamedExport ? namedExport : content.locals;
        update(content);"##
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
        "!!{request}",
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

  pub fn get_import_insert_style_element_code(&self) -> String {
    format!(r##"import insertStyleElement from "!@@/runtime/insertStyleElement.js";"##)
  }

  // TODO
  pub fn get_style_tag_transform_fn_code(&self, loader_options: &StyleLoaderOpts) -> String {
    if let Some(style_tag_transform) = &loader_options.style_tag_transform {
      format!(r##"import styleTagTransformFn from "{style_tag_transform}";"##)
    } else {
      format!(r##"import styleTagTransformFn from "@@/runtime/styleTagTransform.js";"##)
    }
  }

  pub fn get_import_is_old_ie_code(&self, is_auto: bool) -> String {
    if is_auto {
      format!(r##"import isOldIE from "!@@/runtime/isOldIE.js";"##)
    } else {
      format!("")
    }
  }

  pub fn get_import_style_content_code(&self, request: &str) -> String {
    format!(r##"import content, * as namedExport from "!!{request}";"##)
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
    let modules = match &loader_options.attributes {
      Some(attributes) if attributes.contains_key("nonce") => {
        format!(r##"!@@/runtime/setAttributesWithAttributesAndNonce.js"##)
      }
      Some(_) => {
        format!(r##"!@@/runtime/setAttributesWithAttributes.js"##)
      }
      None => format!(r##"!@@/runtime/setAttributesWithoutAttributes.js"##),
    };

    format!(r##"import setAttributes from "{modules}";"##)
  }

  pub fn get_import_style_dom_api_code(&self, is_auto: bool, is_singleton: bool) -> String {
    if is_auto {
      return format!(
        r##"
      import domAPI from "!@@/runtime/styleDomAPI.js";
      import domAPISingleton from "!@@/runtime/singletonStyleDomAPI.js";"##
      );
    }

    if is_singleton {
      return format!(r##"import domAPI from "!@@/runtime/singletonStyleDomAPI.js";"##);
    } else {
      return format!(r##"import domAPI from "!@@/runtime/styleDomAPI.js";"##);
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
    let import_link_api_code = r#"import API from "@@/runtime/injectStylesIntoLinkTag.js";"#;

    let hmr_code = self.get_link_hmr_code(&request);
    let import_insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);
    let import_link_content_code = self.get_import_link_content_code(&request);
    let insert_option_code = self.get_insert_option_code(&loader_options.insert);
    let export_code = format!(r##"export default {{}};"##);

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

  pub fn get_lazy_style_tag_code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
    runtime_options: &str,
    is_singleton: bool,
    is_auto: bool,
  ) -> String {
    let style_api_code = self.get_import_style_api_code();
    let style_dom_api_code = self.get_import_style_dom_api_code(is_auto, is_singleton);
    let insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);
    let set_attributes_code = self.get_set_attributes_code(&loader_options);
    let insert_style_element_code = self.get_import_insert_style_element_code();

    let style_tag_transform_fn_code =
      if let Some(style_tag_transform) = &loader_options.style_tag_transform {
        loader_context
          .build_dependencies
          .insert(PathBuf::from(style_tag_transform));
        format!(r##"import styleTagTransformFn from "{style_tag_transform}";"##)
      } else {
        format!(r##"import styleTagTransformFn from "!@@/runtime/styleTagTransform.js";"##)
      };

    // let style_tag_transform_fn_code = self.get_style_tag_transform_fn_code(&loader_options);
    let import_style_content_code = self.get_import_style_content_code(&request);
    let insert_option_code = self.get_insert_option_code(&loader_options.insert);

    let is_old_ie_code = self.get_import_is_old_ie_code(is_auto);

    let exported = r##"
      if (content && content.locals) {{
        exported.locals = content.locals;
      }}"##;

    let style_tag_transform_fn = self.get_style_tag_transform_fn(is_singleton);

    let dom_api = self.get_dom_api(is_auto);

    let hmr_code = self.get_style_hmr_code(&request, true);

    let export_code = self.get_export_lazy_style_code(&request);

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
    let style_api_code = self.get_import_style_api_code();
    let style_dom_api_code = self.get_import_style_dom_api_code(is_auto, is_singleton);
    let insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);
    let set_attributes_code = self.get_set_attributes_code(&loader_options);
    let insert_style_element_code = self.get_import_insert_style_element_code();

    let style_tag_transform_fn_code =
      if let Some(style_tag_transform) = &loader_options.style_tag_transform {
        loader_context
          .build_dependencies
          .insert(PathBuf::from(style_tag_transform));
        format!(r##"import styleTagTransformFn from "{style_tag_transform}";"##)
      } else {
        format!(r##"import styleTagTransformFn from "!@@/runtime/styleTagTransform.js";"##)
      };

    // let style_tag_transform_fn_code = self.get_style_tag_transform_fn_code(&loader_options);
    let import_style_content_code = self.get_import_style_content_code(&request);
    let insert_option_code = self.get_insert_option_code(&loader_options.insert);

    let is_old_ie_code = self.get_import_is_old_ie_code(is_auto);

    let exported = r##""##;

    let style_tag_transform_fn = self.get_style_tag_transform_fn(is_singleton);

    let dom_api = self.get_dom_api(is_auto);

    let hmr_code = self.get_style_hmr_code(&request, false);

    let export_code = self.get_export_style_code(&request);

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

impl Default for InjectType {
  fn default() -> Self {
    InjectType::StyleTag // 保持当前默认值
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

    let inject_type = self.options.inject_type.unwrap_or_default();

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

    let source = match inject_type {
      InjectType::LinkTag => {
        let source =
          inject_type.get_link_tag_code(&request, loader_context, &self.options, &runtime_options);
        source
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
        source
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
        source
      }
    };

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
