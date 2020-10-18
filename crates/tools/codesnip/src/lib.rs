pub mod mapping;
pub mod parse;

pub use codesnip_attr::{entry, skip};

use crate::{mapping::SnippetMapExt as _, parse::parse_files};
use anyhow::Context as _;
use codesnip_core::{Error::FileNotFound, Filter, SnippetMap};
use serde_json::{from_reader, to_string};
use std::io::Write as _;
use std::{
    fs::File,
    io::stdout,
    path::{Path, PathBuf},
};
use structopt::{clap::AppSettings::DeriveDisplayOrder, StructOpt};
use syn::parse_str;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo", global_setting = DeriveDisplayOrder)]
pub enum Opt {
    /// Extract code snippets.
    Codesnip(Config),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    /// Target file paths.
    #[structopt(value_name = "FILE", parse(from_os_str))]
    pub targets: Vec<PathBuf>,

    /// Use cached data.
    #[structopt(long, value_name = "FILE", parse(from_os_str))]
    pub use_cache: Vec<PathBuf>,

    /// Configure the environment: e.g. --cfg feature="nightly"
    #[structopt(long, value_name = "SPEC", parse(try_from_str = parse_str::<syn::Meta>))]
    pub cfg: Vec<syn::Meta>,

    /// Filter items by attributes path: e.g. --filter-item test
    #[structopt(long, value_name = "PATH", parse(try_from_str = parse_str::<syn::Path>))]
    pub filter_item: Vec<syn::Path>,

    /// Filter attributes by attributes path: e.g. --filter-attr path
    #[structopt(long, value_name = "PATH", parse(try_from_str = parse_str::<syn::Path>))]
    pub filter_attr: Vec<syn::Path>,

    /// Save analyzed data in to file.
    #[structopt(long, value_name = "FILE", parse(from_os_str))]
    pub save_cache: Option<PathBuf>,

    /// Output file, default stdout.
    #[structopt(short, long, value_name = "FILE", parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Optput queried code snippet.
    #[structopt(long, value_name = "NAME")]
    pub query: Option<String>,
}

impl Opt {
    pub fn from_args() -> Self {
        StructOpt::from_args()
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        let Opt::Codesnip(opt) = self;
        opt.execute()
    }
}

impl Config {
    fn filter(&self) -> Filter {
        Filter::new(&self.filter_attr, &self.filter_item)
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        let mut map = SnippetMap::new();
        let items = parse_files(&self.targets, &self.cfg)?;
        map.collect_entries(&items, self.filter());
        map.format_all();

        for cache in self.use_cache.iter() {
            let mapt: SnippetMap =
                from_reader(File::open(&cache).map_err(|err| FileNotFound(cache.clone(), err))?)?;
            map.extend(mapt);
        }

        if self.save_cache.is_some() {
            emit(&to_string(&map)?, self.save_cache.as_ref())?;
        }

        let out = if let Some(name) = &self.query {
            let link = map
                .map
                .get(name)
                .with_context(|| format!("snippet `{}` not found", name))?;
            map.query(name, link)
        } else {
            to_string(&map.to_vscode())?
        };

        emit(&out, self.output.as_ref())?;
        Ok(())
    }
}

fn emit<P: AsRef<Path>>(value: &str, output: Option<P>) -> anyhow::Result<()> {
    match output {
        Some(file) => File::create(file)?.write_all(value.as_bytes())?,
        None => stdout().lock().write_all(value.as_bytes())?,
    }
    Ok(())
}
