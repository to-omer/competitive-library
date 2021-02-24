use crate::num::{MInt, MIntConvert, One, Zero};

#[codesnip::entry("factorial", include("MInt"))]
#[derive(Clone, Debug)]
pub struct MemorizedFactorial<M: MIntConvert<usize>> {
    pub fact: Vec<MInt<M>>,
    pub inv_fact: Vec<MInt<M>>,
}
#[codesnip::entry("factorial")]
impl<M: MIntConvert<usize>> MemorizedFactorial<M> {
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
    #[inline]
    pub fn combination(&self, n: usize, r: usize) -> MInt<M> {
        debug_assert!(n < self.fact.len());
        if r <= n {
            self.fact[n] * self.inv_fact[r] * self.inv_fact[n - r]
        } else {
            MInt::zero()
        }
    }
    #[inline]
    pub fn permutation(&self, n: usize, r: usize) -> MInt<M> {
        debug_assert!(n < self.fact.len());
        if r <= n {
            self.fact[n] * self.inv_fact[n - r]
        } else {
            MInt::zero()
        }
    }
    #[inline]
    pub fn homogeneous_product(&self, n: usize, r: usize) -> MInt<M> {
        debug_assert!(n + r < self.fact.len() + 1);
        if n != 0 && r != 0 {
            self.combination(n + r - 1, r)
        } else {
            MInt::one()
        }
    }
    #[inline]
    pub fn inv(&self, n: usize) -> MInt<M> {
        debug_assert!(n < self.fact.len());
        debug_assert!(n > 0);
        self.inv_fact[n] * self.fact[n - 1]
    }
}

#[codesnip::entry("SmallModMemorizedFactorial", include("MInt"))]
#[derive(Clone, Debug)]
pub struct SmallModMemorizedFactorial<M: MIntConvert<usize>> {
    fact: Vec<MInt<M>>,
}
#[codesnip::entry("SmallModMemorizedFactorial")]
impl<M: MIntConvert<usize>> Default for SmallModMemorizedFactorial<M> {
    fn default() -> Self {
        let p = M::mod_into();
        let mut fact = vec![MInt::<M>::one(); p];
        for i in 1..p {
            fact[i] = fact[i - 1] * MInt::<M>::from(i);
        }
        Self { fact }
    }
}
#[codesnip::entry("SmallModMemorizedFactorial")]
impl<M: MIntConvert<usize>> SmallModMemorizedFactorial<M> {
    pub fn new() -> Self {
        Default::default()
    }
    /// n! = a * p^e
    pub fn factorial(&self, n: usize) -> (MInt<M>, usize) {
        let p = M::mod_into();
        if n == 0 {
            (MInt::<M>::one(), 0)
        } else {
            let e = n / p;
            let res = self.factorial(e);
            if e % 2 == 0 {
                (res.0 * self.fact[n % p], res.1 + e)
            } else {
                (res.0 * -self.fact[n % p], res.1 + e)
            }
        }
    }
    pub fn combination(&self, n: usize, r: usize) -> MInt<M> {
        if r <= n {
            let (a1, e1) = self.factorial(n);
            let (a2, e2) = self.factorial(r);
            let (a3, e3) = self.factorial(n - r);
            if e1 <= e2 + e3 {
                return a1 * (a2 * a3).inv();
            }
        }
        MInt::<M>::zero()
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

    #[test]
    fn test_small_factorials() {
        use crate::num::{MIntBase, MIntConvert};
        use crate::tools::Xorshift;
        static mut MOD: u32 = 2;
        crate::define_basic_mintbase!(
            D,
            unsafe { MOD },
            u32,
            u64,
            [u32, u64, u128, usize],
            [i32, i64, i128, isize]
        );
        let mut rand = Xorshift::time();
        const N: usize = 10_000;
        const Q: usize = 10_000;
        let fact = SmallModMemorizedFactorial::<D>::new();
        for _ in 0..Q {
            let n = rand.rand(N as u64) as usize + 1;
            let k = rand.rand(N as u64) as usize % n;
            let x = fact.factorial(n).1 - fact.factorial(k).1 - fact.factorial(n - k).1;
            assert_eq!(x == 0, (n & k) == k);
        }
    }
}
