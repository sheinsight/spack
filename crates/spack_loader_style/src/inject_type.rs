use std::path::PathBuf;

use rspack_cacheable::cacheable;
use rspack_core::{LoaderContext, RunnerContext, contextify};
use serde::Serialize;
use strum_macros::{Display, EnumString};

use crate::StyleLoaderOpts;

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
      }
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

  pub fn get_style_hmr_code(&self, request: &str, lazy: bool) -> String {
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

    let hmr_code = if lazy {
      format!(
        r##"
        if (!isEqualLocals(oldLocals, isNamedExport ? namedExport : content.locals, isNamedExport)) {{
          module.hot.invalidate();
          return;
        }}
        oldLocals = isNamedExport ? namedExport : content.locals;
        if (update && refs > 0) {{
          update(content);
        }}"##
      )
    } else {
      format!(
        r##"
        if (!isEqualLocals(oldLocals, isNamedExport ? namedExport : content.locals, isNamedExport)) {{
          module.hot.invalidate();
          return;
        }}
        oldLocals = isNamedExport ? namedExport : content.locals;
        update(content);"##
      )
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

      var isNamedExport = !content.locals;
      var oldLocals = isNamedExport ? namedExport : content.locals;


      module.hot.accept(
        "!!{request}",
        function accept() {{
          {hmr_code}
        }}
      );
    }}

    module.hot.dispose(function dispose() {{
      {dispose_content}
    }});
}}
"##
    )
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

    let import_insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);

    let insert_option_code = self.get_insert_option_code(&loader_options.insert);

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

  pub fn get_lazy_style_tag_code(
    &self,
    request: &str,
    loader_context: &mut LoaderContext<RunnerContext>,
    loader_options: &StyleLoaderOpts,
    runtime_options: &str,
    is_singleton: bool,
    is_auto: bool,
  ) -> String {
    let style_dom_api_code = self.get_import_style_dom_api_code(is_auto, is_singleton);
    let insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);
    let set_attributes_code = self.get_set_attributes_code(&loader_options);

    let style_tag_transform_fn_code =
      if let Some(style_tag_transform) = &loader_options.style_tag_transform {
        loader_context
          .build_dependencies
          .insert(PathBuf::from(style_tag_transform));
        format!(r##"import styleTagTransformFn from "{style_tag_transform}";"##)
      } else {
        format!(r##"import styleTagTransformFn from "!@@/runtime/styleTagTransform.js";"##)
      };

    let insert_option_code = self.get_insert_option_code(&loader_options.insert);

    let is_old_ie_code = self.get_import_is_old_ie_code(is_auto);

    let exported = r##"
      if (content && content.locals) {{
        exported.locals = content.locals;
      }}"##;

    let style_tag_transform_fn = self.get_style_tag_transform_fn(is_singleton);

    let dom_api = self.get_dom_api(is_auto);

    let hmr_code = self.get_style_hmr_code(&request, true);

    let source = format!(
      r##"
var exported = {{}};
import API from "!@@/runtime/injectStylesIntoStyleTag.js";

{style_dom_api_code}
{insert_by_selector_code}
{set_attributes_code}
import insertStyleElement from "!@@/runtime/insertStyleElement.js";
{style_tag_transform_fn_code}
import content, * as namedExport from "!!{request}";
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

export * from "!!{request}";
export default exported;
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
    let style_dom_api_code = self.get_import_style_dom_api_code(is_auto, is_singleton);
    let insert_by_selector_code =
      self.get_import_insert_by_selector_code(loader_context, &loader_options.insert);
    let set_attributes_code = self.get_set_attributes_code(&loader_options);

    let style_tag_transform_fn_code =
      if let Some(style_tag_transform) = &loader_options.style_tag_transform {
        loader_context
          .build_dependencies
          .insert(PathBuf::from(style_tag_transform));
        format!(r##"import styleTagTransformFn from "{style_tag_transform}";"##)
      } else {
        format!(r##"import styleTagTransformFn from "!@@/runtime/styleTagTransform.js";"##)
      };

    let insert_option_code = self.get_insert_option_code(&loader_options.insert);

    let is_old_ie_code = self.get_import_is_old_ie_code(is_auto);

    let exported = r##""##;

    let style_tag_transform_fn = self.get_style_tag_transform_fn(is_singleton);

    let dom_api = self.get_dom_api(is_auto);

    let hmr_code = self.get_style_hmr_code(&request, false);

    let source = format!(
      r##"
      import API from "!@@/runtime/injectStylesIntoStyleTag.js";
      {style_dom_api_code}
      {insert_by_selector_code}
      {set_attributes_code}
      import insertStyleElement from "!@@/runtime/insertStyleElement.js";
      {style_tag_transform_fn_code}
      import content, * as namedExport from "!!{request}";
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

      export * from "!!{request}";
      export default content && content.locals ? content.locals : undefined;
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
