use anyhow::{Result, bail};
use codesnip_core::{AttributeExt as _, ItemExt as _, PathExt as _, entry::EntryArgs};
use quote::ToTokens as _;
use std::collections::{BTreeMap, BTreeSet};
use syn::{Item, UseTree, parse::Parse as _};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub snippet: Option<String>,
    pub path: Vec<String>,
    pub cfg: Vec<String>,
}

#[derive(Default)]
pub struct Symbols {
    definitions: BTreeMap<Vec<String>, Symbol>,
    aliases: BTreeMap<Vec<String>, Alias>,
    globs: BTreeMap<Vec<String>, Vec<Glob>>,
    modules: BTreeMap<Vec<String>, Vec<String>>,
    public: BTreeSet<Vec<String>>,
}

struct Glob {
    target: Vec<String>,
    export: Option<Symbol>,
    public: bool,
    cfg: Vec<String>,
}

struct Alias {
    target: Vec<String>,
    cfg: Vec<String>,
}

impl Symbols {
    pub fn new(items: &[Item]) -> Result<Self> {
        let mut symbols = Self::default();
        symbols.modules.insert(Vec::new(), Vec::new());
        symbols.collect(items, &[], &[], None, &[])?;
        Ok(symbols)
    }

    pub fn resolve(&self, path: &[String]) -> Result<Symbol> {
        for len in (1..=path.len()).rev() {
            if let Some(mut symbol) = self.find(&path[..len], &mut BTreeSet::new()) {
                symbol.path.extend_from_slice(&path[len..]);
                return Ok(symbol);
            }
        }
        bail!("no snippet for competitive::{}", path.join("::"))
    }

    pub fn bundle_path(&self, path: &[String]) -> Result<Vec<String>> {
        let symbol = match self.resolve(path) {
            Ok(symbol) => symbol,
            Err(error) => {
                let Some(module) = self.modules.get(path) else {
                    return Err(error);
                };
                let mut path = vec!["crate".to_owned()];
                path.extend_from_slice(module);
                return Ok(path);
            }
        };
        let mut path = vec![if symbol.snippet.is_some() {
            "crate".to_owned()
        } else {
            String::new()
        }];
        path.extend(symbol.path);
        Ok(path)
    }

    pub fn namespace_paths(&self) -> BTreeMap<Vec<String>, Symbol> {
        let mut output = BTreeMap::new();
        let mut pending = vec![Vec::new()];
        while let Some(module) = pending.pop() {
            let mut names = BTreeSet::new();
            self.exported_names(&module, &mut BTreeSet::new(), &mut names);
            for name in names {
                let mut path = module.clone();
                path.push(name);
                if let Ok(mut symbol) = self.resolve(&path) {
                    if symbol.snippet.is_some() {
                        symbol.path.insert(0, "crate".to_owned());
                    } else {
                        symbol.path.insert(0, String::new());
                    }
                    output.insert(path, symbol);
                } else if self.modules.contains_key(&path) {
                    pending.push(path);
                }
            }
        }
        output
    }

    fn collect(
        &mut self,
        items: &[Item],
        original: &[String],
        bundled: &[String],
        owner: Option<&str>,
        parent_cfg: &[String],
    ) -> Result<()> {
        for item in items {
            let attrs = item.get_attributes().unwrap_or_default();
            if attrs.iter().any(|a| a.path().is_codesnip_skip()) {
                continue;
            }
            let mut cfg = parent_cfg.to_vec();
            cfg.extend(
                attrs
                    .iter()
                    .filter(|a| a.path().is_ident("cfg"))
                    .map(|a| a.to_token_stream().to_string()),
            );
            let entries: Vec<_> = attrs
                .iter()
                .filter(|a| a.path().is_codesnip_entry())
                .map(|a| {
                    a.parse_args_empty_with(EntryArgs::parse)?
                        .try_to_entry(item)
                })
                .collect::<syn::Result<_>>()?;
            if entries.len() > 1 {
                bail!("multiple snippet entries on {}", original.join("::"));
            }
            let entry = entries.first();
            let owner = entry.and_then(|e| e.name.as_deref()).or(owner);
            let inline = entry.is_some_and(|e| e.inline);
            let visibility = match item {
                Item::Const(i) => Some(&i.vis),
                Item::Enum(i) => Some(&i.vis),
                Item::Fn(i) => Some(&i.vis),
                Item::Mod(i) => Some(&i.vis),
                Item::Static(i) => Some(&i.vis),
                Item::Struct(i) => Some(&i.vis),
                Item::Trait(i) => Some(&i.vis),
                Item::TraitAlias(i) => Some(&i.vis),
                Item::Type(i) => Some(&i.vis),
                Item::Union(i) => Some(&i.vis),
                Item::Use(i) => Some(&i.vis),
                _ => None,
            };
            let public = matches!(visibility, Some(syn::Visibility::Public(_)));
            if let Some(name) = item.get_default_name() {
                let mut source = original.to_vec();
                source.push(name.clone());
                if public {
                    self.public.insert(source.clone());
                }
                let mut target = bundled.to_vec();
                if !inline {
                    target.push(name.clone());
                }
                if let Some(owner) = owner {
                    let exported_macro = matches!(item, Item::Macro(_))
                        && attrs.iter().any(|a| a.path().is_ident("macro_export"));
                    if exported_macro {
                        source = vec![name.clone()];
                        target = vec![name];
                        self.public.insert(source.clone());
                    }
                    self.definitions.insert(
                        source.clone(),
                        Symbol {
                            snippet: Some(owner.to_owned()),
                            path: target.clone(),
                            cfg: cfg.clone(),
                        },
                    );
                }
                if let Item::Mod(module) = item
                    && let Some((_, children)) = &module.content
                {
                    let bundled = if owner.is_some() { &target } else { bundled };
                    self.modules.insert(source.clone(), bundled.to_vec());
                    self.collect(children, &source, bundled, owner, &cfg)?;
                }
            }
            if let Item::Use(import) = item {
                let export_owner =
                    owner.filter(|_| matches!(import.vis, syn::Visibility::Public(_)));
                for (path, alias) in imports(&import.tree) {
                    let target = absolute(original, &path);
                    if let Some(alias) = alias {
                        let mut source = original.to_vec();
                        source.push(alias.clone());
                        if public {
                            self.public.insert(source.clone());
                        }
                        if let Some(owner) = export_owner {
                            let mut path = bundled.to_vec();
                            path.push(alias);
                            self.definitions.insert(
                                source,
                                Symbol {
                                    snippet: Some(owner.to_owned()),
                                    path,
                                    cfg: cfg.clone(),
                                },
                            );
                        } else if matches!(
                            path.first().map(String::as_str),
                            Some("std" | "core" | "alloc")
                        ) {
                            self.definitions.insert(
                                source,
                                Symbol {
                                    snippet: None,
                                    path,
                                    cfg: cfg.clone(),
                                },
                            );
                        } else {
                            self.aliases.insert(
                                source,
                                Alias {
                                    target,
                                    cfg: cfg.clone(),
                                },
                            );
                        }
                    } else {
                        let export = export_owner.map(|owner| Symbol {
                            snippet: Some(owner.to_owned()),
                            path: bundled.to_vec(),
                            cfg: cfg.clone(),
                        });
                        self.globs.entry(original.to_vec()).or_default().push(Glob {
                            target,
                            export,
                            public,
                            cfg: cfg.clone(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn exported_names(
        &self,
        path: &[String],
        visited: &mut BTreeSet<Vec<String>>,
        names: &mut BTreeSet<String>,
    ) {
        if !visited.insert(path.to_vec()) {
            return;
        }
        for source in &self.public {
            if source.len() == path.len() + 1 && source.starts_with(path) {
                names.insert(source.last().unwrap().clone());
            }
        }
        if let Some(globs) = self.globs.get(path) {
            for glob in globs.iter().filter(|glob| glob.public) {
                self.exported_names(&glob.target, visited, names);
            }
        }
    }

    fn find(&self, path: &[String], visited: &mut BTreeSet<Vec<String>>) -> Option<Symbol> {
        if !visited.insert(path.to_vec()) {
            return None;
        }
        if let Some(symbol) = self.definitions.get(path) {
            return Some(symbol.clone());
        }
        for split in (1..=path.len()).rev() {
            if let Some(alias) = self.aliases.get(&path[..split]) {
                let mut target = alias.target.clone();
                target.extend_from_slice(&path[split..]);
                if let Some(mut symbol) = self.find(&target, visited) {
                    symbol.cfg.extend_from_slice(&alias.cfg);
                    symbol.cfg.sort();
                    symbol.cfg.dedup();
                    return Some(symbol);
                }
            }
            if (split == path.len() || !self.modules.contains_key(&path[..split]))
                && let Some(globs) = self.globs.get(&path[..split - 1])
            {
                for glob in globs {
                    let mut target = glob.target.clone();
                    target.extend_from_slice(&path[split - 1..]);
                    if let Some(mut symbol) = self.find(&target, visited) {
                        if let Some(export) = &glob.export {
                            let cfg = symbol.cfg;
                            symbol = export.clone();
                            symbol.cfg.extend(cfg);
                            symbol.path.extend_from_slice(&path[split - 1..]);
                        }
                        symbol.cfg.extend_from_slice(&glob.cfg);
                        symbol.cfg.sort();
                        symbol.cfg.dedup();
                        return Some(symbol);
                    }
                }
            }
        }
        None
    }
}

pub fn imports(tree: &UseTree) -> Vec<(Vec<String>, Option<String>)> {
    fn visit(tree: &UseTree, mut path: Vec<String>, out: &mut Vec<(Vec<String>, Option<String>)>) {
        match tree {
            UseTree::Path(p) => {
                path.push(p.ident.to_string());
                visit(&p.tree, path, out);
            }
            UseTree::Name(n) => {
                if n.ident != "self" {
                    path.push(n.ident.to_string());
                }
                let alias = path.last().cloned();
                out.push((path, alias));
            }
            UseTree::Rename(n) => {
                if n.ident != "self" {
                    path.push(n.ident.to_string());
                }
                out.push((path, Some(n.rename.to_string())));
            }
            UseTree::Glob(_) => out.push((path, None)),
            UseTree::Group(g) => {
                for tree in &g.items {
                    visit(tree, path.clone(), out);
                }
            }
        }
    }
    let mut out = Vec::new();
    visit(tree, Vec::new(), &mut out);
    out
}

fn absolute(module: &[String], path: &[String]) -> Vec<String> {
    let mut target = module.to_vec();
    let mut start = 0;
    for part in path {
        match part.as_str() {
            "crate" => target.clear(),
            "self" => {}
            "super" => {
                target.pop();
            }
            _ => break,
        }
        start += 1;
    }
    target.extend_from_slice(&path[start..]);
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_reexports_and_paths_inside_snippets() {
        let file = syn::parse_file(
            r#"
            mod num {
                #[codesnip::entry("unrelated")]
                pub use self::unrelated::*;
                #[codesnip::entry("unrelated")]
                mod unrelated { pub struct Other; }
                pub use self::mint::*;
                mod mint {
                    #[codesnip::entry("mint")]
                    pub mod modular { define_moduli!(M998, M1007); }
                }
            }
            mod algebra {
                #[codesnip::entry("ring")]
                pub use self::ring::*;
                #[codesnip::entry("ring")]
                mod ring { pub trait Ring {} }
            }
        "#,
        )
        .unwrap();
        let symbols = Symbols::new(&file.items).unwrap();
        let path = |s: &str| s.split("::").map(str::to_owned).collect::<Vec<_>>();
        assert_eq!(
            symbols.resolve(&path("num::modular::M998")).unwrap(),
            Symbol {
                snippet: Some("mint".into()),
                path: path("modular::M998"),
                cfg: Vec::new(),
            }
        );
        let ring = symbols.resolve(&path("algebra::Ring")).unwrap();
        assert_eq!(ring.path, path("Ring"));
        assert!(symbols.resolve(&path("num::missing::Missing")).is_err());
    }
}
