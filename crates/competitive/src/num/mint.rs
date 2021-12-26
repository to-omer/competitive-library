//! modint

use crate::{
    num::{One, Zero},
    tools::IterScan,
};

#[cfg_attr(nightly, codesnip::entry("MIntBase", include("scanner", "zero_one")))]
#[repr(transparent)]
pub struct MInt<M>
where
    M: MIntBase,
{
    x: M::Inner,
    _marker: std::marker::PhantomData<fn() -> M>,
}

#[cfg_attr(nightly, codesnip::entry("MIntBase"))]
pub trait MIntBase {
    type Inner: Sized + Copy + Eq + std::fmt::Debug + std::hash::Hash;
    fn get_mod() -> Self::Inner;
    fn mod_zero() -> Self::Inner;
    fn mod_one() -> Self::Inner;
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner;
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner;
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner;
    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner;
    fn mod_neg(x: Self::Inner) -> Self::Inner;
    fn mod_inv(x: Self::Inner) -> Self::Inner;
    fn mod_pow(x: Self::Inner, y: usize) -> Self::Inner {
        let (mut x, mut y, mut z) = (x, y, Self::mod_one());
        while y > 0 {
            if y & 1 == 1 {
                z = Self::mod_mul(z, x);
            }
            x = Self::mod_mul(x, x);
            y >>= 1;
        }
        z
    }
}

#[cfg_attr(nightly, codesnip::entry("MIntBase"))]
pub trait MIntConvert<T = <Self as MIntBase>::Inner>: MIntBase {
    fn from(x: T) -> <Self as MIntBase>::Inner;
    fn into(x: <Self as MIntBase>::Inner) -> T;
    fn mod_into() -> T;
}

#[cfg_attr(nightly, codesnip::entry("MIntBase"))]
mod mint_base;

#[cfg_attr(nightly, codesnip::entry("MInt", include("MIntBase")))]
pub mod mint_basic;

#[cfg_attr(nightly, codesnip::entry("montgomery", include("MIntBase")))]
pub mod montgomery;
