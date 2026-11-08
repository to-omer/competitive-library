use crate::tools::{FastOutput, FastPrint, Scan, ScanSource};

#[codesnip::entry("BarrettReduction")]
pub use self::barrett_reduction::{BarrettReduction, Barrettable};
#[codesnip::entry("bounded")]
pub use self::bounded::Bounded;
#[codesnip::entry("Complex")]
pub use self::complex::Complex;
#[codesnip::entry("Decimal")]
pub use self::decimal::Decimal;
#[codesnip::entry("discrete_steps")]
pub use self::discrete_steps::{DiscreteSteps, RangeBoundsExt};
#[codesnip::entry("double_double")]
pub use self::double_double::{DoubleDouble, QuadDouble};
#[codesnip::entry("DualNumber")]
pub use self::dual_number::DualNumber;
#[codesnip::entry("Dyadic")]
pub use self::dyadic::Dyadic;
#[codesnip::entry("float")]
pub use self::float::{Float, Float32, Float64};
#[codesnip::entry("integer")]
pub use self::integer::{
    BinaryRepr, ExtendedGcd, IntBase, Saturating, Saturatingable, Signed, Unsigned, Wrapping,
    Wrappingable,
};
pub use self::mint::*;
#[codesnip::entry("Rational")]
pub use self::rational::Rational;
#[codesnip::entry("URational")]
pub use self::urational::URational;
#[codesnip::entry("zero_one")]
pub use self::zero_one::{One, Zero};

#[codesnip::entry("BarrettReduction", include("zero_one"))]
mod barrett_reduction;
#[codesnip::entry]
mod bounded;
#[codesnip::entry("Complex", include("zero_one", "scanner", "float"))]
mod complex;
#[codesnip::entry("Decimal", include("zero_one", "scanner"))]
pub mod decimal;
#[codesnip::entry(include("bounded"))]
mod discrete_steps;
#[codesnip::entry("double_double", include("float", "Decimal"))]
mod double_double;
#[codesnip::entry("DualNumber", include("zero_one"))]
mod dual_number;
#[codesnip::entry("Dyadic", include("integer"))]
mod dyadic;
#[codesnip::entry(include("zero_one", "bounded", "scanner"))]
mod float;
#[codesnip::entry(include("zero_one", "bounded", "scanner"))]
mod integer;
mod mint;
#[codesnip::entry("Rational", include("integer"))]
mod rational;
#[codesnip::entry("URational", include("integer"))]
mod urational;
#[codesnip::entry]
mod zero_one;

#[codesnip::entry(when("integer", "fastio"))]
impl<T: FastPrint> FastPrint for Saturating<T> {
    #[inline]
    fn fast_print<W: std::io::Write>(&self, writer: &mut FastOutput<W>) {
        self.0.fast_print(writer);
    }
}
