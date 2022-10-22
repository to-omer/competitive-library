//! algebra

use crate::num::{Bounded, One, Zero};

#[codesnip::entry("algebra")]
pub use self::magma::*;
pub use self::monoid_action::{monoid_action_impls::*, MonoidAction};
pub use self::operations::*;
#[codesnip::entry("ring")]
pub use self::ring::*;
pub use self::ring_operations::*;

#[cfg_attr(nightly, codesnip::entry("algebra"))]
mod magma;
mod monoid_action;
mod operations;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "ring",
        include("algebra", "AdditiveOperation", "MultiplicativeOperation")
    )
)]
mod ring;
mod ring_operations;
