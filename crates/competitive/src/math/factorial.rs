use crate::num::{MInt, MIntConvert, One, Zero};

#[codesnip::entry("factorial", include("MIntBase"))]
#[derive(Clone, Debug)]
pub struct MemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
    pub fact: Vec<MInt<M>>,
    pub inv_fact: Vec<MInt<M>>,
}
#[codesnip::entry("factorial")]
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
        if n == 0 && r == 0 {
            MInt::one()
        } else {
            self.combination(n + r - 1, r)
        }
    }
    #[inline]
    pub fn inv(&self, n: usize) -> MInt<M> {
        debug_assert!(n < self.fact.len());
        debug_assert!(n > 0);
        self.inv_fact[n] * self.fact[n - 1]
    }
}

#[codesnip::entry("SmallModMemorizedFactorial", include("MIntBase"))]
#[derive(Clone, Debug)]
pub struct SmallModMemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
    fact: Vec<MInt<M>>,
}
#[codesnip::entry("SmallModMemorizedFactorial")]
impl<M> Default for SmallModMemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
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
impl<M> SmallModMemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
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

#[codesnip::entry("PowPrec", include("MIntBase"))]
#[derive(Debug, Clone)]
pub struct PowPrec<M>
where
    M: MIntConvert<usize>,
{
    sqn: usize,
    p0: Vec<MInt<M>>,
    p1: Vec<MInt<M>>,
}
#[codesnip::entry("PowPrec")]
impl<M> PowPrec<M>
where
    M: MIntConvert<usize>,
{
    pub fn new(a: MInt<M>) -> Self {
        let sqn = (M::mod_into() as f64).sqrt() as usize + 1;
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
        Self { sqn, p0, p1 }
    }
    pub fn pow(&self, n: usize) -> MInt<M> {
        let n = n % (M::mod_into() - 1);
        let (p, q) = (n / self.sqn, n % self.sqn);
        self.p1[p] * self.p0[q]
    }
    pub fn powi(&self, n: isize) -> MInt<M> {
        let n = n.rem_euclid(M::mod_into() as isize - 1) as usize;
        let (p, q) = (n / self.sqn, n % self.sqn);
        self.p1[p] * self.p0[q]
    }
    pub fn inv(&self) -> MInt<M> {
        self.powi(-1)
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
        use crate::num::mint_basic::DynModuloU32;
        use crate::tools::Xorshift;
        let mut rng = Xorshift::time();
        const Q: usize = 100_000;
        DynModuloU32::set_mod(2);
        let fact = SmallModMemorizedFactorial::<DynModuloU32>::new();
        for _ in 0..Q {
            let n = rng.gen(1..=1_000_000_000_000_000_000);
            let k = rng.gen(0..=n);
            let x = fact.factorial(n).1 - fact.factorial(k).1 - fact.factorial(n - k).1;
            assert_eq!(x == 0, (n & k) == k);
        }
    }
}
