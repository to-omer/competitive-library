use super::{MInt, MIntConvert, One, prime_factors};

#[derive(Debug, Clone)]
pub struct PowPrec<M>
where
    M: MIntConvert<usize>,
{
    period: usize,
    sqn: usize,
    p0: Vec<MInt<M>>,
    p1: Vec<MInt<M>>,
}

impl<M> PowPrec<M>
where
    M: MIntConvert<usize>,
{
    pub fn new(a: MInt<M>) -> Self {
        let mut maxe = 0;
        let period: u64 = prime_factors(M::mod_into() as u64)
            .into_iter()
            .map(|(p, e)| {
                maxe = maxe.max(e);
                p.pow(e - 1) * (p - 1)
            })
            .product();
        let period = period as usize;
        let sqn = ((period as f64).sqrt() as usize).max(maxe as usize) + 1;
        let mut p0 = Vec::with_capacity(sqn);
        let mut p1 = Vec::with_capacity(sqn);
        let mut acc = MInt::<M>::one();
        for _ in 0..sqn {
            p0.push(acc);
            acc *= a;
        }
        let b = acc;
        acc = MInt::<M>::one();
        for _ in 0..sqn {
            p1.push(acc);
            acc *= b;
        }
        Self {
            period,
            sqn,
            p0,
            p1,
        }
    }

    pub fn pow(&self, n: usize) -> MInt<M> {
        if n < self.sqn {
            return self.p0[n];
        }
        let n = (n - self.sqn + 1) % self.period;
        let (p, q) = (n / self.sqn, n % self.sqn);
        self.p1[p] * self.p0[q] * self.p0[self.sqn - 1]
    }

    /// gcd(a, mod) must be 1
    pub fn powi(&self, n: isize) -> MInt<M> {
        let n = n.rem_euclid(self.period as isize) as usize;
        let (p, q) = (n / self.sqn, n % self.sqn);
        self.p1[p] * self.p0[q]
    }

    /// gcd(a, mod) must be 1
    pub fn inv(&self) -> MInt<M> {
        self.powi(-1)
    }
}

#[cfg(kani)]
mod verification {
    use super::*;
    use crate::{
        define_basic_mint32, define_basic_mintbase,
        num::{MIntBase, mint_basic::MInt2},
    };
    use std::mem::swap;

    define_basic_mint32!([Modulo8, 8, MInt8]);

    #[kani::proof]
    #[kani::unwind(5)]
    fn proof_full_width_exponents_mod_two() {
        let base: bool = kani::any();
        let exponent: usize = kani::any();
        let powers = PowPrec::new(MInt2::new(base as u32));
        kani::assert(
            powers.pow(exponent).inner() == (exponent == 0 || base) as u32,
            "For either residue modulo two and every usize exponent, precomputed powers match exponentiation, including zero and usize::MAX",
        );
        if base {
            let signed_exponent = exponent as isize;
            kani::assert(
                powers.powi(signed_exponent).inner() == 1 && powers.inv().inner() == 1,
                "For the unit modulo two, every isize exponent and inversion return one",
            );
            kani::cover!(signed_exponent == isize::MIN);
        }
        kani::cover!(exponent == 0 && !base);
        kani::cover!(exponent == usize::MAX);
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn proof_full_width_exponents_mod_eight() {
        let base = kani::any::<u8>() & 7;
        let exponent: usize = kani::any();
        let powers = PowPrec::new(MInt8::new(base as u32));
        let expected = if exponent == 0 {
            1
        } else if base & 1 == 1 {
            if exponent & 1 == 0 { 1 } else { base as u32 }
        } else if exponent == 1 {
            base as u32
        } else if exponent == 2 {
            base as u32 * base as u32 % 8
        } else {
            0
        };
        kani::assert(
            powers.pow(exponent).inner() == expected,
            "For every residue modulo eight and every usize exponent, precomputed powers match the exact periodic powers of odd units and the vanishing powers of even residues",
        );
        if base & 1 == 1 {
            let signed_exponent = exponent as isize;
            let expected = if signed_exponent & 1 == 0 {
                1
            } else {
                base as u32
            };
            kani::assert(
                powers.powi(signed_exponent).inner() == expected
                    && powers.inv().inner() == base as u32,
                "For every odd residue modulo eight, signed powers and inverses agree with parity over the complete isize domain",
            );
            kani::cover!(signed_exponent == isize::MIN);
        }
        kani::cover!(exponent == 0 && base == 0);
        kani::cover!(exponent == 2 && base == 2);
        kani::cover!(exponent == usize::MAX && base == 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        num::{
            Unsigned,
            mint_basic::{DynMIntU32, MInt998244353},
        },
        tools::Xorshift,
    };

    #[test]
    fn test_pow_prec_small() {
        for m in 2..=100 {
            DynMIntU32::set_mod(m);
            for a in 0..m {
                let a = DynMIntU32::new(a);
                let p = PowPrec::new(a);
                for i in (0..=m as usize * 2).chain([usize::MAX - 1, usize::MAX]) {
                    assert_eq!(p.pow(i), a.pow(i));
                }
                if m.gcd(a.inner()) == 1 {
                    for i in -(m as isize * 2)..=(m as isize * 2) {
                        assert_eq!(
                            p.powi(i),
                            if i >= 0 {
                                a.pow(i as _)
                            } else {
                                a.inv().pow((-i) as _)
                            }
                        );
                    }
                    assert_eq!(p.inv(), a.inv());
                }
            }
        }
    }

    #[test]
    fn test_pow_prec_large() {
        let mut rng = Xorshift::default();
        for _ in 0..10 {
            let a = rng.random(1..MInt998244353::get_mod());
            let a = MInt998244353::new(a);
            let p = PowPrec::new(a);
            for _ in 0..100 {
                let i = rng.random(0..2_000_000_000);
                assert_eq!(p.pow(i as _), a.pow(i as _));
                assert_eq!(p.powi(i as _), a.pow(i as _));
                assert_eq!(p.powi(-(i as isize)), a.inv().pow(i as _));
            }
            assert_eq!(p.inv(), a.inv());
        }
    }
}
