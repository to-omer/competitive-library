extern crate proc_macro;

use crate::proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Ident, ItemFn, Lit,
    LitStr, Meta, NestedMeta, Token,
};

struct VerifyAttribute {
    url: String,
    #[allow(dead_code)]
    eps: Option<f64>,
    test_name: Option<String>,
    special_judge: Option<String>,
}

fn parse_attribute(attr: TokenStream) -> syn::Result<VerifyAttribute> {
    let punc = Punctuated::<NestedMeta, Token!(,)>::parse_terminated.parse(attr)?;
    let mut url = None;
    let mut eps = None;
    let mut test_name = None;
    let mut special_judge = None;
    for nmeta in punc.iter() {
        match nmeta {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                let ident = nv
                    .path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new(nv.path.span(), "unknown parameter"))?
                    .to_string();
                match ident.as_str() {
                    "url" => match &nv.lit {
                        Lit::Str(litstr) => match url {
                            None => url = Some(litstr.value()),
                            Some(_) => Err(syn::Error::new(litstr.span(), "extra url specified"))?,
                        },
                        _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
                    },
                    "eps" => match &nv.lit {
                        Lit::Float(litfloat) => match eps {
                            None => eps = Some(litfloat.base10_parse()?),
                            Some(_) => {
                                Err(syn::Error::new(litfloat.span(), "extra eps specified"))?
                            }
                        },
                        _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
                    },
                    "test" => match &nv.lit {
                        Lit::Str(litstr) => match test_name {
                            None => test_name = Some(litstr.value()),
                            Some(_) => Err(syn::Error::new(litstr.span(), "extra test specified"))?,
                        },
                        _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
                    },
                    "judge" => match &nv.lit {
                        Lit::Str(litstr) => match special_judge {
                            None => special_judge = Some(litstr.value()),
                            Some(_) => {
                                Err(syn::Error::new(litstr.span(), "extra judge specified"))?
                            }
                        },
                        _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
                    },
                    _ => (),
                }
            }
            NestedMeta::Lit(Lit::Str(litstr)) => match url {
                None => url = Some(litstr.value()),
                Some(_) => Err(syn::Error::new(litstr.span(), "extra url specified"))?,
            },
            NestedMeta::Lit(Lit::Float(litfloat)) => match eps {
                None => eps = Some(litfloat.base10_parse()?),
                Some(_) => Err(syn::Error::new(litfloat.span(), "extra eps specified"))?,
            },
            _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
        }
    }
    Ok(VerifyAttribute {
        url: url.ok_or_else(|| syn::Error::new(punc.span(), "url not specified"))?,
        eps,
        test_name,
        special_judge,
    })
}

#[proc_macro_attribute]
pub fn verify(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_attribute(attr) {
        Ok(VerifyAttribute {
            url,
            eps: _,
            test_name,
            special_judge,
        }) => {
            let ast = parse_macro_input!(item as ItemFn);
            let md =
                LitStr::new(&format!("{}.md", ast.sig.ident), Span::call_site()).to_token_stream();
            let fn_name = ast.sig.ident.to_token_stream();
            let url = LitStr::new(&url, Span::call_site()).to_token_stream();
            let test_name = test_name.unwrap_or(format!("verify_{}", ast.sig.ident));
            let test_name = Ident::new(&test_name, Span::call_site()).to_token_stream();
            let special_judge = special_judge
                .map(|judge| {
                    let judge = Ident::new(&judge, Span::call_site()).to_token_stream();
                    quote! { Some(#judge) }
                })
                .unwrap_or(quote! {None});
            let gen = quote! {
                #[cfg_attr(feature = "verify_doc", doc(include = #md))]
                #[cfg_attr(feature = "verify_doc", doc(alias = "verify"))]
                #ast
                #[test]
                #[ignore]
                fn #test_name() -> crate::verify::OjResult<()> {
                    let config = crate::verify::VerifyConfig::new(
                        #url,
                        file!(),
                        stringify!(#fn_name),
                    );
                    let res = match (config.get_testcases(), config.gen_env(#special_judge)) {
                        (Ok(problem), Ok(env)) => problem.verify(env, #fn_name),
                        (Err(err), _)  | (_, Err(err)) => Err(err),
                    };
                    config.finalize(res)
                }
            };
            gen.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}
