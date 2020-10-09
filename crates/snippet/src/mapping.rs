use crate::{
    ast_helper::ItemExt as _,
    attribute::{is_snippet, SnippetAttributes},
    config::Config,
    output::{format_with_rustfmt, rustfmt_exits, VSCode},
};
use indicatif::{ProgressBar, ProgressStyle};
use quote::ToTokens as _;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use syn::{
    visit::{self, Visit},
    Attribute, Item,
};

#[derive(Debug)]
pub struct SnippetMap<'c> {
    map: HashMap<String, LinkedSnippet>,
    config: &'c Config,
}

#[derive(Debug, Default)]
struct LinkedSnippet {
    pub contents: String,
    includes: BTreeSet<String>,
}

impl<'c> SnippetMap<'c> {
    pub fn new(config: &'c Config) -> Self {
        Self {
            config,
            map: HashMap::new(),
        }
    }
}

impl SnippetMap<'_> {
    pub fn collect_entries(&mut self, items: &[Item]) {
        let pb = ProgressBar::new(items.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  Collecting [{bar:57.cyan/blue}] {pos}/{len}")
                .progress_chars("=> "),
        );
        for item in items {
            self.visit_item(item);
            pb.inc(1);
        }
        pb.finish_and_clear();
    }
    fn get_mut(&mut self, name: &str) -> &mut LinkedSnippet {
        if !self.map.contains_key(name) {
            self.map.insert(name.to_string(), Default::default());
        }
        self.map
            .get_mut(name)
            .expect("HashMap is not working properly.")
    }
    fn add_snippet(&mut self, name: &str, item: &Item) {
        if let Some(item) = modify(item.clone(), self.config) {
            self.get_mut(name)
                .contents
                .push_str(&item.to_token_stream().to_string());
        }
    }
    fn add_include(&mut self, name: &str, include: String) {
        self.get_mut(name).includes.insert(include);
    }
    pub fn format_all(&mut self) {
        if !rustfmt_exits() {
            eprintln!("warning: rustfmt not found.");
            return;
        }
        let pb = ProgressBar::new(self.map.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  Formatting [{bar:57.cyan/blue}] {pos}/{len}: {msg}")
                .progress_chars("=> "),
        );
        self.map.par_iter_mut().for_each(|(name, link)| {
            pb.set_message(name);
            if let Some(formatted) = format_with_rustfmt(&link.contents) {
                link.contents = formatted;
            } else {
                pb.println(format!("warning: Failed to format `{}`.", name));
            }
            pb.inc(1);
        });
        pb.finish_and_clear();
    }
    fn resolve_include(&self) -> BTreeMap<&str, String> {
        let mut res: BTreeMap<_, String> = BTreeMap::new();
        for (name, link) in self.map.iter() {
            let mut used = BTreeSet::new();
            used.insert(name.as_str());
            let mut stack: Vec<_> = link.includes.iter().map(|s| s.as_str()).collect();
            used.extend(&stack);
            while let Some(include) = stack.pop() {
                if let Some(nlink) = self.map.get(include) {
                    for ninclude in nlink.includes.iter().map(|s| s.as_str()) {
                        if !used.contains(ninclude) {
                            used.insert(ninclude);
                            stack.push(ninclude);
                        }
                    }
                }
            }
            let entry = res.entry(name.as_str()).or_default();
            used.remove(name.as_str());
            for include in used {
                if let Some(nlink) = self.map.get(include) {
                    entry.push_str(nlink.contents.as_str());
                }
            }
            entry.push_str(link.contents.as_str());
        }
        res
    }
    pub fn to_vscode(&self) -> BTreeMap<String, VSCode> {
        let res = self.resolve_include();
        res.into_iter()
            .filter(|(k, _)| !k.starts_with('_'))
            .map(|(k, v)| (k.to_owned(), From::from((k.to_owned(), v))))
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
                for include in sa.include {
                    self.add_include(&name, include);
                }
            }
        }
        visit::visit_item(self, item);
    }
}

fn is_skip(attrs: &[Attribute], config: &Config) -> bool {
    attrs
        .iter()
        .filter_map(|attr| attr.parse_meta().ok())
        .any(|meta| config.filter_item.iter().any(|pat| pat == meta.path()))
}

fn modify(mut item: Item, config: &Config) -> Option<Item> {
    if let Some(attrs) = item.get_attributes() {
        if is_skip(attrs, config) {
            return None;
        }
    }

    if let Some(attrs) = item.get_attributes_mut() {
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
