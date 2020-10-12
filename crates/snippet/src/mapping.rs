use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;
use snippet_core::{format::rustfmt_exits, map::Filter, map::SnippetMap};
use std::collections::{BTreeMap, BTreeSet};
use syn::Item;

pub trait SnippetMapExt {
    fn collect_entries(&mut self, items: &[Item], filter: Filter);
    fn format_all(&mut self);
    fn to_vscode(&self) -> BTreeMap<String, VSCode>;
}

#[derive(Serialize)]
pub struct VSCode {
    prefix: String,
    body: String,
    scope: String,
}
impl From<(String, String)> for VSCode {
    fn from((prefix, contents): (String, String)) -> Self {
        Self {
            prefix,
            body: contents.replace("$", "\\$"),
            scope: "rust".to_string(),
        }
    }
}

impl SnippetMapExt for SnippetMap {
    fn collect_entries(&mut self, items: &[Item], filter: Filter) {
        let pb = ProgressBar::new(items.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{prefix:>12.green} [{bar:57}] {pos}/{len}")
                .progress_chars("=> "),
        );
        pb.set_prefix("Collecting");
        for item in items {
            self.extend_with_filter(item, filter);
            pb.inc(1);
        }
        pb.finish_and_clear();
    }
    fn format_all(&mut self) {
        if !rustfmt_exits() {
            eprintln!("warning: rustfmt not found.");
            return;
        }
        let pb = ProgressBar::new(self.map.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{prefix:>12.green} [{bar:57}] {pos}/{len}: {msg}")
                .progress_chars("=> "),
        );
        pb.set_prefix("Formatting");
        self.map.par_iter_mut().for_each(|(name, link)| {
            pb.set_message(name);
            if !link.format() {
                pb.println(format!("warning: Failed to format `{}`.", name));
            }
            pb.inc(1);
        });
        pb.finish_and_clear();
    }
    fn to_vscode(&self) -> BTreeMap<String, VSCode> {
        let mut res: BTreeMap<&str, String> = BTreeMap::new();
        for (name, link) in self.map.iter() {
            let mut used = BTreeSet::new();
            used.insert(name.as_str());
            let mut contents =
                self.resolve_includes(used, link.includes.iter().map(|s| s.as_str()));
            contents.push_str(link.contents.as_str());
            res.insert(name.as_str(), contents);
        }
        res.into_iter()
            .filter(|(k, _)| !k.starts_with('_'))
            .map(|(k, v)| (k.to_owned(), From::from((k.to_owned(), v))))
            .collect::<BTreeMap<_, _>>()
    }
}
