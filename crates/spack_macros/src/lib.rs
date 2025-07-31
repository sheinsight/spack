mod plugin_registry;
mod threadsafe_callback;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn plugin_registry(_args: TokenStream, input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as syn::ItemEnum);
  plugin_registry::expand_plugin_registry(input)
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

#[proc_macro_derive(ThreadsafeCallback, attributes(threadsafe_callback))]
pub fn threadsafe_callback_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as syn::DeriveInput);
  threadsafe_callback::expand_threadsafe_callback(input).into()
}
