use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

pub fn expand_threadsafe_callback(input: DeriveInput) -> TokenStream {
  let struct_name = &input.ident;

  // 修复目标结构名称生成逻辑
  let struct_name_str = struct_name.to_string();
  let target_struct_name =
    if struct_name_str.starts_with("Raw") && struct_name_str.ends_with("Opts") {
      // 如果是 RawXxxOpts 格式，只移除 Raw 前缀
      struct_name_str.replace("Raw", "")
    } else if struct_name_str.starts_with("Raw") {
      // 如果只有 Raw 前缀，移除并添加 Opts
      format!("{}Opts", struct_name_str.replace("Raw", ""))
    } else {
      // 其他情况直接添加 Opts
      format!("{}Opts", struct_name_str)
    };

  let target_struct_ident = syn::Ident::new(&target_struct_name, struct_name.span());

  let Data::Struct(data_struct) = &input.data else {
    return syn::Error::new_spanned(&input, "ThreadsafeCallback can only be derived for structs")
      .to_compile_error();
  };

  let Fields::Named(fields) = &data_struct.fields else {
    return syn::Error::new_spanned(&input, "ThreadsafeCallback requires named fields")
      .to_compile_error();
  };

  let mut field_conversions = Vec::new();
  let mut has_threadsafe_callback = false;

  for field in &fields.named {
    let field_name = field.ident.as_ref().unwrap();
    let field_type = &field.ty;

    // 检查是否有 threadsafe_callback 属性
    let has_attr = field
      .attrs
      .iter()
      .any(|attr| attr.path().is_ident("threadsafe_callback"));

    if has_attr {
      has_threadsafe_callback = true;
      // 生成 ThreadsafeFunction 到 CompilationHookFn 的转换逻辑
      let conversion = generate_callback_conversion(field_name, field_type);
      field_conversions.push(conversion);
    } else {
      // 普通字段直接复制
      field_conversions.push(quote! {
        #field_name: value.#field_name
      });
    }
  }

  if !has_threadsafe_callback {
    return syn::Error::new_spanned(&input, "No fields marked with #[threadsafe_callback] found")
      .to_compile_error();
  }

  quote! {
    impl Into<#target_struct_ident> for #struct_name {
      fn into(self) -> #target_struct_ident {
        #target_struct_ident {
          #(#field_conversions),*
        }
      }
    }
  }
}

fn generate_callback_conversion(field_name: &syn::Ident, _field_type: &Type) -> TokenStream {
  quote! {
    #field_name: match self.#field_name {
      Some(callback) => {
        let callback = std::sync::Arc::new(callback);
        Some(Box::new(move |response| {
          let callback = callback.clone();
          Box::pin(async move {
            callback.call_with_sync(response.into()).await?;
            Ok(())
          })
        }))
      }
      None => None,
    }
  }
}
