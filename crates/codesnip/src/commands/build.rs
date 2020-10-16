use crate::{mapping::SnippetMapExt as _, parse::parse_files};
use codesnip_core::{Error::FileNotFound, Filter, SnippetMap};
use serde::Serialize;
use serde_json::{from_reader, to_writer};
use std::{
    fs::File,
    io::stdout,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use syn::parse_str;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct OptBuild {
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

    /// Save analyzed data in to file.
    #[structopt(
        long,
        value_name = "FILE",
        parse(from_os_str),
        conflicts_with("use_cache")
    )]
    pub save_cache: Option<PathBuf>,

    /// Target file paths.
    #[structopt(value_name = "FILE", parse(from_os_str), conflicts_with("use_cache"))]
    pub targets: Vec<PathBuf>,

    /// Use cached data.
    #[structopt(long, value_name = "FILE", parse(from_os_str))]
    pub use_cache: Option<PathBuf>,
}

impl OptBuild {
    pub fn filter(&self) -> Filter {
        Filter::new(&self.filter_attr, &self.filter_item)
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        let map = if let Some(cache) = &self.use_cache {
            from_reader(File::open(&cache).map_err(|err| FileNotFound(cache.clone(), err))?)?
        } else {
            let items = parse_files(&self.targets, &self.cfg)?;
            let mut map = SnippetMap::new();
            map.collect_entries(&items, self.filter());
            map.format_all();
            map
        };
        if self.save_cache.is_some() {
            emit(&map, self.save_cache.as_ref())?;
        }
        emit(&map.to_vscode(), self.output.as_ref())?;
        Ok(())
    }
}

fn emit<T: Serialize, P: AsRef<Path>>(value: &T, output: Option<P>) -> anyhow::Result<()> {
    match output {
        Some(file) => {
            let f = File::create(file)?;
            to_writer(f, value)?;
        }
        _ => {
            to_writer(stdout().lock(), value)?;
        }
    }
    Ok(())
}
