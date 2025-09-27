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
        use crate::num::mint_basic::MInt1000000007;
        let fact = MemorizedFactorial::new(100);
        type M = MInt1000000007;
        for i in 0..101 {
            assert_eq!(fact.fact[i] * fact.inv_fact[i], M::new(1));
        }
        for i in 1..101 {
            assert_eq!(fact.inv(i), M::new(i as u32).inv());
        }
        assert_eq!(fact.combination(10, 0), M::new(1));
        assert_eq!(fact.combination(10, 1), M::new(10));
        assert_eq!(fact.combination(10, 5), M::new(252));
        assert_eq!(fact.combination(10, 6), M::new(210));
        assert_eq!(fact.combination(10, 10), M::new(1));
        assert_eq!(fact.combination(10, 11), M::new(0));

        assert_eq!(fact.permutation(10, 0), M::new(1));
        assert_eq!(fact.permutation(10, 1), M::new(10));
        assert_eq!(fact.permutation(10, 5), M::new(30240));
        assert_eq!(fact.permutation(10, 6), M::new(151_200));
        assert_eq!(fact.permutation(10, 10), M::new(3_628_800));
        assert_eq!(fact.permutation(10, 11), M::new(0));
    }
}
