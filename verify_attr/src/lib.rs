extern crate proc_macro;

use crate::proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Ident, ItemFn, Lit,
    LitFloat, LitStr, Meta, NestedMeta, Token,
};

struct VerifyAttribute {
    url: LitStr,
    eps: Option<LitFloat>,
    test_name: Option<Ident>,
    special_judge: Option<Ident>,
}

fn litstr2ident(litstr: &LitStr) -> Ident {
    Ident::new(&litstr.value(), litstr.span())
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
                            None => url = Some(litstr.clone()),
                            Some(_) => Err(syn::Error::new(litstr.span(), "extra url specified"))?,
                        },
                        _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
                    },
                    "eps" => match &nv.lit {
                        Lit::Str(litstr) => match eps {
                            None => match litstr.value().parse::<f64>() {
                                Ok(_) => eps = Some(LitFloat::new(&litstr.value(), litstr.span())),
                                Err(_) => Err(syn::Error::new(litstr.span(), "parse eps error"))?,
                            },
                            Some(_) => Err(syn::Error::new(litstr.span(), "extra eps specified"))?,
                        },
                        _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
                    },
                    "test" => match &nv.lit {
                        Lit::Str(litstr) => match test_name {
                            None => test_name = Some(litstr2ident(litstr)),
                            Some(_) => Err(syn::Error::new(litstr.span(), "extra test specified"))?,
                        },
                        _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
                    },
                    "judge" => match &nv.lit {
                        Lit::Str(litstr) => match special_judge {
                            None => special_judge = Some(litstr2ident(litstr)),
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
                None => url = Some(litstr.clone()),
                Some(_) => Err(syn::Error::new(litstr.span(), "extra url specified"))?,
            },
            _ => Err(syn::Error::new(nmeta.span(), "unknown meta value"))?,
        }
    }
    if eps.is_some() && special_judge.is_some() {
        Err(syn::Error::new(
            punc.span(),
            "only speciy one of `eps` or `judge`",
        ))?
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
            eps,
            test_name,
            special_judge,
        }) => {
            let ast = parse_macro_input!(item as ItemFn);
            let md =
                LitStr::new(&format!("{}.md", ast.sig.ident), Span::call_site()).to_token_stream();
            let fn_name = ast.sig.ident.to_token_stream();
            let url = url.to_token_stream();
            let test_name = test_name
                .unwrap_or_else(|| {
                    Ident::new(&format!("verify_{}", ast.sig.ident), Span::call_site())
                })
                .to_token_stream();
            let inner = if let Some(special_judge) = special_judge {
                let special_judge = special_judge.to_token_stream();
                quote! { case.judge_with_judger(buf.as_ref(), #special_judge) }
            } else if let Some(eps) = eps {
                let eps = eps.to_token_stream();
                quote! { case.judge_with_eps(buf.as_ref(), #eps) }
            } else {
                quote! { case.judge_with_env(buf.as_ref(), &env) }
            };
            let gen = quote! {
                #[cfg_attr(feature = "verify_doc", doc(include = #md))]
                #[cfg_attr(feature = "verify_doc", doc(alias = "verify"))]
                #ast
                #[test]
                #[ignore]
                fn #test_name() -> crate::verify::OjResult<()> {
                    let config = crate::verify::VerifyConfig::new(#url, file!(), stringify!(#fn_name));
                    let res = match (config.get_testcases(), config.gen_env()) {
                        (Ok(problem), Ok(env)) => {
                            let mut res = Vec::new();
                            for case in problem.tests.iter() {
                                let mut buf = Vec::new();
                                let (result, elapsed) = case.execute(&mut buf, #fn_name);
                                let status = if result.is_ok() { #inner } else { crate::verify::VerifyStatus::RE };
                                res.push(crate::verify::VerifyResult::new(case.name.clone(), status, elapsed));
                            }
                            Ok(crate::verify::VerifyResults::new(res))
                        },
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
