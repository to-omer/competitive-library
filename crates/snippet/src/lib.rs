use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn entry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn skip(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
