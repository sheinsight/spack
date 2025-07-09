use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  Expr, Token,
};

pub struct RegisterPluginInput {
  name: Expr, // 将 LitStr 改为 Expr
  plugin: Expr,
}

impl Parse for RegisterPluginInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let name = input.parse()?;
    <Token![,]>::parse(input)?;
    let plugin = input.parse()?;
    Ok(RegisterPluginInput { name, plugin })
  }
}

impl RegisterPluginInput {
  pub fn expand(self) -> TokenStream {
    let RegisterPluginInput { name, plugin } = self;
    let plugin_register_ident = format!(
      "register_{}",
      quote! { #name }
        .to_string()
        .split("::")
        .last()
        .unwrap_or("unknown")
        .replace(" ", "")
    );
    let plugin_register_ident = Ident::new(&plugin_register_ident, Span::call_site());

    let expanded = quote! {
        #[napi]
        fn #plugin_register_ident() -> napi::Result<()> {
            let name = (#name).clone().to_string();
            let plugin = #plugin;
            let name_clone = name.clone();
            rspack_binding_builder::register_custom_plugin(name, plugin).map_err(move |e| {
                napi::Error::from_reason(format!("Cannot register plugins under the same name: {}", name_clone))
            })
        }
    };

    expanded
  }
}
