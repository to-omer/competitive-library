mod mapping;
mod parse;

use crate::{mapping::SnippetMapExt as _, parse::parse_files};
use serde::Serialize;
use serde_json::to_writer;
use snippet_core::{map::Filter, map::SnippetMap, parse::Error};
use std::{fs, io, path::Path, path::PathBuf, process::exit};
use structopt::StructOpt;
use syn::parse_str;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo", rename_all = "kebab-case")]
pub enum Opt {
    /// snippet extraction
    SnippetExtract(Config),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    /// Configure the environment: e.g. --cfg feature="nightly"
    #[structopt(long, value_name = "SPEC", parse(try_from_str = parse_str::<syn::Meta>))]
    pub cfg: Vec<syn::Meta>,

    /// Filter items by attributes path: e.g. --filter-item test
    #[structopt(long, value_name = "PATH", parse(try_from_str = parse_str::<syn::Path>))]
    pub filter_item: Vec<syn::Path>,

    /// Filter attributes by attributes path: e.g. --filter-attr path
    #[structopt(long, value_name = "PATH", parse(try_from_str = parse_str::<syn::Path>))]
    pub filter_attr: Vec<syn::Path>,

    /// Output file, default stdout.
    #[structopt(short, long, value_name = "FILE", parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Target file paths.
    #[structopt(value_name = "FILE", parse(from_os_str))]
    pub targets: Vec<PathBuf>,
}

impl Config {
    pub fn filter(&self) -> Filter {
        Filter::new(&self.filter_attr, &self.filter_item)
    }
}

fn execute() -> Result<(), Error> {
    let Opt::SnippetExtract(config) = Opt::from_args();
    let items = parse_files(&config.targets, &config.cfg)?;
    let mut map = SnippetMap::new();
    map.collect_entries(&items, config.filter());
    map.format_all();
    emit(&map.to_vscode(), config.output.as_ref())?;
    Ok(())
}

const EXIT_FAILURE: i32 = 1;

fn main() {
    if let Err(err) = execute() {
        eprintln!("error: {}", err);
        exit(EXIT_FAILURE);
    }
}

pub fn emit<T: Serialize, P: AsRef<Path>>(value: &T, output: Option<P>) -> io::Result<()> {
    match output {
        Some(file) => {
            let f = fs::File::create(file)?;
            to_writer(f, value)?;
        }
        _ => {
            to_writer(io::stdout().lock(), value)?;
        }
    }
    Ok(())
}
