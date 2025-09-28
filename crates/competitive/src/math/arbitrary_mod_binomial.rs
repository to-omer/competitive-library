use super::{BarrettReduction, Unsigned, prime_factors, solve_simultaneous_linear_congruence};

fn pow64(x: u64, mut y: u64, br: &BarrettReduction<u128>) -> u64 {
    let mut x = x as u128;
    let mut z: u128 = 1;
    while y > 0 {
        if y & 1 == 1 {
            z = br.rem(z * x);
        }
        x = br.rem(x * x);
        y >>= 1;
    }
    z as u64
}

fn pow32(x: u32, mut y: u64, br: &BarrettReduction<u64>) -> u32 {
    let mut x = x as u64;
    let mut z: u64 = 1;
    while y > 0 {
        if y & 1 == 1 {
            z = br.rem(z * x);
        }
        x = br.rem(x * x);
        y >>= 1;
    }
    z as u32
}

#[derive(Debug)]
struct PrimePowerBinomial {
    p: u64,
    e: u32,
    m: u64,
    size: usize,
    fact: Vec<u64>,
    inv_fact: Vec<u64>,
    delta: u64,
    bp: BarrettReduction<u64>,
    bm: BarrettReduction<u64>,
    bm128: BarrettReduction<u128>,
}

impl PrimePowerBinomial {
    fn new(p: u64, e: u32, max_n: u64) -> Self {
        let m = p.checked_pow(e).expect("prime power overflow");
        let bp = BarrettReduction::new(p);
        let bm = BarrettReduction::new(m);
        let bm128 = BarrettReduction::new(m as u128);
        let size = max_n.min(m - 1);
        assert!(size < usize::MAX as u64);
        let size = size as usize;
        let mut fact = vec![1u64; size + 1];
        let mut inv_fact = vec![1u64; size + 1];
        if m < 1 << 31 {
            for i in 2..=size {
                fact[i] = if bp.rem(i as u64) == 0 {
                    fact[i - 1]
                } else {
                    bm.rem(fact[i - 1] * i as u64)
                };
            }
            inv_fact[size] = fact[size].mod_inv(m);
            for i in (3..=size).rev() {
                inv_fact[i - 1] = if bp.rem(i as u64) == 0 {
                    inv_fact[i]
                } else {
                    bm.rem(inv_fact[i] * i as u64)
                };
            }
        } else {
            for i in 2..=size {
                fact[i] = if bp.rem(i as u64) == 0 {
                    fact[i - 1]
                } else {
                    bm128.rem(fact[i - 1] as u128 * i as u128) as u64
                };
            }
            inv_fact[size] = fact[size].mod_inv(m);
            for i in (3..=size).rev() {
                inv_fact[i - 1] = if bp.rem(i as u64) == 0 {
                    inv_fact[i]
                } else {
                    bm128.rem(inv_fact[i] as u128 * i as u128) as u64
                };
            }
        }
        let delta = if p == 2 && e >= 3 { 1 } else { m - 1 };
        Self {
            p,
            e,
            m,
            size,
            fact,
            inv_fact,
            delta,
            bp,
            bm,
            bm128,
        }
    }

    fn combination(&self, mut n: u64, mut k: u64) -> u64 {
        if k > n {
            return 0;
        }
        assert!(self.size as u64 >= n.min(self.m - 1));
        if self.m < 1 << 31 {
            let mut res = 1u64;
            if self.e == 1 {
                while n > 0 {
                    let (nn, n0) = self.bp.div_rem(n);
                    let (nk, k0) = self.bp.div_rem(k);
                    if n0 < k0 {
                        return 0;
                    }
                    res = self.bm.rem(res * self.fact[n0 as usize]);
                    res = self.bm.rem(res * self.inv_fact[k0 as usize]);
                    res = self.bm.rem(res * self.inv_fact[(n0 - k0) as usize]);
                    n = nn;
                    k = nk;
                }
            } else {
                let mut r = n - k;
                let mut e0 = 0;
                let mut eq = 0;
                let mut i = 0;
                while n > 0 {
                    res = self.bm.rem(res * self.fact[self.bm.rem(n) as usize]);
                    res = self.bm.rem(res * self.inv_fact[self.bm.rem(k) as usize]);
                    res = self.bm.rem(res * self.inv_fact[self.bm.rem(r) as usize]);
                    n = self.bp.div(n);
                    k = self.bp.div(k);
                    r = self.bp.div(r);
                    let eps = n - k - r;
                    e0 += eps;
                    if e0 >= self.e as u64 {
                        return 0;
                    }
                    i += 1;
                    if i >= self.e {
                        eq ^= eps & 1;
                    }
                }
                if eq == 1 {
                    res = self.bm.rem(res * self.delta);
                }
                res = self
                    .bm
                    .rem(res * pow32(self.p as _, e0 as _, &self.bm) as u64);
            }
            res
        } else {
            let mut res = 1u128;
            if self.e == 1 {
                while n > 0 {
                    let (nn, n0) = self.bp.div_rem(n);
                    let (nk, k0) = self.bp.div_rem(k);
                    if n0 < k0 {
                        return 0;
                    }
                    res = self.bm128.rem(res * self.fact[n0 as usize] as u128);
                    res = self.bm128.rem(res * self.inv_fact[k0 as usize] as u128);
                    res = self
                        .bm128
                        .rem(res * self.inv_fact[(n0 - k0) as usize] as u128);
                    n = nn;
                    k = nk;
                }
            } else {
                let mut r = n - k;
                let mut e0 = 0;
                let mut eq = 0;
                let mut i = 0;
                while n > 0 {
                    res = self
                        .bm128
                        .rem(res * self.fact[self.bm.rem(n) as usize] as u128);
                    res = self
                        .bm128
                        .rem(res * self.inv_fact[self.bm.rem(k) as usize] as u128);
                    res = self
                        .bm128
                        .rem(res * self.inv_fact[self.bm.rem(r) as usize] as u128);
                    n = self.bp.div(n);
                    k = self.bp.div(k);
                    r = self.bp.div(r);
                    let eps = n - k - r;
                    e0 += eps;
                    if e0 >= self.e as u64 {
                        return 0;
                    }
                    i += 1;
                    if i >= self.e {
                        eq ^= eps & 1;
                    }
                }
                if eq == 1 {
                    res = self.bm128.rem(res * self.delta as u128);
                }
                res = self.bm128.rem(res * pow64(self.p, e0, &self.bm128) as u128);
            }
            res as u64
        }
    }
}

#[derive(Debug)]
pub struct ArbitraryModBinomial {
    ppbs: Vec<PrimePowerBinomial>,
}

impl ArbitraryModBinomial {
    pub fn new(modulus: u64, max_n: u64) -> Self {
        assert_ne!(modulus, 0);
        let ppbs = prime_factors(modulus)
            .into_iter()
            .map(|(p, e)| PrimePowerBinomial::new(p, e, max_n))
            .collect();
        Self { ppbs }
    }

    pub fn combination(&self, n: u64, k: u64) -> u64 {
        solve_simultaneous_linear_congruence(
            self.ppbs
                .iter()
                .map(|ppb| (1u64, ppb.combination(n, k), ppb.m)),
        )
        .unwrap()
        .0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        math::MemorizedFactorial,
        num::mint_basic::{MInt998244353, Modulo998244353},
        tools::Xorshift,
    };

    #[test]
    fn test_arbitrary_mod_binomial_small_mod() {
        for m in 1..=200 {
            let binom = ArbitraryModBinomial::new(m, 100);
            let mut dp = vec![vec![0u64; 101]; 101];
            dp[0][0] = 1 % m;
            for n in 1..=100 {
                dp[n][0] = 1 % m;
                for k in 1..=n {
                    dp[n][k] = dp[n - 1][k - 1].mod_add(dp[n - 1][k], m);
                }
            }
            for n in 0..=100 {
                for k in 0..=100 {
                    assert_eq!(binom.combination(n, k), dp[n as usize][k as usize]);
                }
            }
        }
    }

    #[test]
    fn test_arbitrary_mod_binomial_large_mod() {
        let mut rng = Xorshift::default();
        for i in 1..=200 {
            let m = if i <= 2 {
                (1 << 31) + 1 - i
            } else {
                rng.random(1..=1_000_000_000_000u64)
            };
            let binom = ArbitraryModBinomial::new(m, 100);
            let mut dp = vec![vec![0u64; 101]; 101];
            dp[0][0] = 1 % m;
            for n in 1..=100 {
                dp[n][0] = 1 % m;
                for k in 1..=n {
                    dp[n][k] = dp[n - 1][k - 1].mod_add(dp[n - 1][k], m);
                }
            }
            for n in 0..=100 {
                for k in 0..=100 {
                    assert_eq!(binom.combination(n, k), dp[n as usize][k as usize]);
                }
            }
        }
    }

    #[test]
    fn test_arbitrary_mod_binomial_prime_mod() {
        let mut rng = Xorshift::default();
        let binom = ArbitraryModBinomial::new(MInt998244353::get_mod() as _, 1_000_000);
        let fact = MemorizedFactorial::<Modulo998244353>::new(1_000_000);
        for _ in 0..100_000 {
            let n = rng.random(0..=1_000_000);
            let k = rng.random(0..=n);
            assert_eq!(
                binom.combination(n, k),
                fact.combination(n as _, k as _).inner() as u64
            );
        }
    }
}
