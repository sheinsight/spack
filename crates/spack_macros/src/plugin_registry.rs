use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemEnum, Meta, Result};

// 辅助函数：将 PascalCase 转换为 snake_case
fn to_snake_case(s: &str) -> String {
  let mut result = String::new();
  let mut chars = s.chars().peekable();

  while let Some(ch) = chars.next() {
    if ch.is_uppercase() && !result.is_empty() {
      result.push('_');
    }
    result.push(ch.to_lowercase().next().unwrap());
  }

  result
}

pub fn expand_plugin_registry(input: ItemEnum) -> Result<TokenStream> {
  let enum_name = &input.ident;
  let enum_attrs = &input.attrs;
  let enum_vis = &input.vis;

  // 清理枚举变体（移除 register 属性）
  let clean_variants: Vec<_> = input
    .variants
    .iter()
    .map(|variant| {
      let variant_name = &variant.ident;
      let clean_attrs: Vec<_> = variant
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("register"))
        .collect();

      quote! {
          #(#clean_attrs)*
          #variant_name
      }
    })
    .collect();

  // 为每个有 register 属性的变体生成注册函数
  let register_functions: Vec<_> = input.variants.iter().filter_map(|variant| {
        let variant_name = &variant.ident;

        // 查找 register 属性
        variant.attrs.iter().find(|attr| attr.path().is_ident("register")).and_then(|attr| {
            // 解析属性内容
            match &attr.meta {
                Meta::List(meta_list) => {
                    let binding_path = &meta_list.tokens;
                    let snake_case_name = to_snake_case(&variant_name.to_string());
                    let fn_name = quote::format_ident!("register_{}", snake_case_name);

                    Some(quote! {
                        #[napi]
                        fn #fn_name() -> napi::Result<()> {
                            let name = stringify!(#variant_name).to_string();
                            let plugin = #binding_path;
                            rspack_binding_builder::register_custom_plugin(name, plugin)
                                .map_err(|_| napi::Error::from_reason(format!("Failed to register {}", stringify!(#variant_name))))
                        }
                    })
                }
                _ => None
            }
        })
    }).collect();

  Ok(quote! {
      #(#enum_attrs)*
      #enum_vis enum #enum_name {
          #(#clean_variants,)*
      }
      #(#register_functions)*
  })
}
