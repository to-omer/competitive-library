//! algebra

#[cfg_attr(feature = "snippet_nightly", snippet::entry("algebra", inline))]
mod magma;
#[macro_use]
mod operations;

pub use magma::*;
pub use operations::*;
