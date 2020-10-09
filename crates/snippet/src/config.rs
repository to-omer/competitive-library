use std::path::PathBuf;
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

impl Opt {
    pub fn config() -> Config {
        let Self::SnippetExtract(mut config) = Self::from_args();
        if let Ok(skip) = parse_str::<syn::Path>("snippet::skip") {
            config.filter_item.push(skip);
        }
        config
    }
}
