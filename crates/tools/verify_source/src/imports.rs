use crate::symbols::{Symbol, Symbols, imports};
use anyhow::{Context as _, Result, ensure};
use codesnip_core::parse_file_recursive;
use quote::ToTokens as _;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Range,
    path::{Path, PathBuf},
    process::Command,
};
use syn::{
    Item, ItemUse, Meta, Token, UseTree, parse_quote, punctuated::Punctuated,
    spanned::Spanned as _, visit::Visit, visit_mut::VisitMut,
};

pub struct ImportResolver {
    library: PathBuf,
    host_dependencies: PathBuf,
    cfg: Vec<Meta>,
}

pub struct ResolvedImports {
    pub imports: Vec<ItemUse>,
    pub exports: BTreeMap<Vec<String>, Symbol>,
}

impl ImportResolver {
    pub fn new(root: &Path) -> Result<Self> {
        let mut command = Command::new("rustup");
        command
            .args([
                "run",
                "1.89.0",
                "cargo",
                "build",
                "--release",
                "-p",
                "competitive",
                "--lib",
                "--message-format=json",
                "--target",
                crate::TARGET,
                "--target-dir",
            ])
            .arg(root.join("target/verify_source"))
            .env_remove("RUSTC")
            .env("RUSTC_WRAPPER", "")
            .env("CARGO_ENCODED_RUSTFLAGS", "")
            .current_dir(root);
        let output = command.output()?;
        let messages: Vec<serde_json::Value> = String::from_utf8(output.stdout)?
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
        ensure!(
            output.status.success(),
            "building competitive for {} failed:\n{}{}",
            crate::TARGET,
            messages
                .iter()
                .filter_map(|message| message["message"]["rendered"].as_str())
                .collect::<String>(),
            String::from_utf8_lossy(&output.stderr)
        );
        let library = messages
            .iter()
            .filter(|message| {
                message["reason"] == "compiler-artifact"
                    && message["target"]["name"] == "competitive"
            })
            .find_map(|message| {
                message["filenames"]
                    .as_array()?
                    .iter()
                    .filter_map(|path| path.as_str())
                    .find(|path| path.ends_with(".rlib"))
                    .map(PathBuf::from)
            })
            .context("missing competitive library artifact")?;
        let output = crate::submission_compiler()
            .args(["--print=cfg", "--target", crate::TARGET])
            .output()?;
        ensure!(
            output.status.success(),
            "reading submission cfg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let cfg = String::from_utf8(output.stdout)?
            .lines()
            .map(syn::parse_str)
            .collect::<syn::Result<_>>()?;
        Ok(Self {
            library,
            host_dependencies: root.join("target/verify_source/release/deps"),
            cfg,
        })
    }

    pub fn resolve(
        &self,
        items: &[Item],
        solution: &str,
        symbols: &Symbols,
    ) -> Result<ResolvedImports> {
        let mut code = String::new();
        for item in items {
            if let Item::Use(import) = item {
                code.push_str(&import.to_token_stream().to_string());
                code.push('\n');
            }
        }
        code.push_str(solution);
        let used = self
            .used_imports(&code, "competitive")
            .context("checking original imports failed")?;
        let mut selected = Vec::new();
        for item in syn::parse_file(&code)?.items {
            if let Item::Use(mut import) = item
                && retain_tree(&mut import.tree, &used)
            {
                selected.push(import);
            }
        }

        // An unused explicit import still marks its reexport as used.
        // Resolve reexports after removing unused imports.
        let mut exports = symbols.namespace_paths();
        let mut code = format!(
            "extern crate self as competitive;\n{}\nmod submission {{\n",
            reference_source(&exports),
        );
        for import in &selected {
            code.push_str(&import.to_token_stream().to_string());
            code.push('\n');
        }
        code.push_str(solution);
        code.push_str("\n}");
        let mut file = syn::parse_file(&code)?;
        CrateVisibility.visit_file_mut(&mut file);
        let code = file.to_token_stream().to_string();
        let used = self
            .used_imports(&code, "__verify_library")
            .context("checking library reexports failed")?;
        let mut referenced = ReferencedExports {
            used: &used,
            paths: BTreeSet::new(),
        };
        referenced.visit_file(&syn::parse_file(&code)?);
        exports.retain(|path, _| referenced.paths.contains(path));
        Ok(ResolvedImports {
            imports: selected,
            exports,
        })
    }

    fn used_imports(&self, code: &str, name: &str) -> Result<BTreeSet<usize>> {
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("imports.rs");
        fs::write(&path, code)?;
        let output = crate::submission_compiler()
            .args([
                "--crate-type=lib",
                "--emit=metadata",
                "--error-format=json",
                "-A",
                "warnings",
                "--force-warn",
                "unused_imports",
                "--target",
                crate::TARGET,
            ])
            .arg("--extern")
            .arg(format!("{name}={}", self.library.display()))
            .arg("-L")
            .arg(format!(
                "dependency={}",
                self.library.parent().unwrap().join("deps").display()
            ))
            .arg("-L")
            .arg(format!("dependency={}", self.host_dependencies.display()))
            .arg(&path)
            .arg("-o")
            .arg(temporary.path().join("imports.rmeta"))
            .output()?;
        let diagnostics: Vec<serde_json::Value> = String::from_utf8(output.stderr)?
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
        ensure!(
            output.status.success(),
            "{}:\n{}",
            crate::TARGET,
            diagnostics
                .iter()
                .filter_map(|message| message["rendered"].as_str())
                .collect::<String>()
        );
        let mut unused = Vec::new();
        for diagnostic in diagnostics
            .iter()
            .filter(|message| message["code"]["code"] == "unused_imports")
        {
            for span in diagnostic["spans"].as_array().into_iter().flatten() {
                if span["is_primary"] == true && span["file_name"].as_str() == path.to_str() {
                    let start = span["byte_start"].as_u64().context("missing span start")?;
                    let end = span["byte_end"].as_u64().context("missing span end")?;
                    unused.push(start as usize..end as usize);
                }
            }
        }
        let mut predicates = CfgPredicates::default();
        predicates.visit_file(&syn::parse_file(code)?);
        if let Some(error) = predicates.error {
            return Err(error.into());
        }
        let enabled: BTreeSet<_> = self
            .cfg
            .iter()
            .map(|meta| meta.to_token_stream().to_string())
            .collect();
        let disabled: Vec<_> = predicates
            .atoms
            .into_iter()
            .filter(|(key, _)| !enabled.contains(key))
            .map(|(_, meta)| meta)
            .collect();
        let active = parse_file_recursive(path, &self.cfg, &disabled)?;
        let mut used = UsedImports {
            unused: &unused,
            offsets: BTreeSet::new(),
        };
        used.visit_file(&active);
        Ok(used.offsets)
    }
}

fn reference_source(paths: &BTreeMap<Vec<String>, Symbol>) -> String {
    #[derive(Default)]
    struct Namespace<'a> {
        exports: Vec<(&'a [String], &'a Symbol)>,
        children: BTreeMap<&'a str, Namespace<'a>>,
    }
    impl Namespace<'_> {
        fn write(&self, code: &mut String) {
            for (path, symbol) in &self.exports {
                for cfg in &symbol.cfg {
                    code.push_str(cfg);
                }
                code.push_str(&format!(
                    "pub(crate) use ::__verify_library::{};\n",
                    path.join("::"),
                ));
            }
            for (name, child) in &self.children {
                code.push_str(&format!("pub(crate) mod {name} {{\n"));
                child.write(code);
                code.push_str("}\n");
            }
        }
    }
    let mut namespaces = Namespace::default();
    for (path, symbol) in paths {
        let mut namespace = &mut namespaces;
        for part in &path[..path.len() - 1] {
            namespace = namespace.children.entry(part).or_default();
        }
        namespace.exports.push((path, symbol));
    }
    let mut code = String::new();
    namespaces.write(&mut code);
    code
}

fn retain_tree(tree: &mut UseTree, used: &BTreeSet<usize>) -> bool {
    match tree {
        UseTree::Path(path) => retain_tree(&mut path.tree, used),
        UseTree::Group(group) => {
            group.items = std::mem::take(&mut group.items)
                .into_iter()
                .filter_map(|mut tree| retain_tree(&mut tree, used).then_some(tree))
                .collect();
            !group.items.is_empty()
        }
        _ => used.contains(&tree.span().byte_range().start),
    }
}

struct UsedImports<'a> {
    unused: &'a [Range<usize>],
    offsets: BTreeSet<usize>,
}

impl<'ast> Visit<'ast> for UsedImports<'_> {
    fn visit_use_tree(&mut self, tree: &'ast UseTree) {
        if matches!(tree, UseTree::Path(_) | UseTree::Group(_)) {
            syn::visit::visit_use_tree(self, tree);
        } else {
            let range = tree.span().byte_range();
            if !self
                .unused
                .iter()
                .any(|unused| unused.start < range.end && range.start < unused.end)
            {
                self.offsets.insert(range.start);
            }
        }
    }
}

struct ReferencedExports<'a> {
    used: &'a BTreeSet<usize>,
    paths: BTreeSet<Vec<String>>,
}

impl<'ast> Visit<'ast> for ReferencedExports<'_> {
    fn visit_item_use(&mut self, import: &'ast ItemUse) {
        let mut tree = import.tree.clone();
        if retain_tree(&mut tree, self.used) {
            for (path, _) in imports(&tree) {
                if path.first().is_some_and(|name| name == "__verify_library") {
                    self.paths.insert(path[1..].to_vec());
                }
            }
        }
    }
}

struct CrateVisibility;

impl VisitMut for CrateVisibility {
    fn visit_item_use_mut(&mut self, import: &mut ItemUse) {
        if matches!(import.vis, syn::Visibility::Public(_)) {
            import.vis = parse_quote!(pub(crate));
        }
    }
}

#[derive(Default)]
struct CfgPredicates {
    atoms: BTreeMap<String, Meta>,
    error: Option<syn::Error>,
}

impl CfgPredicates {
    fn attribute(&mut self, meta: &Meta) -> syn::Result<()> {
        if let Meta::List(list) = meta {
            if list.path.is_ident("cfg") {
                self.predicate(list.parse_args()?)?;
            } else if list.path.is_ident("cfg_attr") {
                let args = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                let mut args = args.into_iter();
                if let Some(condition) = args.next() {
                    self.predicate(condition)?;
                }
                for attr in args {
                    self.attribute(&attr)?;
                }
            }
        }
        Ok(())
    }

    fn predicate(&mut self, meta: Meta) -> syn::Result<()> {
        if let Meta::List(list) = &meta
            && (list.path.is_ident("all") || list.path.is_ident("any") || list.path.is_ident("not"))
        {
            for predicate in
                list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
            {
                self.predicate(predicate)?;
            }
        } else {
            self.atoms.insert(meta.to_token_stream().to_string(), meta);
        }
        Ok(())
    }
}

impl<'ast> Visit<'ast> for CfgPredicates {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if let Err(error) = self.attribute(&attribute.meta) {
            self.error.get_or_insert(error);
        }
    }
}
