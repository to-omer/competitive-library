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
    parse::parse_files,
};
use error::ParseResult;
use std::process::exit;

fn execute() -> ParseResult<()> {
    let config = Opt::from_args();
    let items = parse_files(&config)?;
    let mut map = SnippetMap::new(&config);
    map.collect_entries(&items);
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

// $ cargo run --bin snippet-extract -- crates\competitive\src\lib.rs --output=.vscode\rust.code-snippets --filter-attr=cargo_snippet::snippet --cfg="feature=\"snippet_nightly\""
// $ cargo run --bin snippet-extract -- crates\competitive\src\lib.rs crates\competitive\src\main.rs --output=.vscode\rust.code-snippets --cfg="feature=\"snippet_nightly\""
// TODO: implement include snippet
// TODO: check with before json
// TODO: logging
// TODO: add attribute `snippet::skip`
