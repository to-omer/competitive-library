use crate::{ast_helper::ItemExt as _, config::Opt};
use quote::ToTokens as _;
use syn::{Attribute, Lit, Meta, NestedMeta, Path};

pub struct SnippetAttributes {
    pub name: Option<String>,
    pub include: Vec<String>,
    pub inline: bool,
    pub contains_entry: bool,
}
impl SnippetAttributes {
    fn push_key(&mut self, path: Path) {
        if let Some(id) = path.get_ident() {
            match id.to_string().as_str() {
                "inline" => {
                    self.inline = true;
                }
                "no_inline" => {
                    self.inline = false;
                }
                _ => {}
            }
        }
    }
    fn push_value(&mut self, lit: Lit) {
        if self.name.is_none() {
            if let Lit::Str(litstr) = lit {
                self.name = Some(litstr.value());
            }
        }
    }
    fn push_key_value(&mut self, path: Path, lit: Lit) {
        if let Some(id) = path.get_ident() {
            match id.to_string().as_str() {
                "name" => self.push_value(lit),
                "include" => {
                    if let Lit::Str(litstr) = lit {
                        self.include.push(litstr.value());
                    }
                }
                _ => {}
            }
        }
    }
    pub fn extend_one_kv(&mut self, kv: MetaKeyValue) {
        self.contains_entry = true;
        match kv {
            MetaKeyValue::K(path) => self.push_key(path),
            MetaKeyValue::V(lit) => self.push_value(lit),
            MetaKeyValue::KV(path, lit) => self.push_key_value(path, lit),
            MetaKeyValue::KVs(path, lits) => lits
                .into_iter()
                .for_each(|lit| self.push_key_value(path.clone(), lit)),
            MetaKeyValue::Empty => {}
        }
    }
    pub fn extend_kvs(&mut self, iter: impl IntoIterator<Item = MetaKeyValue>) {
        for kv in iter {
            self.extend_one_kv(kv);
        }
    }
}
impl From<&syn::Item> for SnippetAttributes {
    fn from(item: &syn::Item) -> Self {
        let mut self_ = Self {
            name: None,
            include: Vec::new(),
            inline: false,
            contains_entry: false,
        };
        if let Some(attrs) = item.get_attributes() {
            self_.extend_kvs(collect_kvs(attrs));
        }
        if self_.name.is_none() {
            self_.name = item.get_default_name();
        }
        self_
    }
}

pub fn is_snippet(path: &Path) -> bool {
    let path = path.into_token_stream().to_string();
    &path == ":: snippet :: entry" || &path == "snippet :: entry" || &path == "entry"
}

#[test]
fn test_is_snippet() {
    assert!(is_snippet(
        &syn::parse_str::<Path>("snippet::entry").unwrap()
    ));
    assert!(is_snippet(&syn::parse_str::<Path>("entry").unwrap()));
    assert!(is_snippet(
        &syn::parse_str::<Path>("::snippet::entry").unwrap()
    ));
    assert!(!is_snippet(&syn::parse_str::<Path>("snippet").unwrap()));
    assert!(!is_snippet(&syn::parse_str::<Path>("::entry").unwrap()));
}

pub fn check_cfg(attrs: &mut Vec<Attribute>, config: &Opt) -> bool {
    let mut next = Vec::new();
    let mut cond = true;
    for attr in attrs.drain(..) {
        if let Ok(meta) = attr.parse_meta() {
            if meta.path().is_ident("cfg") {
                if let Meta::List(list) = meta {
                    if let Some(NestedMeta::Meta(pred)) = list.nested.first() {
                        cond &= cfg_condition(pred, config);
                        continue;
                    }
                }
            }
        }
        next.push(attr);
    }
    *attrs = next;
    cond
}

pub fn flatten_cfg_attr(attrs: &mut Vec<Attribute>, config: &Opt) {
    let mut next = Vec::new();
    for attr in attrs.drain(..) {
        if let Ok(meta) = attr.parse_meta() {
            if meta.path().is_ident("cfg_attr") {
                if let Meta::List(list) = meta {
                    let mut it = list.nested.iter();
                    if let Some(NestedMeta::Meta(pred)) = it.next() {
                        if cfg_condition(pred, config) {
                            next.extend(it.map(to_attribute));
                            continue;
                        }
                    }
                }
            }
        }
        next.push(attr);
    }
    *attrs = next;
}

fn to_attribute(meta: &NestedMeta) -> Attribute {
    let meta = meta.to_token_stream();
    let attr: Attribute = syn::parse_quote!(#[ #meta ]);
    attr
}

#[test]
fn test_to_attribute() {
    let attr: Attribute = syn::parse_quote!(#[snippet::entry("name", inline)]);
    let attr = to_attribute(&NestedMeta::Meta(attr.parse_meta().unwrap()))
        .to_token_stream()
        .to_string();
    assert_eq!(attr.as_str(), r##"# [snippet :: entry ("name" , inline)]"##);
}

fn cfg_condition(pred: &Meta, config: &Opt) -> bool {
    if let Some(id) = pred.path().get_ident() {
        match id.to_string().as_str() {
            "all" => {
                if let Meta::List(list) = pred {
                    return list.nested.iter().all(|nm| {
                        if let NestedMeta::Meta(pred) = nm {
                            cfg_condition(pred, config)
                        } else {
                            true
                        }
                    });
                }
            }
            "any" => {
                if let Meta::List(list) = pred {
                    return list.nested.iter().any(|nm| {
                        if let NestedMeta::Meta(pred) = nm {
                            cfg_condition(pred, config)
                        } else {
                            false
                        }
                    });
                }
            }
            "not" => {
                if let Meta::List(list) = pred {
                    if let Some(NestedMeta::Meta(pred)) = list.nested.first() {
                        return !cfg_condition(pred, config);
                    }
                }
            }
            _ => return config.cfg.iter().any(|spec| spec == pred),
        }
    }
    // If there is a parsing error, it may be skipped and succeeded.
    true
}

pub enum MetaKeyValue {
    K(Path),
    V(Lit),
    KV(Path, Lit),
    KVs(Path, Vec<Lit>),
    Empty,
}

pub fn collect_kvs(attrs: &[Attribute]) -> Vec<MetaKeyValue> {
    let mut kvs = Vec::new();
    attrs
        .iter()
        .filter_map(|attr| attr.parse_meta().ok())
        .filter(|meta| is_snippet(meta.path()))
        .for_each(|meta| extend_kvs(meta, &mut kvs));
    kvs
}

fn extend_kvs(meta: Meta, kvs: &mut Vec<MetaKeyValue>) {
    match meta {
        Meta::Path(_) => kvs.push(MetaKeyValue::Empty),
        Meta::List(list) => {
            kvs.extend(list.nested.iter().map(|nm| match nm {
                NestedMeta::Meta(meta) => match meta {
                    Meta::Path(path) => MetaKeyValue::K(path.clone()),
                    Meta::List(list) => {
                        let mut vs = Vec::new();
                        for nm in list.nested.iter() {
                            if let NestedMeta::Lit(lit) = nm {
                                vs.push(lit.clone());
                            }
                        }
                        MetaKeyValue::KVs(list.path.clone(), vs)
                    }
                    Meta::NameValue(nv) => MetaKeyValue::KV(nv.path.clone(), nv.lit.clone()),
                },
                NestedMeta::Lit(lit) => MetaKeyValue::V(lit.clone()),
            }));
        }
        Meta::NameValue(nv) => {
            kvs.push(MetaKeyValue::V(nv.lit));
        }
    }
}
