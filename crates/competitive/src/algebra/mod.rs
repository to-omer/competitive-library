//! algebra

use crate::num::{Bounded, One, Zero};
use crate::tools::AssociatedValue;

#[codesnip::entry("MonoidAct")]
pub use self::action::*;
#[codesnip::entry("LazyMapMonoid")]
pub use self::lazy_map::*;
#[codesnip::entry("algebra")]
pub use self::magma::*;
pub use self::operations::*;
#[codesnip::entry("ring")]
pub use self::ring::*;
pub use self::ring_operations::*;

#[codesnip::entry(
    "MonoidAct",
    include("TupleOperation", "LastOperation", "LinearOperation")
)]
mod action;
#[codesnip::entry(
    "LazyMapMonoid",
    include(
        "MonoidAct",
        "AdditiveOperation",
        "ArrayOperation",
        "MaxOperation",
        "MinOperation",
        "CountingOperation",
        "bounded"
    )
)]
mod lazy_map;
#[cfg_attr(nightly, codesnip::entry("algebra"))]
mod magma;
mod operations;
#[codesnip::entry(
    "ring",
    include("algebra", "AdditiveOperation", "MultiplicativeOperation")
)]
mod ring;
mod ring_operations;
