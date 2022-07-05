//! modint

use crate::{
    num::{One, Zero},
    tools::IterScan,
};

#[codesnip::entry("MIntBase", include("scanner", "zero_one"))]
pub use mint_base::{MInt, MIntBase, MIntConvert};

#[cfg_attr(nightly, codesnip::entry("MIntBase"))]
mod mint_base;

#[cfg_attr(nightly, codesnip::entry("MInt", include("MIntBase")))]
pub mod mint_basic;

#[cfg_attr(nightly, codesnip::entry("montgomery", include("MIntBase")))]
pub mod montgomery;
