use super::*;

impl<M> MIntBase for M
where
    M: MontgomeryReduction32,
{
    type Inner = u32;
    #[inline]
    fn get_mod() -> Self::Inner {
        <Self as MontgomeryReduction32>::MOD
    }
    #[inline]
    fn mod_zero() -> Self::Inner {
        0
    }
    #[inline]
    fn mod_one() -> Self::Inner {
        Self::N1
    }
    #[inline]
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        let z = x + y;
        let m = Self::get_mod();
        if z >= m { z - m } else { z }
    }
    #[inline]
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        if x < y {
            x + Self::get_mod() - y
        } else {
            x - y
        }
    }
    #[inline]
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        Self::reduce(x as u64 * y as u64)
    }
    #[inline]
    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        Self::mod_mul(x, Self::mod_inv(y))
    }
    #[inline]
    fn mod_neg(x: Self::Inner) -> Self::Inner {
        if x == 0 { 0 } else { Self::get_mod() - x }
    }
    fn mod_inv(x: Self::Inner) -> Self::Inner {
        let p = Self::get_mod() as i32;
        let (mut a, mut b) = (x as i32, p);
        let (mut u, mut x) = (1, 0);
        while a != 0 {
            let k = b / a;
            x -= k * u;
            b -= k * a;
            std::mem::swap(&mut x, &mut u);
            std::mem::swap(&mut b, &mut a);
        }
        Self::reduce((if x < 0 { x + p } else { x }) as u64 * Self::N3 as u64)
    }
    fn mod_inner(x: Self::Inner) -> Self::Inner {
        Self::reduce(x as u64)
    }
}
impl<M> MIntConvert<u32> for M
where
    M: MontgomeryReduction32,
{
    #[inline]
    fn from(x: u32) -> Self::Inner {
        Self::reduce(x as u64 * Self::N2 as u64)
    }
    #[inline]
    fn into(x: Self::Inner) -> u32 {
        Self::reduce(x as u64)
    }
    #[inline]
    fn mod_into() -> u32 {
        <Self as MIntBase>::get_mod()
    }
}
impl<M> MIntConvert<u64> for M
where
    M: MontgomeryReduction32,
{
    #[inline]
    fn from(x: u64) -> Self::Inner {
        Self::reduce(x % Self::get_mod() as u64 * Self::N2 as u64)
    }
    #[inline]
    fn into(x: Self::Inner) -> u64 {
        Self::reduce(x as u64) as u64
    }
    #[inline]
    fn mod_into() -> u64 {
        <Self as MIntBase>::get_mod() as u64
    }
}
impl<M> MIntConvert<usize> for M
where
    M: MontgomeryReduction32,
{
    #[inline]
    fn from(x: usize) -> Self::Inner {
        Self::reduce(x as u64 % Self::get_mod() as u64 * Self::N2 as u64)
    }
    #[inline]
    fn into(x: Self::Inner) -> usize {
        Self::reduce(x as u64) as usize
    }
    #[inline]
    fn mod_into() -> usize {
        <Self as MIntBase>::get_mod() as usize
    }
}
impl<M> MIntConvert<i32> for M
where
    M: MontgomeryReduction32,
{
    #[inline]
    fn from(x: i32) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as i32;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as i32) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    #[inline]
    fn into(x: Self::Inner) -> i32 {
        Self::reduce(x as u64) as i32
    }
    #[inline]
    fn mod_into() -> i32 {
        <Self as MIntBase>::get_mod() as i32
    }
}
impl<M> MIntConvert<i64> for M
where
    M: MontgomeryReduction32,
{
    #[inline]
    fn from(x: i64) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as i64;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as i64) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    #[inline]
    fn into(x: Self::Inner) -> i64 {
        Self::reduce(x as u64) as i64
    }
    #[inline]
    fn mod_into() -> i64 {
        <Self as MIntBase>::get_mod() as i64
    }
}
impl<M> MIntConvert<isize> for M
where
    M: MontgomeryReduction32,
{
    #[inline]
    fn from(x: isize) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as isize;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as isize) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    #[inline]
    fn into(x: Self::Inner) -> isize {
        Self::reduce(x as u64) as isize
    }
    #[inline]
    fn mod_into() -> isize {
        <Self as MIntBase>::get_mod() as isize
    }
}
/// m is prime, n = 2^32
pub trait MontgomeryReduction32 {
    /// m
    const MOD: u32;
    /// (-m)^{-1} mod n
    const R: u32 = {
        let m = Self::MOD;
        let mut r = 0;
        let mut t = 0;
        let mut i = 0;
        while i < 32 {
            if t % 2 == 0 {
                t += m;
                r += 1 << i;
            }
            t /= 2;
            i += 1;
        }
        r
    };
    /// n^1 mod m
    const N1: u32 = ((1u64 << 32) % Self::MOD as u64) as _;
    /// n^2 mod m
    const N2: u32 = (Self::N1 as u64 * Self::N1 as u64 % Self::MOD as u64) as _;
    /// n^3 mod m
    const N3: u32 = (Self::N1 as u64 * Self::N2 as u64 % Self::MOD as u64) as _;
    /// n^{-1}x = (x + (xr mod n)m) / n
    fn reduce(x: u64) -> u32 {
        let m: u32 = Self::MOD;
        let r = Self::R;
        let mut x = ((x + r.wrapping_mul(x as u32) as u64 * m as u64) >> 32) as u32;
        if x >= m {
            x -= m;
        }
        x
    }
}
macro_rules! define_montgomery_reduction_32 {
    ($([$name:ident, $m:expr, $mint_name:ident $(,)?]),* $(,)?) => {
        $(
            pub enum $name {}
            impl MontgomeryReduction32 for $name {
                const MOD: u32 = $m;
            }
            pub type $mint_name = MInt<$name>;
        )*
    };
}
define_montgomery_reduction_32!(
    [Modulo998244353, 998_244_353, MInt998244353],
    [Modulo2113929217, 2_113_929_217, MInt2113929217],
    [Modulo1811939329, 1_811_939_329, MInt1811939329],
    [Modulo2013265921, 2_013_265_921, MInt2013265921],
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::montgomery::MInt998244353 as M;
    use crate::tools::Xorshift;

    #[test]
    fn test_mint998244353() {
        let mut rng = Xorshift::default();
        const Q: usize = 1000;
        assert_eq!(0, MInt998244353::zero().inner());
        assert_eq!(1, MInt998244353::one().inner());
        assert_eq!(
            Modulo998244353::reduce(Modulo998244353::N3 as u64),
            Modulo998244353::N2
        );
        assert_eq!(
            Modulo998244353::reduce(Modulo998244353::N2 as u64),
            Modulo998244353::N1
        );
        assert_eq!(Modulo998244353::reduce(Modulo998244353::N1 as u64), 1);
        for _ in 0..Q {
            let x = rng.random(..MInt998244353::get_mod());
            assert_eq!(x, MInt998244353::new(x).inner());
            assert_eq!((-M::new(x)).inner(), (-MInt998244353::new(x)).inner());
            assert_eq!(x, MInt998244353::new(x).inv().inv().inner());
            assert_eq!(M::new(x).inv().inner(), MInt998244353::new(x).inv().inner());
        }

        for _ in 0..Q {
            let x = rng.random(..MInt998244353::get_mod());
            let y = rng.random(..MInt998244353::get_mod());
            assert_eq!(
                (M::new(x) + M::new(y)).inner(),
                (MInt998244353::new(x) + MInt998244353::new(y)).inner()
            );
            assert_eq!(
                (M::new(x) - M::new(y)).inner(),
                (MInt998244353::new(x) - MInt998244353::new(y)).inner()
            );
            assert_eq!(
                (M::new(x) * M::new(y)).inner(),
                (MInt998244353::new(x) * MInt998244353::new(y)).inner()
            );
            assert_eq!(
                (M::new(x) / M::new(y)).inner(),
                (MInt998244353::new(x) / MInt998244353::new(y)).inner()
            );
            assert_eq!(
                M::new(x).pow(y as usize).inner(),
                MInt998244353::new(x).pow(y as usize).inner()
            );
        }

        for _ in 0..Q {
            let x = rng.rand64();
            assert_eq!(
                M::from(x as u32).inner(),
                MInt998244353::from(x as u32).inner()
            );
            assert_eq!(M::from(x).inner(), MInt998244353::from(x).inner());
            assert_eq!(
                M::from(x as usize).inner(),
                MInt998244353::from(x as usize).inner()
            );
            assert_eq!(
                M::from(x as i32).inner(),
                MInt998244353::from(x as i32).inner()
            );
            assert_eq!(
                M::from(x as i64).inner(),
                MInt998244353::from(x as i64).inner()
            );
            assert_eq!(
                M::from(x as isize).inner(),
                MInt998244353::from(x as isize).inner()
            );
        }
    }
}
