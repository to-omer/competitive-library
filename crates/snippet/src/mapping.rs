use std::collections::{BTreeMap, HashMap};

use crate::{
    ast_helper::{get_attributes_of_item, get_attributes_of_item_mut},
    attribute::{is_snippet, SnippetAttributes},
    config::Opt,
    output::VSCode,
};
use quote::ToTokens as _;
use syn::{
    visit::{self, Visit},
    Attribute, Item,
};

#[derive(Debug)]
pub struct SnippetMap<'c> {
    pub map: HashMap<String, String>,
    config: &'c Opt,
}
impl<'c> SnippetMap<'c> {
    pub fn new(config: &'c Opt) -> Self {
        Self {
            config,
            map: HashMap::new(),
        }
    }
    pub fn collect_entries(&mut self, items: &[Item]) {
        for item in items {
            self.visit_item(item);
        }
    }
    pub fn add_snippet(&mut self, name: &str, item: &Item) {
        if let Some(item) = modify(item.clone(), self.config) {
            self.map
                .entry(name.to_string())
                .or_default()
                .push_str(&item.to_token_stream().to_string());
        }
    }
    pub fn into_vscode(self) -> BTreeMap<String, VSCode> {
        self.map
            .into_iter()
            .map(|kv| (kv.0.to_owned(), From::from(kv)))
            .collect::<BTreeMap<_, _>>()
    }
}

impl Visit<'_> for SnippetMap<'_> {
    fn visit_item(&mut self, item: &Item) {
        let sa = SnippetAttributes::from(item);
        if sa.contains_entry {
            if let Some(name) = sa.name {
                if sa.inline {
                    if let Item::Mod(syn::ItemMod {
                        attrs,
                        content: Some((_, items)),
                        ..
                    }) = item
                    {
                        if !is_skip(attrs, self.config) {
                            for item in items {
                                self.add_snippet(&name, item);
                            }
                        }
                    } else {
                        self.add_snippet(&name, item);
                    }
                } else {
                    self.add_snippet(&name, item);
                }
            }
        }
        visit::visit_item(self, item);
    }
}

fn is_skip(attrs: &[Attribute], config: &Opt) -> bool {
    attrs
        .iter()
        .filter_map(|attr| attr.parse_meta().ok())
        .any(|meta| config.filter_item.iter().any(|pat| pat == meta.path()))
}

fn modify(mut item: Item, config: &Opt) -> Option<Item> {
    if let Some(attrs) = get_attributes_of_item(&item) {
        if is_skip(attrs, config) {
            return None;
        }
    }

    if let Some(attrs) = get_attributes_of_item_mut(&mut item) {
        attrs.retain(|attr| {
            !attr
                .parse_meta()
                .map(|meta| {
                    is_snippet(meta.path())
                        || config.filter_attr.iter().any(|pat| pat == meta.path())
                })
                .unwrap_or_default()
        })
    }

    if let Item::Mod(syn::ItemMod {
        content: Some((_, items)),
        ..
    }) = &mut item
    {
        *items = items
            .iter()
            .filter_map(|item| modify(item.clone(), config))
            .collect::<Vec<_>>();
    }

    Some(item)
}
