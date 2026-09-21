use super::{MInt, MIntConvert, One, Zero};

#[derive(Clone, Debug)]
pub struct MemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
    pub fact: Vec<MInt<M>>,
    pub inv_fact: Vec<MInt<M>>,
}

impl<M> MemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
    pub fn new(max_n: usize) -> Self {
        let mut fact = vec![MInt::one(); max_n + 1];
        let mut inv_fact = vec![MInt::one(); max_n + 1];
        for i in 2..=max_n {
            fact[i] = fact[i - 1] * MInt::from(i);
        }
        inv_fact[max_n] = fact[max_n].inv();
        for i in (3..=max_n).rev() {
            inv_fact[i - 1] = inv_fact[i] * MInt::from(i);
        }
        Self { fact, inv_fact }
    }

    pub fn combination(&self, n: usize, r: usize) -> MInt<M> {
        debug_assert!(n < self.fact.len());
        if r <= n {
            self.fact[n] * self.inv_fact[r] * self.inv_fact[n - r]
        } else {
            MInt::zero()
        }
    }

    pub fn permutation(&self, n: usize, r: usize) -> MInt<M> {
        debug_assert!(n < self.fact.len());
        if r <= n {
            self.fact[n] * self.inv_fact[n - r]
        } else {
            MInt::zero()
        }
    }

    pub fn homogeneous_product(&self, n: usize, r: usize) -> MInt<M> {
        debug_assert!(n + r < self.fact.len() + 1);
        if n == 0 && r == 0 {
            MInt::one()
        } else {
            self.combination(n + r - 1, r)
        }
    }

    pub fn inv(&self, n: usize) -> MInt<M> {
        debug_assert!(n < self.fact.len());
        debug_assert!(n > 0);
        self.inv_fact[n] * self.fact[n - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorials() {
        use crate::{num::mint_basic::MInt1000000007 as M, tools::Xorshift};
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let limit = rng.random(1..=100usize);
            let fact = MemorizedFactorial::new(limit);
            let mut binom = vec![vec![M::new(0); limit + 2]; limit + 1];
            binom[0][0] = M::new(1);
            let mut product = M::new(1);
            for n in 0..=limit {
                if n > 0 {
                    product *= M::from(n);
                    binom[n][0] = M::new(1);
                    for k in 1..=n {
                        binom[n][k] = binom[n - 1][k - 1] + binom[n - 1][k];
                    }
                    assert_eq!(fact.inv(n) * M::from(n), M::new(1));
                }
                assert_eq!(fact.fact[n], product);
                assert_eq!(fact.fact[n] * fact.inv_fact[n], M::new(1));
                let k = rng.random(0..=limit + 1);
                assert_eq!(fact.combination(n, k), binom[n][k]);
                let expected: M = if k <= n {
                    (n - k + 1..=n).map(M::from).product()
                } else {
                    M::new(0)
                };
                assert_eq!(fact.permutation(n, k), expected);
            }
        }
    }
}
