use crate::{
    entry::EntryArgs,
    ext::{AttributeExt as _, ItemExt as _, PathExt as _},
    format::format_with_rustfmt,
};
use quote::ToTokens as _;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use syn::{
    parse::Parse as _,
    visit::{self, Visit},
    Attribute, Item,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SnippetMap {
    pub map: HashMap<String, LinkedSnippet>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LinkedSnippet {
    pub contents: String,
    pub includes: BTreeSet<String>,
}

#[derive(Debug, Copy, Clone)]
pub struct Filter<'a, 'i> {
    filter_attr: &'a [syn::Path],
    filter_item: &'i [syn::Path],
}

struct CollectEntries<'m, 'i, 'a> {
    map: &'m mut SnippetMap,
    filter: Filter<'i, 'a>,
}

impl SnippetMap {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn extend_with_filter(&mut self, item: &Item, filter: Filter) {
        CollectEntries { map: self, filter }.visit_item(item);
    }
    pub fn resolve_includes<'s>(
        &'s self,
        used: BTreeSet<&'s str>,
        includes: impl IntoIterator<Item = &'s str>,
    ) -> String {
        let mut visited = used.clone();
        let mut stack: Vec<_> = includes.into_iter().collect();
        visited.extend(&stack);
        while let Some(include) = stack.pop() {
            if let Some(nlink) = self.map.get(include) {
                for ninclude in nlink.includes.iter().map(|s| s.as_str()) {
                    if !visited.contains(ninclude) {
                        visited.insert(ninclude);
                        stack.push(ninclude);
                    }
                }
            }
        }
        let mut res = String::new();
        for include in visited.difference(&used).cloned() {
            if let Some(nlink) = self.map.get(include) {
                res.push_str(nlink.contents.as_str());
            }
        }
        res
    }
}

impl LinkedSnippet {
    pub fn format(&mut self) -> bool {
        if let Some(formatted) = format_with_rustfmt(&self.contents) {
            self.contents = formatted;
            true
        } else {
            false
        }
    }
}

impl<'a, 'i> Filter<'a, 'i> {
    pub fn new(filter_attr: &'a [syn::Path], filter_item: &'i [syn::Path]) -> Self {
        Self {
            filter_attr,
            filter_item,
        }
    }
}

impl CollectEntries<'_, '_, '_> {
    fn get_mut(&mut self, name: &str) -> &mut LinkedSnippet {
        if !self.map.map.contains_key(name) {
            self.map.map.insert(name.to_string(), Default::default());
        }
        self.map
            .map
            .get_mut(name)
            .expect("HashMap is not working properly.")
    }
    fn add_snippet(&mut self, name: &str, item: &Item) {
        if let Some(item) = self.filter.modify_item(item.clone()) {
            self.get_mut(name)
                .contents
                .push_str(&item.to_token_stream().to_string());
        }
    }
    fn add_include(&mut self, name: &str, include: String) {
        self.get_mut(name).includes.insert(include);
    }
}

impl Visit<'_> for CollectEntries<'_, '_, '_> {
    fn visit_item(&mut self, item: &Item) {
        if let Some(attrs) = item.get_attributes() {
            for entry in attrs
                .iter()
                .filter(|attr| attr.path.is_codesnip_entry())
                .filter_map(|attr| attr.parse_args_empty_with(EntryArgs::parse).ok())
                .filter_map(|args| args.try_to_entry(item).ok())
            {
                if entry.inline {
                    if let Item::Mod(syn::ItemMod {
                        attrs,
                        content: Some((_, items)),
                        ..
                    }) = item
                    {
                        if !self.filter.is_skip_item(attrs) {
                            for item in items {
                                self.add_snippet(&entry.name, item);
                            }
                        }
                    } else {
                        self.add_snippet(&entry.name, item);
                    }
                } else {
                    self.add_snippet(&entry.name, item);
                }
                for include in entry.include {
                    self.add_include(&entry.name, include);
                }
            }
        }
        visit::visit_item(self, item);
    }
}

impl Filter<'_, '_> {
    fn is_skip_item(self, attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path.is_codesnip_skip() || self.filter_item.iter().any(|pat| pat == &attr.path)
        })
    }

    fn filter_attributes(self, attrs: &mut Vec<Attribute>) {
        attrs.retain(|attr| {
            !(attr.path.is_codesnip_entry() || self.filter_attr.iter().any(|pat| pat == &attr.path))
        })
    }

    fn modify_item(self, mut item: Item) -> Option<Item> {
        if let Some(attrs) = item.get_attributes() {
            if self.is_skip_item(attrs) {
                return None;
            }
        }

        if let Some(attrs) = item.get_attributes_mut() {
            self.filter_attributes(attrs);
        }

        if let Item::Mod(syn::ItemMod {
            content: Some((_, items)),
            ..
        }) = &mut item
        {
            *items = items
                .drain(..)
                .filter_map(|item| self.modify_item(item))
                .collect::<Vec<_>>();
        }

        Some(item)
    }
}
