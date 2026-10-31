use anyhow::{Context as _, Result, ensure};
use codesnip::{mapping::SnippetMapExt as _, source::Sources};
use codesnip_core::{Filter, FormatOption, SnippetMap, parse_file_recursive};
use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::ToTokens as _;
use std::{collections::BTreeMap, fs, path::Path, process::Command};
use syn::Item;

mod imports;
mod source;
mod symbols;

const TARGET: &str = "x86_64-unknown-linux-gnu";

fn main() -> Result<()> {
    ensure!(
        std::env::args().nth(1).is_none(),
        "usage: cargo run --release -p verify_source (from workspace root)"
    );
    eprintln!("Loading snippets");
    let (symbols, snippets, resolver) = load_library(Path::new("."))?;
    let output = Path::new("target/doc/submissions");
    fs::create_dir_all(output)?;
    let mut manifest = BTreeMap::new();
    for package in ["library_checker", "aizu_online_judge"] {
        let tree = parse_file_recursive(
            format!("crates/{package}/src/lib.rs").into(),
            &[],
            &[
                syn::parse_quote!(test),
                syn::parse_quote!(feature = "verify_doc"),
            ],
        )?;
        generate_module(
            &tree.items,
            package,
            &symbols,
            &snippets,
            &resolver,
            output,
            &mut manifest,
        )?;
    }
    ensure!(!manifest.is_empty(), "no verify functions found");
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!(
        "Generated {} submission sources (standalone compilation passed)",
        manifest.len()
    );
    Ok(())
}

fn submission_compiler() -> Command {
    let mut command = Command::new("rustup");
    command.args(["run", "1.89.0", "rustc"]).args([
        "--edition=2024",
        "--crate-name=submission",
        "-Dwarnings",
        "-O",
    ]);
    command
}

fn check_source(code: &str) -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let source = temporary.path().join("submission.rs");
    fs::write(&source, code)?;
    let output = submission_compiler()
        .args(["--error-format=short", "--target", TARGET, "--emit=obj"])
        .arg(&source)
        .arg("-o")
        .arg(temporary.path().join("submission.o"))
        .output()?;
    ensure!(
        output.status.success(),
        "invalid generated submission ({TARGET}):\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn load_library(root: &Path) -> Result<(symbols::Symbols, SnippetMap, imports::ImportResolver)> {
    let sources = Sources::load(root.join(".github/workflows/codesnip.toml"))?;
    ensure!(
        sources.sources.len() == 1,
        "expected one competitive source"
    );
    let source = &sources.sources[0];
    ensure!(
        source.git.is_none() && source.prefix.is_none(),
        "expected a local competitive source without a prefix"
    );
    let library = parse_file_recursive(
        root.join(&source.path),
        source
            .cfg_enable
            .as_deref()
            .or(sources.cfg_enable.as_deref())
            .unwrap_or_default(),
        source
            .cfg_disable
            .as_deref()
            .or(sources.cfg_disable.as_deref())
            .unwrap_or_default(),
    )?;
    let symbols = symbols::Symbols::new(&library.items)?;
    let mut snippets = SnippetMap::new();
    let mut filter_attr = source
        .filter_attr
        .as_deref()
        .or(sources.filter_attr.as_deref())
        .unwrap_or_default()
        .to_vec();
    filter_attr.push(syn::parse_quote!(doc));
    snippets.collect_entries(
        &library.items,
        Filter::new(
            &filter_attr,
            source
                .filter_item
                .as_deref()
                .or(sources.filter_item.as_deref())
                .unwrap_or_default(),
        ),
    )?;
    let main = snippets
        .map
        .get_mut("main")
        .context("missing main snippet")?;
    let mut template = syn::parse_file(&main.contents)?;
    let before = template.items.len();
    template
        .items
        .retain(|i| !matches!(i, Item::Fn(f) if f.sig.ident == "solve"));
    ensure!(
        template.items.len() + 1 == before,
        "main must contain exactly one solve function"
    );
    main.contents = template.to_token_stream().to_string();
    for (name, snippet) in &mut snippets.map {
        let contents = strip_docs(syn::parse_str(&snippet.contents)?).to_string();
        snippet.contents = match sources.format {
            FormatOption::Minify => rust_minify::minify_opt(
                &contents,
                &rust_minify::MinifyOption {
                    remove_skip: true,
                    add_rustfmt_skip: false,
                },
            )
            .with_context(|| format!("minifying snippet {name}"))?,
            FormatOption::Rustfmt => FormatOption::Rustfmt
                .format(&contents, "2024")
                .with_context(|| format!("formatting snippet {name}"))?,
        };
    }
    Ok((symbols, snippets, imports::ImportResolver::new(root)?))
}

// codesnip does not filter attributes inside macro token streams.
fn strip_docs(tokens: TokenStream) -> TokenStream {
    let mut tokens = tokens.into_iter().peekable();
    let mut output = TokenStream::new();
    while let Some(token) = tokens.next() {
        if matches!(&token, TokenTree::Punct(p) if p.as_char() == '#')
            && matches!(tokens.peek(), Some(TokenTree::Group(g))
                if g.delimiter() == Delimiter::Bracket
                    && matches!(g.stream().into_iter().next(), Some(TokenTree::Ident(i)) if i == "doc"))
        {
            tokens.next();
            continue;
        }
        match token {
            TokenTree::Group(group) => {
                output.extend([TokenTree::Group(Group::new(
                    group.delimiter(),
                    strip_docs(group.stream()),
                ))]);
            }
            token => output.extend([token]),
        }
    }
    output
}

fn generate_module(
    items: &[Item],
    module: &str,
    symbols: &symbols::Symbols,
    snippets: &SnippetMap,
    resolver: &imports::ImportResolver,
    output: &Path,
    manifest: &mut BTreeMap<String, String>,
) -> Result<()> {
    for item in items {
        match item {
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    generate_module(
                        items,
                        &format!("{module}/{}", m.ident),
                        symbols,
                        snippets,
                        resolver,
                        output,
                        manifest,
                    )?;
                }
            }
            Item::Fn(function) => {
                let name = &function.sig.ident;
                if source::verification(function)
                    .with_context(|| format!("{module}::{name}"))?
                    .is_none()
                {
                    continue;
                }
                eprintln!("Generating {module}::{name}");
                let code = source::generate(items, function, symbols, snippets, resolver)
                    .with_context(|| format!("{module}::{name}"))?;
                ensure!(
                    code.len() <= 1_048_576,
                    "submission exceeds 1 MiB: {module}::{name}"
                );
                check_source(&code).with_context(|| {
                    format!("standalone compilation failed for {module}::{name}")
                })?;
                let path = output.join(module).join(format!("{name}.rs.txt"));
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(path, code)?;
                let page = format!("{module}/fn.{name}.html");
                let source = format!("submissions/{module}/{name}.rs.txt");
                ensure!(
                    manifest.insert(page, source).is_none(),
                    "duplicate verify: {module}::{name}"
                );
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write as _, process::Stdio};

    #[test]
    fn selected_solutions_run_with_shared_macros_and_helpers() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        let file = syn::parse_file(
            r#"
            use competitive::prelude::*;
            use competitive::data_structure::UnionFind;
            #[codesnip::entry(include("montgomery"))]
            type M = competitive::num::montgomery::MInt998244353;
            #[codesnip::entry("Query")]
            competitive::define_enum_scan! {
                enum Query: usize {
                    0 => Join { u: usize, v: usize }
                    1 => Same { u: usize, v: usize }
                }
            }
            #[codesnip::entry("adjust", include("M"))]
            fn adjust(value: bool) -> M { M::new(value as u32) + M::new(7) }
            #[codesnip::entry("boolean")]
            #[macro_export]
            macro_rules! boolean { ($value:expr) => { u8::from($value) }; }
            #[cfg_attr(not(feature = "verify_doc"), verify::library_checker("unionfind", include("UnionFind", "Query", "boolean")))]
            pub fn plain(reader: impl Read, writer: impl Write) {
                prepare_io!(reader, writer);
                // 入力を読んで連結性を判定する。
                sc!(n, q);
                let mut uf = UnionFind::new(n);
                for _ in 0..q {
                    sc!(query: Query);
                    match query {
                        Query::Join { u, v } => { uf.unite(u, v); }
                        Query::Same { u, v } => { pp!(boolean!(uf.same(u, v))); }
                    }
                }
            }
            #[verify::library_checker("unionfind", include("UnionFind", "Query", "adjust", "TotalOrd"), huge_pages)]
            pub fn adjusted(reader: impl Read, writer: impl Write) {
                prepare_io!(buffered; reader, writer);
                sc!(n, q);
                let mut uf = UnionFind::new(n);
                for _ in 0..q {
                    sc!(query: Query);
                    match query {
                        Query::Join { u, v } => { uf.unite(u, v); }
                        Query::Same { u, v } => { pp!(adjust(uf.same(u, v))); }
                    }
                }
            }
        "#,
        )
        .unwrap();
        let input = b"3 4\n1 0 1\n0 0 1\n1 0 1\n1 1 2\n";
        let mut count = 0;
        for item in &file.items {
            if let Item::Fn(function) = item
                && source::verification(function).unwrap().is_some()
            {
                count += 1;
                let code = source::generate(&file.items, function, &symbols, &snippets, &resolver)
                    .unwrap();
                assert!(!code.contains("mod submission"));
                assert!(!code.contains("extern crate self as competitive"));
                assert!(code.starts_with("pub fn solve() {\n    crate::prepare!(fast);"));
                assert!(!code.contains("cfg_attr(any(),rustfmt::skip)"));
                assert!(!code.contains("#[doc"));
                assert!(code.contains("0 => Join { u: usize, v: usize }\n"));
                assert!(
                    code.find("mod main_macros").unwrap()
                        < code.find("pub mod union_find").unwrap()
                );
                let parsed = syn::parse_file(&code).unwrap();
                let items = &parsed.items;
                let main = items
                    .iter()
                    .find_map(|item| match item {
                        Item::Macro(item)
                            if item.mac.path.to_token_stream().to_string() == "crate :: main" =>
                        {
                            Some(&item.mac)
                        }
                        _ => None,
                    })
                    .unwrap();
                let huge_pages = function.sig.ident == "adjusted";
                assert_eq!(
                    main.tokens.to_string().starts_with("huge_pages ;"),
                    huge_pages
                );
                assert_eq!(code.contains("pub struct HugePageAllocator"), huge_pages);
                if huge_pages {
                    check_source(&code).unwrap();
                }
                let solve = items
                    .iter()
                    .position(|item| matches!(item, Item::Fn(f) if f.sig.ident == "solve"))
                    .unwrap();
                assert!(
                    items[..solve]
                        .iter()
                        .all(|item| !matches!(item, Item::Use(_)))
                );
                assert_eq!(
                    code.contains("pub struct TotalOrd"),
                    function.sig.ident == "adjusted"
                );
                let output = run(&code, input);
                let expected = if function.sig.ident == "plain" {
                    "0\n1\n0\n"
                } else {
                    "7\n8\n7\n"
                };
                assert_eq!(output, expected.as_bytes());
            }
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn selected_solution_omits_other_trait_implementations() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        for name in [
            "dynamic_tree_vertex_add_path_sum",
            "dynamic_tree_vertex_add_subtree_sum",
            "dynamic_tree_subtree_add_subtree_sum",
            "dynamic_tree_vertex_set_path_composite",
        ] {
            let text =
                fs::read_to_string(root.join(format!("crates/library_checker/src/tree/{name}.rs")))
                    .unwrap();
            let original = syn::parse_file(&text).unwrap();
            let variants = ["competitive", "::competitive"].map(|prefix| {
                syn::parse_file(&text.replace(
                    "impl TopTreeSpec for",
                    &format!("impl {prefix}::tree::TopTreeSpec for"),
                ))
                .unwrap()
            });
            let mut count = 0;
            for item in &original.items {
                if let Item::Fn(function) = item
                    && source::verification(function).unwrap().is_some()
                {
                    count += 1;
                    let code =
                        source::generate(&original.items, function, &symbols, &snippets, &resolver)
                            .unwrap();
                    let top_tree = function.sig.ident.to_string().ends_with("_top_tree");
                    for qualified in &variants {
                        let changed = qualified
                            .items
                            .iter()
                            .find_map(|item| match item {
                                Item::Fn(f) if f.sig.ident == function.sig.ident => Some(f),
                                _ => None,
                            })
                            .unwrap();
                        let changed_code = source::generate(
                            &qualified.items,
                            changed,
                            &symbols,
                            &snippets,
                            &resolver,
                        )
                        .unwrap();
                        assert_eq!(
                            code.split_once("mod main_macros").unwrap().1,
                            changed_code.split_once("mod main_macros").unwrap().1,
                        );
                        if !top_tree {
                            assert_eq!(code, changed_code);
                        }
                        if name == "dynamic_tree_vertex_add_subtree_sum" && top_tree {
                            assert_eq!(
                                run(
                                    &changed_code,
                                    b"2 4\n1 2\n0 1\n2 0 1\n1 0 3\n2 0 1\n2 1 0\n",
                                ),
                                b"1\n4\n2\n",
                            );
                        }
                    }
                    assert_eq!(code.contains("TopTreeSpec"), top_tree);
                    assert_eq!(code.contains("LinkCutTreeSpec"), !top_tree);
                    if name == "dynamic_tree_vertex_set_path_composite" {
                        assert_eq!(
                            code.contains("impl LazyMapMonoid for PathComposite"),
                            !top_tree
                        );
                        assert_eq!(
                            code.contains("impl Unital for BidirectionalAffine"),
                            !top_tree
                        );
                        assert_eq!(
                            code.contains("impl Associative for BidirectionalAffine"),
                            !top_tree
                        );
                        assert!(!code.contains("TopTreePathComposite"));
                    }
                }
            }
            assert_eq!(count, 2);
        }
    }

    #[test]
    fn unrelated_declarations_do_not_change_submission() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        let solution = r#"
            use competitive::prelude::*;
            #[verify::library_checker("aplusb")]
            pub fn plain(reader: impl Read, writer: impl Write) {
                prepare_io!(reader, writer);
                sc!(n);
                let math = 4.0_f64.sqrt();
                pp!(math + n as f64);
            }
        "#;
        let unused = r#"
            #[doc = r"An unused import
            with a multiline attribute"]
            use competitive::tools::Scanner as UnusedScanner;
            use competitive::num::montgomery::MInt998244353 as M;
            #[codesnip::entry("UnusedQuery", include("M"))]
            competitive::define_enum_scan! {
                pub enum UnusedQuery: usize { 0 => Value { v: M } }
            }
            pub struct Unused;
            impl Unused { pub fn sqrt(&self) -> M { M::new(1) } }
            #[codesnip::entry("Skipped", when("fast_main"))]
            #[codesnip::skip]
            pub struct Skipped;
            use competitive::math::{self, lcm as n};
            #[codesnip::entry("Unrelated", include("n", "gcd"))]
            pub fn unrelated() -> u64 { n(12, 18) + math::gcd(12, 18) }
        "#;
        let generate = |text: &str| {
            let file = syn::parse_file(text).unwrap();
            let function = file
                .items
                .iter()
                .find_map(|item| match item {
                    Item::Fn(f) if f.sig.ident == "plain" => Some(f),
                    _ => None,
                })
                .unwrap();
            source::generate(&file.items, function, &symbols, &snippets, &resolver).unwrap()
        };
        let baseline = generate(solution);
        let with_unused = generate(&format!("{solution}{unused}"));
        assert_eq!(baseline, with_unused);
        let associated = generate(&solution.replace("4.0_f64.sqrt()", "f64::sqrt(4.0)"));
        assert_eq!(
            baseline.split_once("mod main_macros").unwrap().1,
            associated.split_once("mod main_macros").unwrap().1,
        );
        for code in [baseline, associated, with_unused] {
            assert!(!code.contains("mod_sqrt"));
            assert!(!code.contains("montgomery"));
            assert!(!code.contains("pub struct Unused"));
            let namespaces: Vec<_> = syn::parse_file(&code)
                .unwrap()
                .items
                .into_iter()
                .filter_map(|item| match item {
                    Item::Mod(module) if matches!(module.vis, syn::Visibility::Public(_)) => {
                        Some(module.ident.to_string())
                    }
                    _ => None,
                })
                .collect();
            assert!(namespaces.is_empty());
            assert_eq!(run(&code, b"0\n"), b"2\n");
        }
    }

    #[test]
    fn qualified_paths_keep_original_namespaces() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        let file = syn::parse_file(
            r#"
            use competitive::prelude::*;
            #[codesnip::entry("divisor", include("gcd"))]
            fn divisor(a: u64, b: u64) -> u64 { competitive::math::gcd(a, b) }
            #[verify::library_checker("aplusb", include("divisor", "UnionFind"))]
            pub fn plain(reader: impl Read, writer: impl Write) {
                prepare_io!(reader, writer);
                sc!(a: u64, b: u64);
                let math = 7;
                mod math { pub fn gcd(_: u64, _: u64) -> u64 { 11 } }
                pp!(divisor(a, b));
                pp!(math::gcd(a, b));
                pp!(self::divisor(a, b));
                let qualified = ::competitive::math::gcd(a, b);
                pp!(qualified);
                pp!(math);
                mod other {
                    pub mod math { pub fn gcd(_: u64, _: u64) -> u64 { 11 } }
                    pub fn value() -> u64 { self::math::gcd(0, 0) }
                }
                pp!(other::math::gcd(a, b));
                pp!(other::value());
                let matched = match a { 12 => ::competitive::math::gcd(a, b), _ => 0 };
                pp!(matched);
                let greater = (a > ::competitive::math::gcd(a, b)) as u8;
                pp!(greater);
                let shifted = 64_u64 >> ::competitive::math::gcd(a, b);
                pp!(shifted);
                fn singleton() -> ::competitive::data_structure::UnionFind {
                    ::competitive::data_structure::UnionFind::new(1)
                }
                let _ = singleton();
                struct Container<T>(std::marker::PhantomData<T>);
                impl<T> Container<T> {
                    fn math<U>() -> usize { std::mem::size_of::<T>() + std::mem::size_of::<U>() }
                    fn value() -> usize { Self::math::<u16>() }
                }
                pp!(Container::<u8>::math::<u16>() + Container::<u8>::value());
                struct Methods;
                impl Methods {
                    fn math<U>(&self) -> usize { std::mem::size_of::<U>() }
                }
                pp!(Methods.math::<u16>());
            }
        "#,
        )
        .unwrap();
        let Item::Fn(function) = &file.items[2] else {
            unreachable!()
        };
        let code = source::generate(&file.items, function, &symbols, &snippets, &resolver).unwrap();
        assert!(!code.contains("::crate::"));
        assert_eq!(
            run(&code, b"12 18\n"),
            b"6\n11\n6\n6\n7\n11\n11\n6\n1\n1\n6\n2\n"
        );
    }

    #[test]
    fn imports_preserve_names_and_scopes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        let file = syn::parse_file(
            r#"
            use ::competitive::prelude::*;
            use ::competitive::math::*;
            use ::competitive::math::{self, lcm as common_multiple};
            use ::competitive::num::montgomery::MInt998244353 as M;
            use ::competitive::num::montgomery::*;
            use std::cmp::min;
            use std::vec::{self};
            use crate::other_vec as vec;
            #[codesnip::entry("other_vec")]
            #[macro_export]
            macro_rules! other_vec { () => { 7 }; }
            #[codesnip::entry("helper")]
            fn helper() -> u64 {
                let mut bytes = &b""[..];
                bytes.read_to_end(&mut Vec::new()).unwrap();
                common_multiple(6, 4)
            }
            #[verify::library_checker("aplusb", include("lcm", "montgomery", "helper", "other_vec"))]
            pub fn plain(reader: impl Read, writer: impl Write) {
                prepare_io!(reader, writer);
                pp!(common_multiple(6, 4), math::gcd(6, 4), helper(), M::new(3));
                {
                    use ::competitive::math::gcd;
                    pp!(gcd(6, 4));
                }
                let gcd = |_: u64, _: u64| 42;
                pp!(gcd(6, 4));
                mod nested { pub use ::competitive::math::gcd; }
                pp!(nested::gcd(6, 4), MInt998244353::new(4), min(5, 7));
                {
                    mod competitive { pub mod math { pub fn gcd(_: u64, _: u64) -> u64 { 23 } } }
                    pp!(competitive::math::gcd(6, 4));
                }
                pp!(stringify!(competitive::math::gcd));
                macro_rules! spell { ($($token:tt)*) => { stringify!($($token)*) }; }
                pp!(spell!(competitive::math::gcd));
                let _: vec::Vec<i32> = std::vec![];
                pp!(vec!());
                mod exported { pub use ::competitive::num::montgomery as n; }
                pp!(exported::n::MInt998244353::new(9));
            }
            "#,
        )
        .unwrap();
        let Item::Fn(function) = file.items.last().unwrap() else {
            unreachable!()
        };
        let code = source::generate(&file.items, function, &symbols, &snippets, &resolver).unwrap();
        assert_eq!(
            run(&code, b""),
            b"12 2 12 3\n2\n42\n2 4 5\n23\ncompetitive::math::gcd\ncompetitive::math::gcd\n7\n9\n"
        );
    }

    #[test]
    fn trait_imports_require_explicit_dependencies() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        for (imports, local) in [
            ("use competitive::num::*;", ""),
            ("use competitive::num as n;", "use n::*;"),
            ("", "use competitive::num::*;"),
        ] {
            for dependency in ["", ", \"zero_one\""] {
                let file = syn::parse_file(&format!(
                    r#"
                    use competitive::prelude::*;
                    {imports}
                    #[codesnip::entry("Fallback")]
                    trait Fallback {{ fn is_zero(&mut self) -> bool {{ false }} }}
                    #[codesnip::entry("Fallback")]
                    impl Fallback for u32 {{}}
                    #[verify::library_checker("aplusb", include("bounded", "Fallback"{dependency}))]
                    pub fn plain(reader: impl Read, writer: impl Write) {{
                        prepare_io!(reader, writer);
                        {local}
                        let mut value = 0_u32;
                        let _ = value.is_minimum();
                        pp!(value.is_zero());
                        let _ = Fallback::is_zero(&mut value);
                    }}
                    "#
                ))
                .unwrap();
                let Item::Fn(function) = file.items.last().unwrap() else {
                    unreachable!()
                };
                let generated =
                    source::generate(&file.items, function, &symbols, &snippets, &resolver);
                if dependency.is_empty() {
                    let error = generated.unwrap_err().to_string();
                    assert!(
                        error.contains("competitive::num::Zero: zero_one"),
                        "{error}"
                    );
                } else {
                    assert_eq!(run(&generated.unwrap(), b""), b"true\n");
                }
            }
        }
    }

    #[test]
    fn conditional_imports_use_submission_configuration() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        let file = syn::parse_file(
            r#"
            use competitive::prelude::*;
            #[cfg(target_os = "linux")]
            use competitive::math::gcd as divisor;
            #[cfg(target_os = "macos")]
            use competitive::math::lcm as divisor;
            #[cfg(debug_assertions)]
            use competitive::num::Bounded;
            #[verify::library_checker("aplusb", include("gcd"))]
            pub fn plain(reader: impl Read, writer: impl Write) {
                prepare_io!(reader, writer);
                #[cfg(debug_assertions)]
                let value = 0_u64.is_minimum() as u64;
                #[cfg(not(debug_assertions))]
                let value = 1;
                pp!(divisor(6_u64, 4_u64), value);
            }
            "#,
        )
        .unwrap();
        let Item::Fn(function) = file.items.last().unwrap() else {
            unreachable!()
        };
        let code = source::generate(&file.items, function, &symbols, &snippets, &resolver).unwrap();
        check_source(&code).unwrap();
    }

    #[test]
    fn submission_warnings_are_rejected() {
        let code = r#"
            fn main() {
                #[cfg(all(target_os = "linux", target_arch = "x86_64", not(debug_assertions)))]
                let unused = 0;
            }
        "#;
        let error = check_source(code).unwrap_err().to_string();
        assert!(error.contains("unused variable"), "{error}");
    }

    #[test]
    fn equivalent_method_and_associated_calls_have_the_same_dependencies() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        let text = fs::read_to_string(root.join("crates/library_checker/src/tree/lca.rs")).unwrap();
        let mut libraries = Vec::new();
        for text in [
            text.clone(),
            text.replace("graph.hld(0)", "UndirectedSparseGraph::hld(&graph, 0)"),
        ] {
            let file = syn::parse_file(&text).unwrap();
            let function = file
                .items
                .iter()
                .find_map(|item| match item {
                    Item::Fn(f) if f.sig.ident == "lca_hld" => Some(f),
                    _ => None,
                })
                .unwrap();
            let code =
                source::generate(&file.items, function, &symbols, &snippets, &resolver).unwrap();
            assert_eq!(run(&code, b"3 2\n0 0\n1 2\n1 1\n"), b"0\n1\n");
            libraries.push(code.split_once("mod main_macros").unwrap().1.to_owned());
        }
        assert_eq!(libraries[0], libraries[1]);
    }

    #[test]
    fn invalid_submissions_are_reported() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (symbols, snippets, resolver) = load_library(&root).unwrap();
        for (include, body, helper, expected) in [
            (", include = \"gcd\"", "", "", "expected parentheses"),
            (
                ", include(\"gcd\"), include(\"lcm\")",
                "",
                "",
                "duplicate include",
            ),
            (", include(\"Missing\")", "", "", "Missing"),
            (", huge_pages, huge_pages", "", "", "duplicate huge_pages"),
            ("", "let unused = 0;", "", "unused variable: `unused`"),
            (
                "",
                "mod competitive { pub fn value() -> u32 { 7 } } let _ = competitive::value();",
                "",
                "cannot relocate unqualified competitive paths",
            ),
            (
                ", include(\"gcd\")",
                r#"
                macro_rules! define_local {
                    () => {
                        mod competitive {
                            pub mod math {
                                pub fn gcd(_: u64, _: u64) -> u64 { 23 }
                            }
                        }
                    };
                }
                define_local!();
                let _ = competitive::math::gcd(6, 4);
                let _ = ::competitive::math::gcd(6_u64, 4);
                "#,
                "",
                "cannot relocate unqualified competitive paths",
            ),
            (
                ", include(\"UnionFind\")",
                "",
                "#[codesnip::entry] struct UnionFind;",
                "conflicts with library snippet",
            ),
            (", include(\"math\")", "", "use competitive::math;", "math"),
            (
                ", include(\"M\")",
                "",
                "#[codesnip::entry] type M = competitive::num::mint_basic::MInt998244353;",
                "missing submission snippet for competitive::num::mint_basic",
            ),
            (
                "",
                "type M = competitive::num::mint_basic::MInt998244353; let _ = M::new(0);",
                "",
                "missing submission snippet for competitive::num::mint_basic",
            ),
            (
                ", include(\"M\")",
                "",
                "use competitive::num::mint_basic::MInt998244353 as M;",
                "M",
            ),
            (
                ", include(\"gcd\")",
                "pp!(gcd(6, 4));",
                "use competitive::math::*; use competitive::math::lcm as gcd;",
                "missing submission snippet for competitive::math::lcm: lcm",
            ),
            (
                "",
                "pp!(@it (0..2).merge_by(2..4, |a, b| a < b));",
                "use competitive::tools::IteratorExt as _;",
                "missing submission snippet for competitive::tools::IteratorExt",
            ),
        ] {
            let file = syn::parse_file(&format!(
                r#"
                use competitive::prelude::*;
                #[verify::library_checker("aplusb"{include})]
                pub fn plain(reader: impl Read, writer: impl Write) {{
                    prepare_io!(reader, writer);
                    {body}
                    pp!(2);
                }}
                {helper}
            "#
            ))
            .unwrap();
            let Item::Fn(function) = &file.items[1] else {
                unreachable!()
            };
            let error = source::generate(&file.items, function, &symbols, &snippets, &resolver)
                .and_then(|code| check_source(&code))
                .unwrap_err();
            assert!(format!("{error:#}").contains(expected), "{error:#}");
        }
    }

    fn run(code: &str, input: &[u8]) -> Vec<u8> {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("submission.rs");
        let binary = temporary.path().join("submission");
        fs::write(&source, code).unwrap();
        let output = submission_compiler()
            .arg("--error-format=short")
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut child = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        output.stdout
    }
}
