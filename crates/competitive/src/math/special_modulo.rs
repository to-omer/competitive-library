use crate::num::MIntBase;

pub struct Mersenne61;
impl Mersenne61 {
    pub const MOD: u64 = (1 << 61) - 1;
}
impl MIntBase for Mersenne61 {
    type Inner = u64;
    #[inline]
    fn get_mod() -> Self::Inner {
        Self::MOD
    }
    #[inline]
    fn mod_zero() -> Self::Inner {
        0
    }
    #[inline]
    fn mod_one() -> Self::Inner {
        1
    }
    #[inline]
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        let mut z = x + y;
        if z >= Self::MOD {
            z -= Self::MOD
        }
        z
    }
    #[inline]
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        if x < y {
            x + Self::MOD - y
        } else {
            x - y
        }
    }
    #[inline]
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        let z = x as u128 * y as u128;
        Self::mod_add((z >> 61) as _, z as u64 & Self::MOD)
    }
    #[inline]
    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        Self::mod_mul(x, Self::mod_inv(y))
    }
    #[inline]
    fn mod_neg(x: Self::Inner) -> Self::Inner {
        if x == 0 {
            0
        } else {
            Self::MOD - x
        }
    }
    #[inline]
    fn mod_inv(x: Self::Inner) -> Self::Inner {
        let p = Self::MOD as i64;
        let (mut a, mut b) = (x as i64, p);
        let (mut u, mut x) = (1, 0);
        while a != 0 {
            let k = b / a;
            x -= k * u;
            b -= k * a;
            std::mem::swap(&mut x, &mut u);
            std::mem::swap(&mut b, &mut a);
        }
        (if x < 0 { x + p } else { x }) as _
    }
}

pub struct Mersenne127;
impl Mersenne127 {
    pub const MOD: u128 = (1 << 127) - 1;
    const MASK63: u128 = (1 << 63) - 1;
    const MASK64: u128 = (1 << 64) - 1;
    // a: [0, 2^128-1)
    // Return a mod p: [0, 2^127)
    fn reduce128(a: u128) -> u128 {
        if a >> 127 == 0 {
            a
        } else {
            (Self::MOD + 2u128).wrapping_add(a)
        }
    }
    // a: [0, 2^127)
    // Return a mod p: [0, 2^127-1)
    fn reduce127(a: u128) -> u128 {
        if a != Self::MOD {
            a
        } else {
            0
        }
    }
}
impl MIntBase for Mersenne127 {
    type Inner = u128;
    #[inline]
    fn get_mod() -> Self::Inner {
        Self::MOD
    }
    #[inline]
    fn mod_zero() -> Self::Inner {
        0
    }
    #[inline]
    fn mod_one() -> Self::Inner {
        1
    }
    #[inline]
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        let mut z = x + y;
        if z >= Self::MOD {
            z -= Self::MOD
        }
        z
    }
    #[inline]
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        if x < y {
            x + Self::MOD - y
        } else {
            x - y
        }
    }
    #[inline]
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        // a * 2^127 == a mod p
        // x = xu * 2^64 + xd
        // y = yu * 2^64 + yd
        // x * y = (xu * 2^64 + xd) * (yu * 2^64 + yd)
        //       = xu * yu * 2^128 + (xu * yd + xd * yu) * 2^64 + xd * yd
        let xu = x >> 64;
        let xd = x & Self::MASK64;
        let yu = y >> 64;
        let yd = y & Self::MASK64;
        let p = xu * yu * 2;
        let q = xd * yu + xu * yd;
        let q = (q >> 63) + ((q & Self::MASK63) << 64);
        let r = xd * yd;
        let (s, x) = p.overflowing_add(q);
        let (t, y) = s.overflowing_add(r);
        let z = x as u32 + y as u32;
        Self::reduce127(Self::reduce128(t + (z + z) as u128))
    }
    #[inline]
    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        Self::mod_mul(x, Self::mod_inv(y))
    }
    #[inline]
    fn mod_neg(x: Self::Inner) -> Self::Inner {
        if x == 0 {
            0
        } else {
            Self::MOD - x
        }
    }
    #[inline]
    fn mod_inv(x: Self::Inner) -> Self::Inner {
        let p = Self::MOD as i64;
        let (mut a, mut b) = (x as i64, p);
        let (mut u, mut x) = (1, 0);
        while a != 0 {
            let k = b / a;
            x -= k * u;
            b -= k * a;
            std::mem::swap(&mut x, &mut u);
            std::mem::swap(&mut b, &mut a);
        }
        (if x < 0 { x + p } else { x }) as _
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_mersenne61() {
        const MOD: u64 = Mersenne61::MOD;
        fn binary_mul_mod(mut a: u64, mut b: u64) -> u64 {
            let mut c = 0u64;
            while b > 0 {
                if b % 2 == 1 {
                    c = (c + a) % MOD;
                }
                a = (a + a) % MOD;
                b /= 2;
            }
            c
        }
        assert_eq!(binary_mul_mod(10, 9), 90);
        let max = MOD - 1;
        assert_eq!(binary_mul_mod(max, max), Mersenne61::mod_mul(max, max));

        const Q: usize = 100_000;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let (a, b) = rng.gen((..Mersenne61::get_mod(), ..Mersenne61::get_mod()));
            let bc = binary_mul_mod(a, b);
            let mc = Mersenne61::mod_mul(a, b);
            assert_eq!(bc, mc);
        }
    }

    #[test]
    fn test_reduce128() {
        const MOD: u128 = Mersenne127::MOD;
        assert_eq!(Mersenne127::reduce128(MOD - 1), MOD - 1);
        assert_eq!(Mersenne127::reduce128(MOD), MOD);

        assert_eq!(Mersenne127::reduce128(MOD + 1), 1);
        assert_eq!(Mersenne127::reduce128(MOD + 2), 2);
        assert_eq!(Mersenne127::reduce128(MOD + 3), 3);
        assert_eq!(Mersenne127::reduce128(MOD + MOD), MOD);

        assert_eq!(Mersenne127::reduce128(MOD + MOD + 1), MOD + 1); // only MOD + MOD + 1 greater than MOD
    }

    #[test]
    fn test_mersenne127() {
        const MOD: u128 = Mersenne127::MOD;
        fn binary_mul_mod(mut a: u128, mut b: u128) -> u128 {
            let mut c = 0u128;
            while b > 0 {
                if b % 2 == 1 {
                    c = (c + a) % MOD;
                }
                a = (a + a) % MOD;
                b /= 2;
            }
            c
        }
        assert_eq!(binary_mul_mod(10, 9), 90);
        let max = MOD - 1;
        assert_eq!(binary_mul_mod(max, max), Mersenne127::mod_mul(max, max));

        const Q: usize = 100_000;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let a = ((rng.rand64() as u128) << 64 | rng.rand64() as u128) % MOD;
            let b = ((rng.rand64() as u128) << 64 | rng.rand64() as u128) % MOD;
            let bc = binary_mul_mod(a, b);
            let mc = Mersenne127::mod_mul(a, b);
            assert_eq!(bc, mc);
        }
    }
}
