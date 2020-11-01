use crate::{entry::EntryArgs, format_with_rustfmt, AttributeExt as _, ItemExt as _, PathExt as _};
use quote::ToTokens as _;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use syn::{
    parse::Parse as _,
    visit::{self, Visit},
    Attribute, Item, ItemMod, Path,
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
    filter_attr: &'a [Path],
    filter_item: &'i [Path],
}

struct CollectEntries<'m, 'i, 'a> {
    map: &'m mut SnippetMap,
    filter: Filter<'i, 'a>,
}

impl SnippetMap {
    pub fn new() -> Self {
        Default::default()
    }
    fn get_mut(&mut self, name: &str) -> &mut LinkedSnippet {
        if !self.map.contains_key(name) {
            self.map.insert(name.to_string(), Default::default());
        }
        self.map
            .get_mut(name)
            .expect("HashMap is not working properly.")
    }
    pub fn extend_with_filter(&mut self, item: &Item, filter: Filter) {
        CollectEntries { map: self, filter }.visit_item(item);
    }
    fn resolve_includes<'s>(
        &'s self,
        used: &BTreeSet<&'s str>,
        includes: impl IntoIterator<Item = &'s str>,
    ) -> BTreeSet<&'s str> {
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
        visited
    }
    pub fn bundle<'s>(
        &self,
        name: &'s str,
        link: &LinkedSnippet,
        mut excludes: BTreeSet<&'s str>,
        guard: bool,
    ) -> String {
        fn push_guard(contents: &mut String, name: &str) {
            contents.push_str("// codesnip-guard: ");
            contents.push_str(name);
            contents.push('\n');
        }

        if excludes.contains(name) {
            return Default::default();
        }
        excludes.insert(name);
        let visited = self.resolve_includes(&excludes, link.includes.iter().map(|s| s.as_str()));
        let mut contents = String::new();
        for include in visited.difference(&excludes).cloned() {
            if guard {
                push_guard(&mut contents, include);
            }
            if let Some(nlink) = self.map.get(include) {
                contents.push_str(nlink.contents.as_str());
            }
        }
        if guard {
            push_guard(&mut contents, name);
        }
        contents.push_str(link.contents.as_str());
        contents
    }
}

impl IntoIterator for SnippetMap {
    type Item = (String, LinkedSnippet);
    type IntoIter = <HashMap<String, LinkedSnippet> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl Extend<(String, LinkedSnippet)> for SnippetMap {
    fn extend<T: IntoIterator<Item = (String, LinkedSnippet)>>(&mut self, iter: T) {
        for (name, link) in iter {
            self.map.entry(name).or_default().append(link);
        }
    }
}

impl LinkedSnippet {
    pub fn push_contents(&mut self, contents: &str) {
        self.contents.push_str(contents);
    }
    pub fn push_item_with_filter(&mut self, item: &Item, filter: Filter) {
        if let Some(item) = filter.modify_item(item.clone()) {
            self.contents
                .push_str(&item.into_token_stream().to_string());
        }
    }
    pub fn push_include(&mut self, include: String) {
        self.includes.insert(include);
    }
    pub fn push_includes(&mut self, includes: impl IntoIterator<Item = String>) {
        self.includes.extend(includes);
    }
    pub fn append(&mut self, mut other: Self) {
        self.contents.push_str(&other.contents);
        self.includes.append(&mut other.includes);
    }
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
    pub fn new(filter_attr: &'a [Path], filter_item: &'i [Path]) -> Self {
        Self {
            filter_attr,
            filter_item,
        }
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
                let link = self.map.get_mut(&entry.name);
                let filter = self.filter;
                match (entry.inline, item) {
                    (true, Item::Mod(ItemMod { attrs, content, .. })) => {
                        if !filter.is_skip_item(attrs) {
                            if let Some((_, items)) = content {
                                for item in items {
                                    link.push_item_with_filter(item, filter);
                                }
                            }
                        }
                    }
                    _ => link.push_item_with_filter(item, filter),
                }
                link.push_includes(entry.include);
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

        if let Item::Mod(ItemMod {
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
