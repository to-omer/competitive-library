//! algebra

#[cfg_attr(nightly, codesnip::entry("algebra"))]
mod magma;
mod monoid_action;
mod operations;

#[codesnip::entry("algebra")]
pub use self::magma::*;
pub use self::monoid_action::{monoid_action_impls::*, MonoidAction};
pub use self::operations::*;
