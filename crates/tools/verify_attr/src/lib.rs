extern crate proc_macro;
use crate::proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Ident, ItemFn, Lit,
    LitFloat, LitStr, Meta, NestedMeta, Token,
};

struct VerifyAttribute {
    problem_id: LitStr,
    eps: Option<LitFloat>,
    special_judge: Option<Ident>,
}

fn litstr2ident(litstr: &LitStr) -> Ident {
    Ident::new(&litstr.value(), litstr.span())
}

fn parse_attribute(attr: TokenStream) -> syn::Result<VerifyAttribute> {
    let punc = Punctuated::<NestedMeta, Token!(,)>::parse_terminated.parse(attr)?;
    let mut problem_id = None;
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
                    "problem_id" => match &nv.lit {
                        Lit::Str(litstr) => match problem_id {
                            None => problem_id = Some(litstr.clone()),
                            Some(_) => {
                                return Err(syn::Error::new(
                                    litstr.span(),
                                    "extra problem_id specified",
                                ))
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
            NestedMeta::Lit(Lit::Str(litstr)) => match problem_id {
                None => problem_id = Some(litstr.clone()),
                Some(_) => {
                    return Err(syn::Error::new(litstr.span(), "extra problem_id specified"))
                }
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
        problem_id: problem_id
            .ok_or_else(|| syn::Error::new(punc.span(), "problem_id not specified"))?,
        eps,
        special_judge,
    })
}

macro_rules! define_verify {
    ($name:ident, $service:path) => {
        #[proc_macro_attribute]
        pub fn $name(attr: TokenStream, item: TokenStream) -> TokenStream {
            match parse_attribute(attr) {
                Ok(VerifyAttribute {
                    problem_id,
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
                        quote! { case.judge_with_checker(buf.as_ref(), &checker) }
                    };
                    let gen = quote! {
                        #[cfg_attr(feature = "verify_doc", cfg_attr(all(), doc = include_str!(#md)))]
                        #[cfg_attr(feature = "verify_doc", doc(alias = "verify"))]
                        #ast
                        #[test]
                        #[ignore]
                        fn #verify_name() {
                            let target = ::std::module_path!().to_string() + "::" + &::std::stringify!(#verify_name);
                            let _ = ::verify::init_logger(target.to_string());
                            let config = ::verify::VerifyConfig::new($service, #problem_id, ::std::file!(), ::std::stringify!(#fn_name), &target);
                            let res = match config.get_testcases_and_checker() {
                                (::std::result::Result::Ok((cases, checker))) => {
                                    let mut res = ::verify::VerifyResults::new();
                                    for case in cases {
                                        let name = case.name.to_string();
                                        match case.load_testcase() {
                                            ::std::result::Result::Ok(case) => {
                                                let mut elapseds = ::std::vec![];
                                                loop {
                                                    let start = ::std::time::Instant::now();
                                                    let result = ::std::panic::catch_unwind(|| {
                                                        let mut buf = ::std::vec::Vec::new();
                                                        #fn_name(case.input.as_slice(), &mut buf);
                                                        buf
                                                    });
                                                    elapseds.push(start.elapsed());
                                                    if elapseds.len() >= 10 {
                                                        let status = match result {
                                                            ::std::result::Result::Ok(buf) => #inner,
                                                            ::std::result::Result::Err(err) => ::verify::VerifyStatus::RuntimeError,
                                                        };
                                                        res.push(case.case.name, status, elapseds);
                                                        break;
                                                    }
                                                }
                                            },
                                            ::std::result::Result::Err(err) => {
                                                res.push(name, ::verify::VerifyStatus::InternalError, ::std::vec![]);
                                            }
                                        }
                                    }
                                    ::std::result::Result::Ok(res)
                                },
                                ::std::result::Result::Err(err) => ::std::result::Result::Err(err),
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
    };
}

define_verify!(library_checker, ::verify::Service::LibraryChecker);
define_verify!(aizu_online_judge, ::verify::Service::AizuOnlineJudge);
