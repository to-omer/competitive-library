use proc_macro::TokenStream;
use quote::ToTokens;
use snippet_core::entry::EntryArgs;
use syn::{parse, parse_macro_input, Error, Item};

#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as EntryArgs);
    match parse::<Item>(item) {
        Err(_) => Error::new_spanned(attr, "expected to apply to `Item`")
            .to_compile_error()
            .into(),
        Ok(item) => {
            if let Err(err) = attr.try_to_entry(&item) {
                return err.to_compile_error().into();
            }
            item.into_token_stream().into()
        }
    }
}

#[proc_macro_attribute]
pub fn skip(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
