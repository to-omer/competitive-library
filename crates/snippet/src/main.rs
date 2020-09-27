mod ast_helper;
mod attribute;
mod config;
mod error;
mod mapping;
mod output;
mod parse;

use crate::{
    config::Opt,
    mapping::SnippetMap,
    output::{emit, format_with_rustfmt},
    parse::parse_file_recursive,
};
use error::ParseResult;
use std::process::exit;

fn execute() -> ParseResult<()> {
    let config = Opt::from_args();
    let ast = parse_file_recursive(config.target.clone())?;
    let mut map = SnippetMap::new(&config);
    map.collect_entries(&ast);
    for contents in map.map.values_mut() {
        if let Some(formatted) = format_with_rustfmt(&contents) {
            *contents = formatted;
        }
    }
    emit(&map.into_vscode(), &config)?;
    Ok(())
}

const EXIT_FAILURE: i32 = 1;

fn main() {
    if let Err(err) = execute() {
        eprintln!("{}", err);
        exit(EXIT_FAILURE);
    }
}

// $ cargo run --bin snippet-extract --all-features -- crates\competitive\src\lib.rs --filter-attr=cargo_snippet::snippet --output=.vscode\rust.code-snippets
// TODO: recognize cfg/cfg_attr
// TODO: implement include snippet
