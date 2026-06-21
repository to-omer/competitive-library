use super::{MInt, MIntConvert, MemorizedFactorial, One, Zero};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
};

pub struct BinomialPrefixSum<M>
where
    M: MIntConvert<usize>,
{
    query: Vec<(usize, usize)>,
    _marker: PhantomData<fn() -> M>,
}

impl<M> Debug for BinomialPrefixSum<M>
where
    M: MIntConvert<usize>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinomialPrefixSum")
            .field("query", &self.query)
            .finish()
    }
}

impl<M> Default for BinomialPrefixSum<M>
where
    M: MIntConvert<usize>,
{
    fn default() -> Self {
        Self {
            query: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<M> BinomialPrefixSum<M>
where
    M: MIntConvert<usize>,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            query: Vec::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, n: usize, m: usize) -> usize {
        let q = self.query.len();
        self.query.push((m.min(n), n));
        q
    }

    pub fn for_each<F>(self, mut f: F)
    where
        F: FnMut(usize, MInt<M>),
    {
        let query = &self.query;
        if query.is_empty() {
            return;
        }
        let max_n = query.iter().map(|&(_, n)| n).max().unwrap_or(0);
        let modulus = M::mod_into();
        debug_assert!(modulus > 2 && modulus % 2 == 1);
        debug_assert!(max_n < modulus);
        debug_assert!(query.iter().all(|&(m, n)| m <= n));

        let fact = MemorizedFactorial::<M>::new(max_n);
        let inv2 = MInt::<M>::from(2usize).inv();
        let mut cur = MInt::<M>::one();
        crate::mo_algorithm!(
            query,
            (m, n),
            |old_m| cur += fact.combination(n, old_m + 1),
            |new_m| cur -= fact.combination(n, new_m + 1),
            |old_n| cur = cur + cur - fact.combination(old_n, m),
            |new_n| cur = (cur + fact.combination(new_n, m)) * inv2,
            |i| f(i, cur),
        );
    }

    pub fn solve(self) -> Vec<MInt<M>> {
        let mut ans = vec![MInt::zero(); self.query.len()];
        self.for_each(|i, x| ans[i] = x);
        ans
    }
}

pub struct BinomialPolynomialPrefixSum<M, const K: usize>
where
    M: MIntConvert<usize>,
{
    query: Vec<(usize, usize, [MInt<M>; K])>,
}

impl<M, const K: usize> Debug for BinomialPolynomialPrefixSum<M, K>
where
    M: MIntConvert<usize>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinomialPolynomialPrefixSum")
            .field("query", &self.query)
            .finish()
    }
}

impl<M, const K: usize> Default for BinomialPolynomialPrefixSum<M, K>
where
    M: MIntConvert<usize>,
{
    fn default() -> Self {
        Self {
            query: Default::default(),
        }
    }
}

impl<M, const K: usize> BinomialPolynomialPrefixSum<M, K>
where
    M: MIntConvert<usize>,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            query: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, n: usize, m: usize, coef: [MInt<M>; K]) -> usize {
        let q = self.query.len();
        self.query.push((n, m, coef));
        q
    }

    pub fn for_each_contribution<F>(self, mut f: F)
    where
        F: FnMut(usize, MInt<M>),
    {
        let mut binom = BinomialPrefixSum::<M>::with_capacity(self.query.len() * K);
        let mut derived = Vec::with_capacity(self.query.len() * K);
        let mut stirling = [[MInt::<M>::zero(); K]; K];
        if K > 0 {
            stirling[0][0] = MInt::one();
        }
        for n in 1..K {
            for r in 1..=n {
                stirling[n][r] = stirling[n - 1][r - 1] + MInt::from(r) * stirling[n - 1][r];
            }
        }
        let mut coef_cache = HashMap::with_capacity(self.query.len());
        for (i, (n, m, coef)) in self.query.iter().enumerate() {
            let (n, m) = (*n, *m);
            let coef = *coef_cache.entry(*coef).or_insert_with(|| {
                let mut falling = [MInt::<M>::zero(); K];
                for (k, &c) in coef.iter().enumerate() {
                    if c.is_zero() {
                        continue;
                    }
                    for r in 0..=k {
                        falling[r] += c * stirling[k][r];
                    }
                }
                falling
            });
            let mut falling = MInt::one();
            for (r, &coef) in coef.iter().enumerate() {
                if r > n || r > m {
                    break;
                }
                if !coef.is_zero() {
                    binom.push(n - r, m - r);
                    derived.push((i, coef * falling));
                }
                if r + 1 < K {
                    falling *= MInt::from(n - r);
                }
            }
        }
        binom.for_each(|i, x| {
            let (q, coef) = derived[i];
            f(q, coef * x);
        });
    }

    pub fn solve(self) -> Vec<MInt<M>> {
        let mut ans = vec![MInt::zero(); self.query.len()];
        self.for_each_contribution(|i, x| ans[i] += x);
        ans
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        num::montgomery::{MInt998244353, Modulo998244353},
        tools::Xorshift,
    };

    #[test]
    fn test_binomial_prefix_sum() {
        const N: usize = 30;
        const Q: usize = 300;
        let mut rng = Xorshift::default();
        let mut c = vec![vec![MInt998244353::zero(); N + 1]; N + 1];
        c[0][0] = MInt998244353::one();
        for i in 1..=N {
            c[i][0] = MInt998244353::one();
            c[i][i] = MInt998244353::one();
            for j in 1..i {
                c[i][j] = c[i - 1][j - 1] + c[i - 1][j];
            }
        }
        let mut query = vec![];
        let mut solver = BinomialPrefixSum::<Modulo998244353>::with_capacity(Q);
        let mut callback_solver = BinomialPrefixSum::<Modulo998244353>::with_capacity(Q);
        for _ in 0..Q {
            let (n, m) = rng.random((0..=N, 0..=N * 2));
            solver.push(n, m);
            callback_solver.push(n, m);
            query.push((n, m));
        }
        let expected: Vec<MInt998244353> = query
            .iter()
            .map(|&(n, m)| (0..=m.min(n)).map(|i| c[n][i]).sum())
            .collect();
        let result = solver.solve();
        assert_eq!(expected, result);

        let mut result = vec![MInt998244353::zero(); query.len()];
        callback_solver.for_each(|i, x| result[i] = x);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_binomial_polynomial_prefix_sum() {
        const N: usize = 20;
        const Q: usize = 300;
        let mut rng = Xorshift::default();
        let mut c = vec![vec![MInt998244353::zero(); N + 1]; N + 1];
        c[0][0] = MInt998244353::one();
        for i in 1..=N {
            c[i][0] = MInt998244353::one();
            c[i][i] = MInt998244353::one();
            for j in 1..i {
                c[i][j] = c[i - 1][j - 1] + c[i - 1][j];
            }
        }

        macro_rules! check {
            ($k:expr) => {{
                const K: usize = $k;
                let mut query = vec![];
                let mut solver =
                    BinomialPolynomialPrefixSum::<Modulo998244353, K>::with_capacity(Q);
                let mut callback_solver =
                    BinomialPolynomialPrefixSum::<Modulo998244353, K>::with_capacity(Q);
                for _ in 0..Q {
                    let (n, m) = rng.random((0..=N, 0..=N * 2));
                    let coef = if rng.gen_bool(0.1) {
                        [MInt998244353::zero(); K]
                    } else {
                        std::array::from_fn(|_| rng.random(..))
                    };
                    solver.push(n, m, coef);
                    callback_solver.push(n, m, coef);
                    query.push((n, m, coef));
                }
                let expected: Vec<_> = query
                    .iter()
                    .map(|&(n, m, coef)| {
                        (0..=m.min(n)).fold(MInt998244353::zero(), |s, i| {
                            let x = MInt998244353::from(i);
                            let y = coef
                                .iter()
                                .rev()
                                .fold(MInt998244353::zero(), |y, &a| y * x + a);
                            s + c[n][i] * y
                        })
                    })
                    .collect();
                let result = solver.solve();
                assert_eq!(expected, result);

                let mut result = vec![MInt998244353::zero(); query.len()];
                callback_solver.for_each_contribution(|i, x| result[i] += x);
                assert_eq!(expected, result);
            }};
        }
        check!(0);
        check!(1);
        check!(2);
        check!(3);
        check!(4);
        check!(5);
    }
}
