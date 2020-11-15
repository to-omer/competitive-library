//! modint

#[codesnip::skip]
use crate::{
    num::{One, Zero},
    tools::IterScan,
};

#[repr(transparent)]
pub struct MInt<M>
where
    M: MIntBase,
{
    x: M::Inner,
    _marker: std::marker::PhantomData<fn() -> M>,
}

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

pub trait MIntConvert<T = <Self as MIntBase>::Inner>: MIntBase {
    fn from(x: T) -> <Self as MIntBase>::Inner;
    fn into(x: <Self as MIntBase>::Inner) -> T;
    fn mod_into() -> T;
}

pub mod mint_base;

#[cfg_attr(nightly, codesnip::skip)]
pub mod montgomery;

#[test]
fn test_mint() {
    use super::mint_base::MInt998244353;
    use crate::tools::Xorshift;
    let mut rand = Xorshift::default();
    const Q: usize = 10_000;
    for _ in 0..Q {
        let a = MInt998244353::new(rand.rand(MInt998244353::get_mod() as u64 - 1) as u32 + 1);
        let x = a.inv();
        assert!(x.x < MInt998244353::get_mod());
        assert_eq!(a * x, MInt998244353::one());
        assert_eq!(x, a.pow(MInt998244353::get_mod() as usize - 2));
    }
}
