//! mathematical datas

use crate::num::{One, Zero};

pub use factorial::*;
pub use fast_fourier_transform::*;
pub use formal_power_series::*;
pub use gcd::*;
pub use lagrange_interpolation::*;
#[codesnip::entry("Matrix")]
pub use matrix::Matrix;
pub use number_theoretic_transform::*;
pub use nums::*;
pub use polynomial::*;
pub use prime::*;
pub use special_modulo::*;

mod factorial;
mod fast_fourier_transform;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "FormalPowerSeries",
        inline,
        include(
            "NumberTheoreticTransform",
            "MInt",
            "mod_sqrt",
            "factorial",
            "PartialIgnoredOrd"
        )
    )
)]
mod formal_power_series;
mod gcd;
mod lagrange_interpolation;
#[cfg_attr(nightly, codesnip::entry("Matrix", include("zero_one")))]
mod matrix;
mod mod_sqrt;
#[cfg_attr(
    nightly,
    codesnip::entry("NumberTheoreticTransform", inline, include("MInt", "AssociatedValue"))
)]
mod number_theoretic_transform;
mod nums;
mod polynomial;
mod prime;
mod special_modulo;
