use proc_macro::TokenStream;

#[proc_macro_derive(Model, attributes(model))]
pub fn model(_: TokenStream) -> TokenStream {
    TokenStream::new()
}
