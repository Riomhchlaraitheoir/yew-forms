use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(FormData, attributes(form_data))]
pub fn form_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    match macros_impl::form_data(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}