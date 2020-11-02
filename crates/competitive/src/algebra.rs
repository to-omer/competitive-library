//! algebra

#[cfg_attr(nightly, codesnip::entry("algebra", inline))]
mod magma;
mod operations;

pub use magma::*;
pub use operations::*;
