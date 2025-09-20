//! algebra

use crate::num::{Bounded, One, Zero};

#[cfg_attr(nightly, codesnip::entry("MonoidAct"))]
pub use self::action::*;
#[cfg_attr(nightly, codesnip::entry("LazyMapMonoid"))]
pub use self::lazy_map::*;
#[codesnip::entry("algebra")]
pub use self::magma::*;
pub use self::operations::*;
#[codesnip::entry("ring")]
pub use self::ring::*;
pub use self::ring_operations::*;

#[cfg_attr(
    nightly,
    codesnip::entry(
        "MonoidAct",
        include("TupleOperation", "LastOperation", "LinearOperation")
    )
)]
mod action;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "LazyMapMonoid",
        include(
            "MonoidAct",
            "AdditiveOperation",
            "MaxOperation",
            "MinOperation",
            "bounded"
        )
    )
)]
mod lazy_map;
#[cfg_attr(nightly, codesnip::entry("algebra"))]
mod magma;
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
