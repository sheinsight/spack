mod register_plugin;
mod threadsafe_callback;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn register_plugin(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as register_plugin::RegisterPluginInput);
  input.expand().into()
}

#[proc_macro_derive(ThreadsafeCallback, attributes(threadsafe_callback))]
pub fn threadsafe_callback_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as syn::DeriveInput);
  threadsafe_callback::expand_threadsafe_callback(input).into()
}
