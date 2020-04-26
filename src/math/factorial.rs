use super::anymod::*;
use super::modi64::*;
use cargo_snippet::snippet;

#[snippet(name = "factorial")]
#[derive(Clone, Debug)]
pub struct MemorizedFactorial {
    n: usize,
    fact: Vec<Modi64>,
    inv_fact: Vec<Modi64>,
}
#[snippet(name = "factorial")]
impl MemorizedFactorial {
    pub fn new(max_n: usize) -> MemorizedFactorial {
        let mut fact = vec![Modi64::new(1); max_n + 1];
        let mut inv_fact = vec![Modi64::new(1); max_n + 1];
        for i in 1..(max_n + 1) {
            fact[i] = fact[i - 1] * Modi64::new(i as i64);
            inv_fact[i] = inv_fact[i - 1] / Modi64::new(i as i64);
        }
        MemorizedFactorial {
            n: max_n + 1,
            fact: fact,
            inv_fact: inv_fact,
        }
    }
    pub fn combination(&self, n: usize, r: usize) -> Modi64 {
        if r <= n {
            self.fact[n] * self.inv_fact[r] * self.inv_fact[n - r]
        } else {
            Modi64::new(0)
        }
    }
    pub fn permutation(&self, n: usize, r: usize) -> Modi64 {
        if r <= n {
            self.fact[n] * self.inv_fact[n - r]
        } else {
            Modi64::new(0)
        }
    }
    pub fn homogeneous_product(&self, n: usize, r: usize) -> Modi64 {
        if n != 0 && r != 0 {
            self.combination(n + r - 1, r)
        } else {
            Modi64::new(1)
        }
    }
}

#[test]
fn test_factorials() {
    let fact = MemorizedFactorial::new(100);
    for i in 0..101 {
        assert_eq!(fact.fact[i] * fact.inv_fact[i], Modi64::new(1));
    }
    assert_eq!(fact.combination(10, 0), Modi64::new(1));
    assert_eq!(fact.combination(10, 1), Modi64::new(10));
    assert_eq!(fact.combination(10, 2), Modi64::new(45));
    assert_eq!(fact.combination(10, 3), Modi64::new(120));
    assert_eq!(fact.combination(10, 4), Modi64::new(210));
    assert_eq!(fact.combination(10, 5), Modi64::new(252));
    assert_eq!(fact.combination(10, 6), Modi64::new(210));
    assert_eq!(fact.combination(10, 7), Modi64::new(120));
    assert_eq!(fact.combination(10, 8), Modi64::new(45));
    assert_eq!(fact.combination(10, 9), Modi64::new(10));
    assert_eq!(fact.combination(10, 10), Modi64::new(1));
    assert_eq!(fact.combination(10, 11), Modi64::new(0));

    assert_eq!(fact.permutation(10, 0), Modi64::new(1));
    assert_eq!(fact.permutation(10, 1), Modi64::new(10));
    assert_eq!(fact.permutation(10, 2), Modi64::new(90));
    assert_eq!(fact.permutation(10, 3), Modi64::new(720));
    assert_eq!(fact.permutation(10, 4), Modi64::new(5040));
    assert_eq!(fact.permutation(10, 5), Modi64::new(30240));
    assert_eq!(fact.permutation(10, 6), Modi64::new(151200));
    assert_eq!(fact.permutation(10, 7), Modi64::new(604800));
    assert_eq!(fact.permutation(10, 8), Modi64::new(1814400));
    assert_eq!(fact.permutation(10, 9), Modi64::new(3628800));
    assert_eq!(fact.permutation(10, 10), Modi64::new(3628800));
    assert_eq!(fact.permutation(10, 11), Modi64::new(0));
}

#[derive(Clone, Debug)]
pub struct SmallModMemorizedFactorial {
    p: usize,
    fact: Vec<AnyMod>,
}
impl SmallModMemorizedFactorial {
    pub fn new(p: usize) -> SmallModMemorizedFactorial {
        let mut fact = vec![AnyMod::new(1, p as i64); p];
        for i in 1..p {
            fact[i] = fact[i - 1] * AnyMod::new(i as i64, p as i64);
        }
        SmallModMemorizedFactorial { p: p, fact: fact }
    }
    /// n! = a * p^e
    pub fn factorial(&self, n: usize) -> (AnyMod, usize) {
        if n == 0 {
            (AnyMod::new(1, self.p as i64), 0)
        } else {
            let e = n / self.p;
            let res = self.factorial(e);
            if e % 2 == 0 {
                (res.0 * self.fact[n % self.p], res.1 + e)
            } else {
                (res.0 * -self.fact[n % self.p], res.1 + e)
            }
        }
    }
    pub fn combination(&self, n: usize, r: usize) -> AnyMod {
        if r <= n {
            let (a1, e1) = self.factorial(n);
            let (a2, e2) = self.factorial(r);
            let (a3, e3) = self.factorial(n - r);
            if e1 <= e2 + e3 {
                return a1 * (a2 * a3).inv();
            }
        }
        AnyMod::new(0, self.p as i64)
    }
}

#[test]
fn test_small_factorials() {
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::time();
    const N: usize = 10_000;
    const Q: usize = 10_000;
    let fact = SmallModMemorizedFactorial::new(2);
    for _ in 0..Q {
        let n = rand.rand(N as u64) as usize + 1;
        let k = rand.rand(N as u64) as usize % n;
        let x = fact.factorial(n).1 - fact.factorial(k).1 - fact.factorial(n - k).1;
        assert_eq!(x == 0, (n & k) == k);
    }
}
