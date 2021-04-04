//! algebra

#[cfg_attr(nightly, codesnip::entry("algebra", inline))]
mod magma;
mod monoid_action;
mod operations;

pub use magma::*;
pub use monoid_action::{monoid_action_impls::*, MonoidAction};
pub use operations::*;
