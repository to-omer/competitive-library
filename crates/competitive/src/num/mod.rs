use crate::tools::IterScan;

#[codesnip::entry("BarrettReduction")]
pub use self::barrett_reduction::BarrettReduction;
#[codesnip::entry("bounded")]
pub use self::bounded::Bounded;
#[codesnip::entry("Complex")]
pub use self::complex::Complex;
#[codesnip::entry("discrete_steps")]
pub use self::discrete_steps::{DiscreteSteps, RangeBoundsExt};
#[codesnip::entry("DoubleDouble")]
pub use self::double_double::DoubleDouble;
#[codesnip::entry("float")]
pub use self::float::{Float, Float32, Float64};
#[codesnip::entry("integer")]
pub use self::integer::{BinaryRepr, ExtendedGcd, IntBase, Saturating, Signed, Unsigned, Wrapping};
pub use self::mint::*;
#[codesnip::entry("QuadDouble")]
pub use self::quad_double::QuadDouble;
#[codesnip::entry("zero_one")]
pub use self::zero_one::{One, Zero};

#[cfg_attr(nightly, codesnip::entry("BarrettReduction"))]
mod barrett_reduction;
#[cfg_attr(nightly, codesnip::entry)]
mod bounded;
#[cfg_attr(nightly, codesnip::entry("Complex", include("zero_one", "scanner")))]
mod complex;
#[cfg_attr(nightly, codesnip::entry(include("bounded")))]
mod discrete_steps;
#[cfg_attr(nightly, codesnip::entry("DoubleDouble", include("bounded")))]
mod double_double;
#[cfg_attr(nightly, codesnip::entry(include("zero_one", "bounded")))]
mod float;
#[cfg_attr(nightly, codesnip::entry(include("zero_one", "bounded")))]
mod integer;
mod mint;
#[cfg_attr(nightly, codesnip::entry("QuadDouble"))]
mod quad_double;
#[cfg_attr(nightly, codesnip::entry)]
mod zero_one;
