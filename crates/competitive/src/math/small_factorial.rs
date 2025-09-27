use super::{MInt, MIntConvert, One, Zero, prime_factors};

#[derive(Clone, Debug)]
pub struct SmallModMemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
    p: u32,
    c: u32,
    fact: Vec<MInt<M>>,
    inv_fact: Vec<MInt<M>>,
    pow: Vec<MInt<M>>,
}

impl<M> Default for SmallModMemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
    fn default() -> Self {
        let m = M::mod_into();
        let pf = prime_factors(m as _);
        assert!(pf.len() <= 1);
        let p = pf[0].0 as u32;
        let c = pf[0].1;
        let mut fact = vec![MInt::one(); m];
        let mut inv_fact = vec![MInt::one(); m];
        let mut pow = vec![MInt::one(); c as usize];
        for i in 2..m {
            fact[i] = fact[i - 1]
                * if i as u32 % p != 0 {
                    MInt::from(i)
                } else {
                    MInt::one()
                };
        }
        inv_fact[m - 1] = fact[m - 1].inv();
        for i in (3..m).rev() {
            inv_fact[i - 1] = inv_fact[i]
                * if i as u32 % p != 0 {
                    MInt::from(i)
                } else {
                    MInt::one()
                };
        }
        for i in 1..c as usize {
            pow[i] = pow[i - 1] * MInt::from(p as usize);
        }
        Self {
            p,
            c,
            fact,
            inv_fact,
            pow,
        }
    }
}

impl<M> SmallModMemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
    pub fn new() -> Self {
        Default::default()
    }

    /// n! = a * p^e, c==1
    pub fn factorial(&self, n: usize) -> (MInt<M>, usize) {
        let p = self.p as usize;
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

    pub fn combination(&self, mut n: usize, mut r: usize) -> MInt<M> {
        if r > n {
            return MInt::<M>::zero();
        }
        if self.p == 2 && self.c == 1 {
            return MInt::from(((!n & r) == 0) as usize);
        }
        let mut k = n - r;
        let m = M::mod_into();
        let p = self.p as usize;
        let cnte = |mut x: usize| {
            let mut e = 0usize;
            while x > 0 {
                e += x;
                x /= p;
            }
            e
        };
        let e0 = cnte(n / p) - cnte(r / p) - cnte(k / p);
        if e0 >= self.c as usize {
            return MInt::<M>::zero();
        }
        let mut res = self.pow[e0];
        if (self.p > 2 && self.c >= 2 || self.c == 2)
            && (cnte(n / m) - cnte(r / m) - cnte(k / m)) % 2 == 1
        {
            res = -res;
        }
        while n > 0 {
            res *= self.fact[n % m] * self.inv_fact[r % m] * self.inv_fact[k % m];
            n /= p;
            r /= p;
            k /= p;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_factorials() {
        use crate::num::mint_basic::DynModuloU32;
        use crate::tools::Xorshift;
        let mut rng = Xorshift::new();
        const Q: usize = 100_000;
        DynModuloU32::set_mod(2);
        let fact = SmallModMemorizedFactorial::<DynModuloU32>::new();
        for _ in 0..Q {
            let n = rng.random(1..=1_000_000_000_000_000_000);
            let k = rng.random(0..=n);
            let x = fact.factorial(n).1 - fact.factorial(k).1 - fact.factorial(n - k).1;
            assert_eq!(x == 0, (n & k) == k);
            let x = fact.combination(n, k);
            assert_eq!(x.is_one(), (n & k) == k);
        }
    }
}
