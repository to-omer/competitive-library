extern crate proc_macro;

use crate::proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Ident, ItemFn, Lit,
    LitFloat, LitStr, Meta, NestedMeta, Token,
};

struct VerifyAttribute {
    url: LitStr,
    eps: Option<LitFloat>,
    special_judge: Option<Ident>,
}

fn litstr2ident(litstr: &LitStr) -> Ident {
    Ident::new(&litstr.value(), litstr.span())
}

fn parse_attribute(attr: TokenStream) -> syn::Result<VerifyAttribute> {
    let punc = Punctuated::<NestedMeta, Token!(,)>::parse_terminated.parse(attr)?;
    let mut url = None;
    let mut eps = None;
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
                            Some(_) => {
                                return Err(syn::Error::new(litstr.span(), "extra url specified"))
                            }
                        },
                        _ => return Err(syn::Error::new(nmeta.span(), "unknown meta value")),
                    },
                    "eps" => match &nv.lit {
                        Lit::Str(litstr) => match eps {
                            None => match litstr.value().parse::<f64>() {
                                Ok(_) => eps = Some(LitFloat::new(&litstr.value(), litstr.span())),
                                Err(_) => {
                                    return Err(syn::Error::new(litstr.span(), "parse eps error"))
                                }
                            },
                            Some(_) => {
                                return Err(syn::Error::new(litstr.span(), "extra eps specified"))
                            }
                        },
                        _ => return Err(syn::Error::new(nmeta.span(), "unknown meta value")),
                    },
                    "judge" => match &nv.lit {
                        Lit::Str(litstr) => match special_judge {
                            None => special_judge = Some(litstr2ident(litstr)),
                            Some(_) => {
                                return Err(syn::Error::new(litstr.span(), "extra judge specified"))
                            }
                        },
                        _ => return Err(syn::Error::new(nmeta.span(), "unknown meta value")),
                    },
                    _ => (),
                }
            }
            NestedMeta::Lit(Lit::Str(litstr)) => match url {
                None => url = Some(litstr.clone()),
                Some(_) => return Err(syn::Error::new(litstr.span(), "extra url specified")),
            },
            _ => return Err(syn::Error::new(nmeta.span(), "unknown meta value")),
        }
    }
    if eps.is_some() && special_judge.is_some() {
        return Err(syn::Error::new(
            punc.span(),
            "only speciy one of `eps` or `judge`",
        ));
    }
    Ok(VerifyAttribute {
        url: url.ok_or_else(|| syn::Error::new(punc.span(), "url not specified"))?,
        eps,
        special_judge,
    })
}

#[proc_macro_attribute]
pub fn verify(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_attribute(attr) {
        Ok(VerifyAttribute {
            url,
            eps,
            special_judge,
        }) => {
            let ast = parse_macro_input!(item as ItemFn);
            let fn_name = ast.sig.ident.clone();
            let md = LitStr::new(&format!("{}.md", fn_name), Span::call_site());
            let verify_name = Ident::new(&format!("verify_{}", fn_name), Span::call_site());
            let test_name = Ident::new(&format!("test_{}", fn_name), Span::call_site());
            let inner = if let Some(special_judge) = special_judge {
                quote! { case.judge_with_judger(buf.as_ref(), #special_judge) }
            } else if let Some(eps) = eps {
                quote! { case.judge_with_eps(buf.as_ref(), #eps) }
            } else {
                quote! { case.judge_with_env(buf.as_ref(), &env) }
            };
            let gen = quote! {
                #[cfg_attr(feature = "verify_doc", cfg_attr(TRUE, doc = include_str!(#md)))]
                #[cfg_attr(feature = "verify_doc", doc(alias = "verify"))]
                #ast
                #[test]
                #[ignore]
                fn #verify_name() {
                    let target = ::std::module_path!().to_string() + "::" + &::std::stringify!(#verify_name);
                    let _ = ::verify::init_logger(target.to_string());
                    let config = ::verify::VerifyConfig::new(#url, ::std::file!(), ::std::stringify!(#fn_name), &target);
                    let res = match (config.get_testcases(), config.gen_env()) {
                        (::std::result::Result::Ok(problem), ::std::result::Result::Ok(env)) => {
                            let mut res = ::verify::VerifyResults::new();
                            for case in problem.tests.iter() {
                                let start = ::std::time::Instant::now();
                                let result = ::std::panic::catch_unwind(|| {
                                    let mut buf = ::std::vec::Vec::new();
                                    case.execute(&mut buf, #fn_name);
                                    buf
                                });
                                let elapsed = start.elapsed();
                                let status = match result {
                                    ::std::result::Result::Ok(buf) => #inner,
                                    ::std::result::Result::Err(err) => ::verify::VerifyStatus::RuntimeError,
                                };
                                res.push(case.name.clone(), status, elapsed);
                            }
                            ::std::result::Result::Ok(res)
                        },
                        (::std::result::Result::Err(err), _)  | (_, ::std::result::Result::Err(err)) => ::std::result::Result::Err(err),
                    };
                    let res = config.finalize(res);
                    ::std::assert!(res.is_ok(), "{}", res.unwrap_err());
                }
                #[cfg_attr(feature = "verify_test", test)]
                fn #test_name() {
                    let res = ::std::panic::catch_unwind(|| {
                        let (stdin, stdout) = (::std::io::stdin(), ::std::io::stdout());
                        let (mut reader, mut writer) = (::std::io::BufReader::new(stdin.lock()), ::std::io::BufWriter::new(stdout.lock()));
                        #fn_name(reader, writer);
                    });
                    ::std::assert!(res.is_ok(), "{}", ::std::stringify!(#fn_name));
                }
            };
            gen.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}
