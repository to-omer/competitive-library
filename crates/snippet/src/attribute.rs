use crate::ast_helper::{get_attributes_of_item, get_default_name_of_item};
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
            self.name = Some(lit.to_token_stream().to_string());
        }
    }
    fn push_key_value(&mut self, path: Path, lit: Lit) {
        if let Some(id) = path.get_ident() {
            match id.to_string().as_str() {
                "name" => self.push_value(lit),
                "include" => self.include.push(lit.to_token_stream().to_string()),
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
            name: get_default_name_of_item(item),
            include: Vec::new(),
            inline: false,
            contains_entry: false,
        };
        if let Some(attrs) = get_attributes_of_item(item) {
            self_.extend_kvs(collect_kvs(attrs));
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
