use crate::num::anymodu32::AnyModu32;
use crate::num::modu32::{Modu32, Modulo};

#[cargo_snippet::snippet(name = "factorial")]
#[derive(Clone, Debug)]
pub struct MemorizedFactorial<M: Modulo> {
    fact: Vec<Modu32<M>>,
    inv_fact: Vec<Modu32<M>>,
}
#[cargo_snippet::snippet(name = "factorial")]
impl<M: Modulo> MemorizedFactorial<M> {
    pub fn new(max_n: usize) -> Self {
        let mut fact = vec![Modu32::one(); max_n + 1];
        let mut inv_fact = vec![Modu32::one(); max_n + 1];
        for i in 2..=max_n {
            fact[i] = fact[i - 1] * Modu32::new(i as u32);
        }
        inv_fact[max_n] = fact[max_n].inv();
        for i in (3..=max_n).rev() {
            inv_fact[i - 1] = inv_fact[i] * Modu32::new(i as u32);
        }
        Self { fact, inv_fact }
    }
    #[inline]
    pub fn combination(&self, n: usize, r: usize) -> Modu32<M> {
        debug_assert!(n < self.fact.len());
        if r <= n {
            self.fact[n] * self.inv_fact[r] * self.inv_fact[n - r]
        } else {
            Modu32::zero()
        }
    }
    #[inline]
    pub fn permutation(&self, n: usize, r: usize) -> Modu32<M> {
        debug_assert!(n < self.fact.len());
        if r <= n {
            self.fact[n] * self.inv_fact[n - r]
        } else {
            Modu32::zero()
        }
    }
    #[inline]
    pub fn homogeneous_product(&self, n: usize, r: usize) -> Modu32<M> {
        debug_assert!(n + r < self.fact.len() + 1);
        if n != 0 && r != 0 {
            self.combination(n + r - 1, r)
        } else {
            Modu32::one()
        }
    }
    #[inline]
    pub fn inv(&self, n: usize) -> Modu32<M> {
        debug_assert!(n < self.fact.len());
        debug_assert!(n > 0);
        self.inv_fact[n] * self.fact[n - 1]
    }
}

#[test]
fn test_factorials() {
    use crate::num::modu32::modulos::Modulo1000000007;
    let fact = MemorizedFactorial::new(100);
    type M = Modu32<Modulo1000000007>;
    for i in 0..101 {
        assert_eq!(fact.fact[i] * fact.inv_fact[i], M::new(1));
    }
    for i in 1..101 {
        assert_eq!(fact.inv(i), M::new(i as u32).inv());
    }
    assert_eq!(fact.combination(10, 0), M::new(1));
    assert_eq!(fact.combination(10, 1), M::new(10));
    assert_eq!(fact.combination(10, 2), M::new(45));
    assert_eq!(fact.combination(10, 3), M::new(120));
    assert_eq!(fact.combination(10, 4), M::new(210));
    assert_eq!(fact.combination(10, 5), M::new(252));
    assert_eq!(fact.combination(10, 6), M::new(210));
    assert_eq!(fact.combination(10, 7), M::new(120));
    assert_eq!(fact.combination(10, 8), M::new(45));
    assert_eq!(fact.combination(10, 9), M::new(10));
    assert_eq!(fact.combination(10, 10), M::new(1));
    assert_eq!(fact.combination(10, 11), M::new(0));

    assert_eq!(fact.permutation(10, 0), M::new(1));
    assert_eq!(fact.permutation(10, 1), M::new(10));
    assert_eq!(fact.permutation(10, 2), M::new(90));
    assert_eq!(fact.permutation(10, 3), M::new(720));
    assert_eq!(fact.permutation(10, 4), M::new(5040));
    assert_eq!(fact.permutation(10, 5), M::new(30240));
    assert_eq!(fact.permutation(10, 6), M::new(151200));
    assert_eq!(fact.permutation(10, 7), M::new(604800));
    assert_eq!(fact.permutation(10, 8), M::new(1814400));
    assert_eq!(fact.permutation(10, 9), M::new(3628800));
    assert_eq!(fact.permutation(10, 10), M::new(3628800));
    assert_eq!(fact.permutation(10, 11), M::new(0));
}

#[derive(Clone, Debug)]
pub struct SmallModMemorizedFactorial {
    fact: Vec<AnyModu32>,
}
impl SmallModMemorizedFactorial {
    pub fn new() -> Self {
        let p = AnyModu32::get_modulo() as usize;
        let mut fact = vec![AnyModu32::one(); p];
        for i in 1..p {
            fact[i] = fact[i - 1] * AnyModu32::new(i as u32);
        }
        Self { fact }
    }
    /// n! = a * p^e
    pub fn factorial(&self, n: usize) -> (AnyModu32, usize) {
        let p = AnyModu32::get_modulo() as usize;
        if n == 0 {
            (AnyModu32::one(), 0)
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
    pub fn combination(&self, n: usize, r: usize) -> AnyModu32 {
        if r <= n {
            let (a1, e1) = self.factorial(n);
            let (a2, e2) = self.factorial(r);
            let (a3, e3) = self.factorial(n - r);
            if e1 <= e2 + e3 {
                return a1 * (a2 * a3).inv();
            }
        }
        AnyModu32::zero()
    }
}

#[test]
fn test_small_factorials() {
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::time();
    const N: usize = 10_000;
    const Q: usize = 10_000;
    AnyModu32::set_modulo(2);
    let fact = SmallModMemorizedFactorial::new();
    for _ in 0..Q {
        let n = rand.rand(N as u64) as usize + 1;
        let k = rand.rand(N as u64) as usize % n;
        let x = fact.factorial(n).1 - fact.factorial(k).1 - fact.factorial(n - k).1;
        assert_eq!(x == 0, (n & k) == k);
    }
}
