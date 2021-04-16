#[cfg_attr(nightly, codesnip::entry("bounded", inline))]
mod bounded;
#[cfg_attr(nightly, codesnip::entry("Complex", inline, include("zero_one")))]
mod complex;
#[cfg_attr(nightly, codesnip::entry(inline, include("zero_one")))]
mod float;
mod integer;
mod mint;
#[cfg_attr(nightly, codesnip::entry("QuadDouble", inline))]
mod quad_double;
#[cfg_attr(nightly, codesnip::entry("zero_one", inline))]
mod zero_one;

pub use bounded::Bounded;
pub use complex::Complex;
pub use integer::Saturating;
pub use mint::*;
pub use quad_double::QuadDouble;
pub use zero_one::*;
