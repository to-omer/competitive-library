//! algebra

#[cfg_attr(nightly, snippet::entry("algebra", inline))]
mod magma;
#[macro_use]
mod operations;

pub use magma::*;
pub use operations::*;
