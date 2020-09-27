use std::path::PathBuf;
use structopt::{clap, StructOpt};
use syn::parse_str;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "snippet-extract", about = "snippet")]
#[structopt(rename_all = "kebab-case")]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    /// Target file path
    #[structopt(name = "PATH")]
    pub target: PathBuf,

    /// Output file, default stdout
    #[structopt(short, long)]
    pub output: Option<PathBuf>,

    /// Filter items by attributes path: e.g. --filter-item=test
    #[structopt(long, parse(try_from_str = parse_str::<syn::Path>))]
    pub filter_item: Vec<syn::Path>,

    /// Filter attributes by attributes path: e.g. --filter-attr=path
    #[structopt(long, parse(try_from_str = parse_str::<syn::Path>))]
    pub filter_attr: Vec<syn::Path>,
}

impl Opt {
    pub fn from_args() -> Self {
        let mut found_sub = false;
        let args = std::env::args().filter(|x| {
            if found_sub {
                true
            } else {
                found_sub = x == "snippet-extract";
                x != "snippet-extract"
            }
        });
        Self::from_iter(args)
    }
}
