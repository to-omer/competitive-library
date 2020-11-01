pub mod mapping;
pub mod parse;

pub use codesnip_attr::{entry, skip};

use crate::{mapping::SnippetMapExt as _, parse::parse_files};
use anyhow::Context as _;
use codesnip_core::{Error::FileNotFound, Filter, SnippetMap};
use serde_json::{from_reader, to_string};
use std::{
    fs::File,
    io::{stdout, BufReader, Write as _},
    path::{Path, PathBuf},
};
use structopt::{
    clap::AppSettings::{DeriveDisplayOrder, InferSubcommands},
    StructOpt,
};
use syn::parse_str;

#[derive(Debug, StructOpt)]
#[structopt(
    bin_name = "cargo",
    global_settings = &[DeriveDisplayOrder, InferSubcommands]
)]
pub enum Opt {
    /// Extract code snippets.
    Codesnip(Config),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    /// Target file paths.
    #[structopt(short, long, value_name = "FILE", parse(from_os_str))]
    pub target: Vec<PathBuf>,

    /// Use cached data.
    #[structopt(long, value_name = "FILE", parse(from_os_str))]
    pub use_cache: Vec<PathBuf>,

    /// Configure the environment: e.g. --cfg=nightly
    #[structopt(long, value_name = "SPEC", parse(try_from_str = parse_str::<syn::Meta>))]
    pub cfg: Vec<syn::Meta>,

    /// Filter items by attributes path: e.g. --filter-item=test
    #[structopt(long, value_name = "PATH", parse(try_from_str = parse_str::<syn::Path>))]
    pub filter_item: Vec<syn::Path>,

    /// Filter attributes by attributes path: e.g. --filter-attr=path
    #[structopt(long, value_name = "PATH", parse(try_from_str = parse_str::<syn::Path>))]
    pub filter_attr: Vec<syn::Path>,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Save analyzed data into file.
    Cache {
        /// Output file.
        #[structopt(value_name = "FILE", parse(from_os_str))]
        output: PathBuf,
    },
    /// List names.
    List,
    /// Output snippet for VSCode.
    Snippet {
        /// Output file, default stdout.
        #[structopt(value_name = "FILE", parse(from_os_str))]
        output: Option<PathBuf>,
    },
    /// bundle
    Bundle {
        /// snippet name.
        #[structopt(value_name = "NAME")]
        name: String,
        /// excludes
        #[structopt(short, long, value_name = "NAME")]
        excludes: Vec<String>,
    },
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
        let items = parse_files(&self.target, &self.cfg)?;
        map.collect_entries(&items, self.filter());
        map.format_all();

        for cache in self.use_cache.iter() {
            let file = File::open(&cache).map_err(|err| FileNotFound(cache.clone(), err))?;
            let reader = BufReader::new(file);
            let mapt: SnippetMap = from_reader(reader)?;
            map.extend(mapt);
        }

        self.cmd.execute(map)
    }
}

impl Command {
    pub fn execute(&self, map: SnippetMap) -> anyhow::Result<()> {
        match self {
            Command::Cache { output } => {
                emit(&to_string(&map)?, Some(output))?;
            }
            Command::List => {
                for name in map.map.keys() {
                    println!("{}", name);
                }
            }
            Command::Snippet { output } => {
                emit(&to_string(&map.to_vscode())?, output.as_ref())?;
            }
            Command::Bundle { name, excludes } => {
                let link = map
                    .map
                    .get(name)
                    .with_context(|| format!("snippet `{}` not found", name))?;
                let excludes = excludes.iter().map(|s| s.as_str()).collect();
                println!("{}", map.bundle(&name, link, excludes, true));
            }
        }
        Ok(())
    }
}

fn emit<P: AsRef<Path>>(value: &str, output: Option<P>) -> anyhow::Result<()> {
    match output {
        Some(file) => {
            if let Some(parent) = file.as_ref().parent() {
                std::fs::create_dir_all(parent)?;
            }
            File::create(file)?.write_all(value.as_bytes())?
        }
        None => stdout().lock().write_all(value.as_bytes())?,
    }
    Ok(())
}
