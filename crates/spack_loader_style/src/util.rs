use std::path::PathBuf;

use crate::StyleLoaderOpts;

pub fn get_insert_option_code(insert: &Option<String>) -> String {
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

pub fn get_import_style_dom_api_code(is_auto: bool, is_singleton: bool) -> String {
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

pub fn get_set_attributes_code(loader_options: &StyleLoaderOpts) -> String {
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

pub fn get_style_tag_transform_fn(is_singleton: bool) -> String {
  if is_singleton {
    format!("")
  } else {
    format!("options.styleTagTransform = styleTagTransformFn;")
  }
}

pub fn get_dom_api(is_auto: bool) -> String {
  if is_auto {
    format!("isOldIE() ? domAPISingleton : domAPI;")
  } else {
    format!("domAPI;")
  }
}

pub fn get_import_is_old_ie_code(is_auto: bool) -> String {
  if is_auto {
    format!(r##"import isOldIE from "!@@/runtime/isOldIE.js";"##)
  } else {
    format!("")
  }
}

pub fn get_style_tag_transform_fn_code(loader_options: &StyleLoaderOpts) -> String {
  if let Some(style_tag_transform) = &loader_options.style_tag_transform {
    format!(r##"import styleTagTransformFn from "{style_tag_transform}";"##)
  } else {
    format!(r##"import styleTagTransformFn from "@@/runtime/styleTagTransform.js";"##)
  }
}

pub fn get_style_hmr_code(request: &str, lazy: bool) -> String {
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
