use std::{collections::HashMap, path::PathBuf};

use rspack_cacheable::cacheable;
use rspack_core::{LoaderContext, RunnerContext, contextify};
use serde::Serialize;
use strum_macros::{Display, EnumString};

use crate::{
  StyleLoaderOpts, get_dom_api, get_import_is_old_ie_code, get_import_style_dom_api_code,
  get_insert_option_code, get_set_attributes_code, get_style_hmr_code, get_style_tag_transform_fn,
};

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
  fn get_link_tag_code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
    runtime_options: &str,
  ) -> String {
    let import_link_api_code = r#"import API from "@@/runtime/injectStylesIntoLinkTag.js";"#;

    // let import_insert_by_selector_code =
    //   self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);

    let import_insert_by_selector_code = match &loader_options.insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        let path = contextify(&loader_context.context.options.context, &insert);
        loader_context
          .build_dependencies
          .insert(PathBuf::from(&path));
        format!(r##"import insertFn from "{path}";"##)
      }
      _ => {
        format!(r##"import insertFn from "!@@/runtime/insertBySelector.js";"##)
      }
    };

    let insert_option_code = get_insert_option_code(&loader_options.insert);

    let source = format!(
      r##"
{import_link_api_code}
{import_insert_by_selector_code}
import content from "!!{request}";

var options = {runtime_options};
{insert_option_code}
var update = API(content, options);

if (module.hot) {{
  module.hot.accept(
    "!!{request}",
    function accept(){{
      update(content);
    }}
  );
  module.hot.dispose(function dispose() {{
    update();
  }});
}}

export default {{}};
"##
    );
    source
  }

  fn get_lazy_style_tag_code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
    runtime_options: &str,
    is_singleton: bool,
    is_auto: bool,
  ) -> String {
    let style_dom_api_code = get_import_style_dom_api_code(is_auto, is_singleton);
    // let insert_by_selector_code =
    //   self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);

    let import_insert_by_selector_code = match &loader_options.insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        let path = contextify(&loader_context.context.options.context, &insert);
        loader_context
          .build_dependencies
          .insert(PathBuf::from(&path));
        format!(r##"import insertFn from "{path}";"##)
      }
      _ => {
        format!(r##"import insertFn from "!@@/runtime/insertBySelector.js";"##)
      }
    };

    let set_attributes_code = get_set_attributes_code(&loader_options);

    let style_tag_transform_fn_code =
      if let Some(style_tag_transform) = &loader_options.style_tag_transform {
        loader_context
          .build_dependencies
          .insert(PathBuf::from(style_tag_transform));
        format!(r##"import styleTagTransformFn from "{style_tag_transform}";"##)
      } else {
        format!(r##"import styleTagTransformFn from "!@@/runtime/styleTagTransform.js";"##)
      };

    let insert_option_code = get_insert_option_code(&loader_options.insert);

    let is_old_ie_code = get_import_is_old_ie_code(is_auto);

    let style_tag_transform_fn = get_style_tag_transform_fn(is_singleton);

    let dom_api = get_dom_api(is_auto);

    let hmr_code = get_style_hmr_code(&request, true);

    let source = format!(
      r##"
var exported = {{}};
import API from "!@@/runtime/injectStylesIntoStyleTag.js";

{style_dom_api_code}
{import_insert_by_selector_code}
{set_attributes_code}
import insertStyleElement from "!@@/runtime/insertStyleElement.js";
{style_tag_transform_fn_code}
import content, * as namedExport from "!!{request}";
{is_old_ie_code}

if (content && content.locals) {{
  exported.locals = content.locals;
}}

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

export * from "!!{request}";
export default exported;
"##
    );

    println!("source: {}", source.clone());

    source
  }

  fn get_style_tag_code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
    runtime_options: &str,
    is_singleton: bool,
    is_auto: bool,
  ) -> String {
    let style_dom_api_code = get_import_style_dom_api_code(is_auto, is_singleton);
    // let insert_by_selector_code =
    //   self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);

    let import_insert_by_selector_code = match &loader_options.insert {
      Some(insert) if PathBuf::from(insert).is_absolute() => {
        let path = contextify(&loader_context.context.options.context, &insert);
        loader_context
          .build_dependencies
          .insert(PathBuf::from(&path));
        format!(r##"import insertFn from "{path}";"##)
      }
      _ => {
        format!(r##"import insertFn from "!@@/runtime/insertBySelector.js";"##)
      }
    };

    let set_attributes_code = get_set_attributes_code(&loader_options);

    let style_tag_transform_fn_code =
      if let Some(style_tag_transform) = &loader_options.style_tag_transform {
        loader_context
          .build_dependencies
          .insert(PathBuf::from(style_tag_transform));
        format!(r##"import styleTagTransformFn from "{style_tag_transform}";"##)
      } else {
        format!(r##"import styleTagTransformFn from "!@@/runtime/styleTagTransform.js";"##)
      };

    let insert_option_code = get_insert_option_code(&loader_options.insert);

    let is_old_ie_code = get_import_is_old_ie_code(is_auto);

    let style_tag_transform_fn = get_style_tag_transform_fn(is_singleton);

    let dom_api = get_dom_api(is_auto);

    let hmr_code = get_style_hmr_code(&request, false);

    let source = format!(
      r##"
      import API from "!@@/runtime/injectStylesIntoStyleTag.js";
      {style_dom_api_code}
      {import_insert_by_selector_code}
      {set_attributes_code}
      import insertStyleElement from "!@@/runtime/insertStyleElement.js";
      {style_tag_transform_fn_code}
      import content, * as namedExport from "!!{request}";
      {is_old_ie_code}

      var options = {runtime_options};

      {style_tag_transform_fn}
      options.setAttributes = setAttributes;
      {insert_option_code}
      options.domAPI = {dom_api};
      options.insertStyleElement = insertStyleElement;

      var update = API(content, options);

      {hmr_code}

      export * from "!!{request}";
      export default content && content.locals ? content.locals : undefined;
      "##
    );

    source
  }

  pub fn code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
  ) -> String {
    let inject_type = loader_options.inject_type.unwrap_or_default();
    let mut runtime_options = HashMap::new();
    if let Some(attributes) = &loader_options.attributes {
      runtime_options.insert("attributes".to_string(), serde_json::json!(attributes));
    }
    if let Some(base) = &loader_options.base {
      runtime_options.insert("base".to_string(), serde_json::json!(base));
    }
    let runtime_options = serde_json::to_string_pretty(&runtime_options).unwrap();

    let source = match inject_type {
      InjectType::LinkTag => {
        inject_type.get_link_tag_code(&request, loader_context, loader_options, &runtime_options)
      }
      style @ (InjectType::StyleTag | InjectType::SingletonStyleTag | InjectType::AutoStyleTag) => {
        let is_singleton = matches!(style, InjectType::SingletonStyleTag);
        let is_auto = matches!(style, InjectType::AutoStyleTag);
        style.get_style_tag_code(
          &request,
          loader_context,
          loader_options,
          &runtime_options,
          is_singleton,
          is_auto,
        )
      }
      lazy @ (InjectType::LazyStyleTag
      | InjectType::LazySingletonStyleTag
      | InjectType::LazyAutoStyleTag) => {
        let is_singleton = matches!(lazy, InjectType::LazySingletonStyleTag);
        let is_auto = matches!(lazy, InjectType::LazyAutoStyleTag);
        lazy.get_lazy_style_tag_code(
          &request,
          loader_context,
          loader_options,
          &runtime_options,
          is_singleton,
          is_auto,
        )
      }
    };

    source
  }
}

impl Default for InjectType {
  fn default() -> Self {
    InjectType::StyleTag // 保持当前默认值
  }
}
