pub mod commands;
pub mod mapping;
pub mod parse;

pub use codesnip_attr::{entry, skip};

use crate::commands::OptBuild;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo", rename_all = "kebab-case")]
pub enum Opt {
    /// Codesnip
    Codesnip(OptCodesnip),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum OptCodesnip {
    /// build snippets for VSCode
    Build(OptBuild),
}

impl Opt {
    pub fn from_args() -> Self {
        StructOpt::from_args()
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        let Opt::Codesnip(opt) = self;
        match opt {
            OptCodesnip::Build(opt) => opt.execute(),
        }
    }
}
