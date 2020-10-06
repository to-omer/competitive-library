mod ast_helper;
mod attribute;
mod config;
mod error;
mod mapping;
mod output;
mod parse;

use crate::{config::Opt, mapping::SnippetMap, output::emit, parse::parse_files};
use std::process::exit;
use structopt::StructOpt;

fn execute() -> error::Result<()> {
    let Opt::SnippetExtract(config) = Opt::from_args();
    let items = parse_files(&config)?;
    let mut map = SnippetMap::new(&config);
    map.collect_entries(&items);
    map.format_all();
    emit(&map.to_vscode(), &config)?;
    Ok(())
}

const EXIT_FAILURE: i32 = 1;

fn main() {
    env_logger::init();
    if let Err(err) = execute() {
        log::error!("{}", err);
        exit(EXIT_FAILURE);
    }
}
