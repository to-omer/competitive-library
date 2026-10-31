use super::*;

impl<M> MIntBase for M
where
    M: MontgomeryReduction32,
{
    type Inner = u32;
    fn get_mod() -> Self::Inner {
        <Self as MontgomeryReduction32>::MOD
    }
    fn mod_zero() -> Self::Inner {
        0
    }
    fn mod_one() -> Self::Inner {
        Self::N1
    }
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        let z = x + y;
        let m = Self::get_mod();
        if z >= m { z - m } else { z }
    }
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        if x < y {
            x + Self::get_mod() - y
        } else {
            x - y
        }
    }
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        Self::reduce(x as u64 * y as u64)
    }

    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        Self::mod_mul(x, Self::mod_inv(y))
    }
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
        // With a zero high limb, the positive inverse avoids the reduction's carry.
        let y = ((x.wrapping_mul(Self::R.wrapping_neg()) as u64 * Self::MOD as u64) >> 32) as u32;
        if y == 0 { 0 } else { Self::MOD - y }
    }
}
impl<M> MIntConvert<u32> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: u32) -> Self::Inner {
        Self::reduce(x as u64 * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> u32 {
        Self::mod_inner(x)
    }
    fn mod_into() -> u32 {
        <Self as MIntBase>::get_mod()
    }
}
impl<M> MIntConvert<u64> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: u64) -> Self::Inner {
        Self::reduce(x % Self::get_mod() as u64 * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> u64 {
        Self::mod_inner(x) as u64
    }
    fn mod_into() -> u64 {
        <Self as MIntBase>::get_mod() as u64
    }
}
impl<M> MIntConvert<usize> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: usize) -> Self::Inner {
        Self::reduce(x as u64 % Self::get_mod() as u64 * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> usize {
        Self::mod_inner(x) as usize
    }
    fn mod_into() -> usize {
        <Self as MIntBase>::get_mod() as usize
    }
}
impl<M> MIntConvert<i32> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: i32) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as i32;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as i32) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> i32 {
        Self::mod_inner(x) as i32
    }
    fn mod_into() -> i32 {
        <Self as MIntBase>::get_mod() as i32
    }
}
impl<M> MIntConvert<i64> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: i64) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as i64;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as i64) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> i64 {
        Self::mod_inner(x) as i64
    }
    fn mod_into() -> i64 {
        <Self as MIntBase>::get_mod() as i64
    }
}
impl<M> MIntConvert<isize> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: isize) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as isize;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as isize) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> isize {
        Self::mod_inner(x) as isize
    }
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
    [Modulo167772161, 167_772_161, MInt167772161],
    [Modulo469762049, 469_762_049, MInt469762049],
    [Modulo754974721, 754_974_721, MInt754974721],
    [Modulo998244353, 998_244_353, MInt998244353],
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::{Unsigned, mint_basic::MInt998244353 as M};
    use crate::tools::{Xorshift, testutil::integer_boundary_values};

    #[test]
    fn test_mod_inner() {
        fn check<M>()
        where
            M: MontgomeryReduction32,
        {
            let p = M::MOD as u64;
            let inverse = (1u64 << 32).mod_inv(p);
            let mut values = integer_boundary_values!(u32);
            for k in [1, 2, 3, u32::MAX as u64 / p] {
                for x in k * p - 1..=k * p + 1 {
                    if let Ok(x) = u32::try_from(x) {
                        values.push(x);
                    }
                }
            }
            values.extend(0..65536);
            values.extend(Xorshift::default().random_iter(0..=u32::MAX).take(65536));
            for x in values {
                let expected = (x as u64 * inverse % p) as u32;
                assert_eq!(M::mod_inner(x), expected);
                assert_eq!(<M as MIntConvert<u32>>::into(x), expected);
                assert_eq!(<M as MIntConvert<u64>>::into(x), expected as u64);
                assert_eq!(<M as MIntConvert<usize>>::into(x), expected as usize);
                assert_eq!(<M as MIntConvert<i32>>::into(x), expected as i32);
                assert_eq!(<M as MIntConvert<i64>>::into(x), expected as i64);
                assert_eq!(<M as MIntConvert<isize>>::into(x), expected as isize);
            }
        }
        enum Modulo<const P: u32> {}
        impl<const P: u32> MontgomeryReduction32 for Modulo<P> {
            const MOD: u32 = P;
        }
        enum HighModulo {}
        impl MontgomeryReduction32 for HighModulo {
            const MOD: u32 = 4294967291;
            const R: u32 = 3435973837;
        }
        check::<Modulo<3>>();
        check::<Modulo<17>>();
        check::<Modulo<65537>>();
        check::<Modulo167772161>();
        check::<Modulo469762049>();
        check::<Modulo754974721>();
        check::<Modulo998244353>();
        check::<Modulo<2013265921>>();
        check::<HighModulo>();
    }

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
