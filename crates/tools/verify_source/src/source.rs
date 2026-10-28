use crate::symbols::{Symbol, Symbols, imports};
use anyhow::{Context as _, Result, bail, ensure};
use codesnip_core::{Filter, ItemExt as _, PathExt as _, SnippetMap, entry::EntryArg};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens as _;
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};
use syn::{
    Item, ItemFn, ItemUse, Meta, Stmt, Token, parse_quote, punctuated::Punctuated,
    spanned::Spanned as _, visit::Visit, visit_mut::VisitMut,
};

pub fn verification(function: &ItemFn) -> Result<Option<Vec<String>>> {
    fn find(meta: &Meta) -> Result<Option<Vec<String>>> {
        let path = meta.path().to_token_stream().to_string();
        if matches!(
            path.as_str(),
            "verify :: library_checker" | "verify :: aizu_online_judge"
        ) {
            let Meta::List(list) = meta else {
                bail!("missing verify arguments");
            };
            let args =
                list.parse_args_with(Punctuated::<syn::Expr, Token![,]>::parse_terminated)?;
            let mut includes = None;
            for arg in args {
                let name = match &arg {
                    syn::Expr::Call(call) => &call.func,
                    syn::Expr::Assign(assign) => &assign.left,
                    _ => continue,
                };
                if let syn::Expr::Path(path) = &**name
                    && path.path.is_ident("include")
                {
                    let EntryArg::Include(arg) = syn::parse2(arg.to_token_stream())? else {
                        unreachable!()
                    };
                    ensure!(includes.is_none(), "duplicate include");
                    includes = Some(
                        arg.includes
                            .iter()
                            .map(|name| name.litstr.value())
                            .collect(),
                    );
                }
            }
            return Ok(Some(includes.unwrap_or_default()));
        }
        if meta.path().is_ident("cfg_attr")
            && let Meta::List(list) = meta
        {
            let args = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for attr in args.iter().skip(1) {
                if let Some(includes) = find(attr)? {
                    return Ok(Some(includes));
                }
            }
        }
        Ok(None)
    }
    for attr in &function.attrs {
        if let Some(includes) = find(&attr.meta)? {
            return Ok(Some(includes));
        }
    }
    Ok(None)
}

pub fn generate(
    items: &[Item],
    selected: &ItemFn,
    symbols: &Symbols,
    snippets: &SnippetMap,
    resolver: &crate::imports::ImportResolver,
) -> Result<String> {
    let block = &selected.block;
    ensure!(
        selected.sig.inputs.len() == 2 && selected.sig.generics.params.is_empty(),
        "unsupported verify signature"
    );
    let Some(Stmt::Macro(first)) = block.stmts.first() else {
        bail!("verify must start with prepare_io!");
    };
    ensure!(
        first.mac.path.is_ident("prepare_io"),
        "verify must start with prepare_io!"
    );
    let parameters = first.mac.tokens.to_string();
    ensure!(
        matches!(
            parameters.as_str(),
            "reader , writer" | "buffered ; reader , writer"
        ),
        "unsupported prepare_io!: {parameters}"
    );
    let mut dependencies = verification(selected)?.context("not a verify function")?;
    dependencies.push("fast_main".into());
    let mut local = SnippetMap::new();
    let mut helpers = Vec::new();
    for item in items {
        if !item
            .get_attributes()
            .unwrap_or_default()
            .iter()
            .any(|a| a.path().is_codesnip_entry())
        {
            continue;
        }
        let mut entries = SnippetMap::new();
        entries.extend_with_filter(item, Filter::new(&[parse_quote!(doc)], &[]))?;
        if entries.map.is_empty() {
            continue;
        }
        ensure!(
            !matches!(item, Item::Mod(_) | Item::Use(_)),
            "submission entries must be on declarations, not modules or imports"
        );
        let owners = entries
            .map
            .iter()
            .filter(|(_, entry)| !entry.contents.is_empty())
            .map(|(name, _)| name.clone())
            .collect::<BTreeSet<_>>();
        helpers.push((owners, item));
        for entry in entries.map.values_mut() {
            entry.contents.clear();
        }
        local.extend(entries)?;
    }
    for name in local.map.keys() {
        ensure!(
            !snippets.map.contains_key(name),
            "local submission snippet conflicts with library snippet: {name}"
        );
    }
    let mut combined = snippets.clone();
    combined.extend(local)?;
    let roots: Vec<_> = dependencies.iter().map(String::as_str).collect();
    // Use codesnip's resolved entries, including conditional dependencies.
    let bundled = combined.bundle(&roots, BTreeSet::new(), true)?;
    let included: BTreeSet<_> = bundled
        .lines()
        .filter_map(|line| line.strip_prefix("// codesnip-guard: "))
        .map(str::to_owned)
        .collect();
    helpers.retain(|(owners, _)| !owners.is_disjoint(&included));
    let mut before = String::new();
    let mut after = String::new();
    for (_, item) in helpers {
        let source = replace_source(
            item.span(),
            item.get_attributes()
                .unwrap_or_default()
                .iter()
                .filter(|a| a.path().is_codesnip_entry() || a.path().is_ident("doc"))
                .map(|a| (a.span().byte_range(), String::new()))
                .collect(),
        )?;
        let output = if item.span().byte_range().start < selected.span().byte_range().start {
            &mut before
        } else {
            &mut after
        };
        output.push_str(&source);
        output.push('\n');
    }
    let native_body = replace_source(
        block.span(),
        vec![(
            first.span().byte_range(),
            "::competitive::prepare!(fast);".into(),
        )],
    )?;
    let native = format!("{before}\npub fn solve() {native_body}\n{after}");
    let resolved = resolver.resolve(items, &native, symbols)?;
    let mut scope = CrateScope::default();
    scope.visit_file(&syn::parse_file(&native)?);
    for import in &resolved.imports {
        scope.visit_item_use(import);
    }
    ensure!(
        !scope.shadowed || !scope.unqualified,
        "cannot relocate unqualified competitive paths when that name may be locally bound"
    );
    for (path, symbol) in &resolved.exports {
        if let Some(owner) = &symbol.snippet {
            ensure!(
                included.contains(owner),
                "missing submission snippet for competitive::{}: {owner}",
                path.join("::")
            );
        }
    }
    let body = replace_source(
        block.span(),
        vec![(first.span().byte_range(), "crate::prepare!(fast);".into())],
    )?;
    let mut uses = String::new();
    for import in &resolved.imports {
        uses.push_str(&bundle_import(import, symbols, &resolved.exports, true)?);
        uses.push('\n');
    }
    let solution = format!("pub fn solve() {body}\n{uses}\n{before}\n{after}");
    let file = syn::parse_file(&solution)?;
    let mut paths = BundlePaths {
        symbols,
        exports: &resolved.exports,
        replacements: Vec::new(),
        error: None,
    };
    paths.visit_file(&file);
    if let Some(error) = paths.error {
        return Err(error);
    }
    let code = replace_source(file.span(), paths.replacements)?;
    let code = codesnip_core::FormatOption::Rustfmt
        .format(&code, "2024")
        .context("formatting solution failed")?;
    let solve_end = syn::parse_file(&code)?.items[0].span().byte_range().end;
    let code = format!("{}\n{}", &code[..solve_end], code[solve_end..].trim_start());
    let main = snippets.map.get("main").context("missing main snippet")?;
    let dependencies: Vec<_> = included
        .iter()
        .filter(|name| snippets.map.contains_key(*name))
        .map(String::as_str)
        .collect();
    let library = snippets.bundle(&dependencies, BTreeSet::from(["main"]), false)?;
    let code = format!(
        "{}\n{}\n{}\n",
        code.trim_end(),
        main.contents.trim(),
        library.trim(),
    );
    Ok(code)
}

fn bundle_import(
    import: &ItemUse,
    symbols: &Symbols,
    exports: &BTreeMap<Vec<String>, Symbol>,
    root: bool,
) -> Result<String> {
    match &import.tree {
        syn::UseTree::Group(group) => {
            let mut output = String::new();
            for tree in &group.items {
                let mut import = import.clone();
                import.tree = tree.clone();
                output.push_str(&bundle_import(&import, symbols, exports, root)?);
            }
            return Ok(output);
        }
        syn::UseTree::Path(path) if path.ident == "competitive" => {}
        _ => return Ok(format!("{}\n", import.to_token_stream())),
    }
    let mut output = String::new();
    for (path, alias) in imports(&import.tree) {
        let mut targets = Vec::new();
        let target = symbols.bundle_path(&path[1..])?;
        if let Some(alias) = alias {
            targets.push((target, Some(alias)));
        } else if target == ["crate"] {
            for (source, symbol) in exports {
                if source.starts_with(&path[1..]) && source.len() == path.len() {
                    targets.push((symbol.path.clone(), source.last().cloned()));
                }
            }
        } else {
            targets.push((target, None));
        }
        for (path, alias) in targets {
            if root
                && path.first().is_some_and(|name| name == "crate")
                && path.len() == 2
                && (alias.as_deref() == path.last().map(String::as_str)
                    || alias.as_deref() == Some("_"))
            {
                continue;
            }
            let mut item = import.clone();
            let mut tree = path.join("::");
            if let Some(alias) = alias {
                if path.last() != Some(&alias) {
                    tree.push_str(&format!(" as {alias}"));
                }
            } else {
                tree.push_str("::*");
            }
            let parsed: ItemUse = syn::parse_str(&format!("use {tree};"))?;
            item.leading_colon = parsed.leading_colon;
            item.tree = parsed.tree;
            output.push_str(&item.to_token_stream().to_string());
            output.push('\n');
        }
    }
    Ok(output)
}

struct BundlePaths<'a> {
    symbols: &'a Symbols,
    exports: &'a BTreeMap<Vec<String>, Symbol>,
    replacements: Vec<(Range<usize>, String)>,
    error: Option<anyhow::Error>,
}

#[derive(Default)]
struct CrateScope {
    shadowed: bool,
    unqualified: bool,
}

impl<'ast> Visit<'ast> for CrateScope {
    fn visit_item(&mut self, item: &'ast Item) {
        self.shadowed |= item.get_default_name().as_deref() == Some("competitive");
        syn::visit::visit_item(self, item);
    }

    fn visit_item_use(&mut self, import: &'ast ItemUse) {
        for (path, alias) in imports(&import.tree) {
            self.shadowed |= alias.as_deref() == Some("competitive");
            self.unqualified |= import.leading_colon.is_none()
                && path.first().is_some_and(|name| name == "competitive");
        }
    }

    fn visit_type_param(&mut self, parameter: &'ast syn::TypeParam) {
        self.shadowed |= parameter.ident == "competitive";
        syn::visit::visit_type_param(self, parameter);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        fn contains_crate_name(tokens: TokenStream) -> bool {
            tokens.into_iter().any(|token| match token {
                TokenTree::Ident(name) => name == "competitive",
                TokenTree::Group(group) => contains_crate_name(group.stream()),
                _ => false,
            })
        }
        self.shadowed |= contains_crate_name(mac.tokens.clone());
        syn::visit::visit_macro(self, mac);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.unqualified |= path.leading_colon.is_none()
            && path
                .segments
                .first()
                .is_some_and(|part| part.ident == "competitive");
        syn::visit::visit_path(self, path);
    }
}

impl<'ast> Visit<'ast> for BundlePaths<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path
            .segments
            .first()
            .is_some_and(|part| part.ident == "competitive")
        {
            let mut flattened = path.clone();
            let mut flatten = FlattenPath {
                symbols: self.symbols,
                error: None,
            };
            flatten.visit_path_mut(&mut flattened);
            if let Some(error) = flatten.error {
                self.error.get_or_insert(error);
            } else {
                self.replacements.push((
                    path.span().byte_range(),
                    flattened.to_token_stream().to_string(),
                ));
            }
        } else {
            syn::visit::visit_path(self, path);
        }
    }

    fn visit_item_use(&mut self, import: &'ast ItemUse) {
        match bundle_import(import, self.symbols, self.exports, false) {
            Ok(code) => self.replacements.push((import.span().byte_range(), code)),
            Err(error) => {
                self.error.get_or_insert(error);
            }
        }
    }
}

struct FlattenPath<'a> {
    symbols: &'a Symbols,
    error: Option<anyhow::Error>,
}

impl VisitMut for FlattenPath<'_> {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        syn::visit_mut::visit_path_mut(self, path);
        if !path
            .segments
            .first()
            .is_some_and(|part| part.ident == "competitive")
        {
            return;
        }
        let source: Vec<_> = path
            .segments
            .iter()
            .skip(1)
            .map(|part| part.ident.to_string())
            .collect();
        let result = self.symbols.bundle_path(&source).and_then(|target| {
            let mut target: syn::Path = syn::parse_str(&target.join("::"))?;
            for (part, original) in target
                .segments
                .iter_mut()
                .rev()
                .zip(path.segments.iter().rev())
            {
                part.arguments = original.arguments.clone();
            }
            Ok(target)
        });
        match result {
            Ok(target) => *path = target,
            Err(error) => {
                self.error.get_or_insert(error);
            }
        }
    }
}

fn replace_source(span: Span, mut replacements: Vec<(Range<usize>, String)>) -> Result<String> {
    let mut source = span.source_text().context("missing original source text")?;
    let offset = span.byte_range().start;
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    for (range, replacement) in replacements {
        source.replace_range(range.start - offset..range.end - offset, &replacement);
    }
    Ok(source)
}
