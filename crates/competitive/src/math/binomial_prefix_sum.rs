use super::{MInt, MIntConvert, MIntDotProduct, MemorizedFactorial, One, Zero};
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
        M: MIntDotProduct,
        F: FnMut(usize, MInt<M>),
    {
        let query = self.query;
        if query.is_empty() {
            return;
        }
        let max_n = query.iter().map(|&(_, n)| n).max().unwrap_or(0);
        let modulus = M::mod_into();
        debug_assert!(modulus > 2 && modulus % 2 == 1);
        debug_assert!(max_n < modulus);
        debug_assert!(query.iter().all(|&(m, n)| m <= n));

        let mut power2 = vec![MInt::<M>::one(); max_n + 1];
        for n in 1..=max_n {
            power2[n] = power2[n - 1] + power2[n - 1];
        }
        let mut points = Vec::with_capacity(query.len());
        for (i, &(m, n)) in query.iter().enumerate() {
            if m == n {
                f(i, power2[n]);
                continue;
            }
            let k = m.min(n - m - 1);
            let value = if k == 0 {
                MInt::one()
            } else if k * 2 + 1 == n {
                power2[n - 1]
            } else {
                points.push((k, n, i));
                continue;
            };
            f(i, if m == k { value } else { power2[n] - value });
        }
        if points.is_empty() {
            return;
        }
        let max_n = points.iter().map(|&(_, n, _)| n).max().unwrap();
        let MemorizedFactorial {
            fact: mut weighted_fact,
            inv_fact,
        } = MemorizedFactorial::<M>::new(max_n);
        let mut scale = (power2[max_n] + power2[max_n]).inv();
        for value in weighted_fact.iter_mut().rev() {
            *value *= scale;
            scale += scale;
        }
        Self::evaluate(
            &points,
            &weighted_fact,
            &inv_fact,
            &power2,
            &mut |i, value| {
                let (m, n) = query[i];
                f(i, if m < n - m { value } else { power2[n] - value });
            },
        );
    }

    pub fn solve(self) -> Vec<MInt<M>>
    where
        M: MIntDotProduct,
    {
        let mut ans = vec![MInt::zero(); self.query.len()];
        self.for_each(|i, x| ans[i] = x);
        ans
    }

    fn evaluate<F>(
        points: &[(usize, usize, usize)],
        weighted_fact: &[MInt<M>],
        inv_fact: &[MInt<M>],
        power2: &[MInt<M>],
        f: &mut F,
    ) where
        M: MIntDotProduct,
        F: FnMut(usize, MInt<M>),
    {
        let (_, n, _) = points[0];
        if points.iter().all(|&(_, l, _)| l == n) {
            let min_m = points.iter().map(|&(m, _, _)| m).min().unwrap();
            let max_m = points.iter().map(|&(m, _, _)| m).max().unwrap();
            let factor = (weighted_fact[n] + weighted_fact[n]) * power2[n];
            let mut sum = M::dot_product_rev(&inv_fact[..=min_m], &inv_fact[n - min_m..=n]);
            let mut prefix = Vec::with_capacity(max_m - min_m + 1);
            prefix.push(sum * factor);
            for m in min_m + 1..=max_m {
                sum += inv_fact[m] * inv_fact[n - m];
                prefix.push(sum * factor);
            }
            for &(m, _, i) in points.iter() {
                f(i, prefix[m - min_m]);
            }
            return;
        }
        let max_n = weighted_fact.len() - 1;
        let block = ((max_n as f64 / (points.len() as f64).sqrt()).round() as usize).max(1);
        let rows = max_n / block;
        // Store S(row * block, column * block) / 2^(row * block).
        let mut grid = vec![MInt::<M>::zero(); (rows + 1) * (rows + 2) / 2];
        for row in 0..=rows {
            let n = row * block;
            let factor = weighted_fact[n] + weighted_fact[n];
            let base = row * (row + 1) / 2;
            let mut sum = MInt::<M>::zero();
            for column in 0..=row / 2 {
                let k = column * block;
                if column > 0 {
                    sum += M::dot_product_rev(
                        &inv_fact[k - block..k],
                        &inv_fact[n - k + 1..=n - k + block],
                    );
                }
                let value = sum * factor;
                grid[base + row - column] = MInt::one() - value;
                grid[base + column] = value + inv_fact[k] * inv_fact[n - k] * factor;
            }
        }
        for &(m, n, i) in points.iter() {
            let row = ((n + block / 2) / block).min(rows);
            let column = (m + block / 2) / block;
            let start = row * block;
            let anchor = column * block;
            let distance = n.abs_diff(start) + m.abs_diff(anchor);
            let center = (n - 1) / 2;
            if m.min(center - m) <= distance {
                let factor = weighted_fact[n] + weighted_fact[n];
                let sum = if m <= center - m {
                    factor
                        * (inv_fact[n] + M::dot_product_rev(&inv_fact[1..=m], &inv_fact[n - m..n]))
                } else {
                    let half = weighted_fact[0];
                    let mid = n / 2;
                    let center_value = if n % 2 == 0 {
                        half - weighted_fact[n] * inv_fact[mid] * inv_fact[mid]
                    } else {
                        half
                    };
                    center_value
                        - factor
                            * M::dot_product_rev(
                                &inv_fact[m + 1..=center],
                                &inv_fact[n - center..n - m],
                            )
                };
                f(i, sum * power2[n]);
                continue;
            }
            let mut sum = grid[row * (row + 1) / 2 + column];
            if start < n {
                sum -= inv_fact[anchor]
                    * M::dot_product(
                        &weighted_fact[start..n],
                        &inv_fact[start - anchor..n - anchor],
                    );
            } else if n < start {
                sum += inv_fact[anchor]
                    * M::dot_product(
                        &weighted_fact[n..start],
                        &inv_fact[n - anchor..start - anchor],
                    );
            }
            let factor = weighted_fact[n] + weighted_fact[n];
            if anchor < m {
                sum += factor
                    * M::dot_product_rev(&inv_fact[anchor + 1..=m], &inv_fact[n - m..n - anchor]);
            } else if m < anchor {
                sum -= factor
                    * M::dot_product_rev(&inv_fact[m + 1..=anchor], &inv_fact[n - anchor..n - m]);
            }
            f(i, sum * power2[n]);
        }
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
        M: MIntDotProduct,
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

    pub fn solve(self) -> Vec<MInt<M>>
    where
        M: MIntDotProduct,
    {
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
        use crate::num::mint_basic::{DynModuloU32, Modulo1000000007};

        fn check<M>(max_n: usize)
        where
            M: MIntConvert<usize> + MIntDotProduct,
        {
            let mut rng = Xorshift::default();
            let mut row = vec![MInt::<M>::zero(); max_n + 1];
            row[0] = MInt::one();
            let mut query = vec![];
            for n in 0..=max_n {
                if n > 0 {
                    for m in (1..=n).rev() {
                        row[m] = row[m] + row[m - 1];
                    }
                }
                let mut prefix = row[..=n].to_vec();
                for m in 1..=n {
                    prefix[m] = prefix[m] + prefix[m - 1];
                }
                if n <= 32 || n.is_power_of_two() || n == max_n {
                    for first in [0, n / 4] {
                        let mut solver = BinomialPrefixSum::<M>::new();
                        for m in (first..=n - first).rev() {
                            solver.push(n, m);
                        }
                        let expected: Vec<_> =
                            prefix[first..=n - first].iter().rev().copied().collect();
                        assert_eq!(solver.solve(), expected);
                    }
                }
                let ms = if n <= 32 || n == max_n {
                    (0..=n + 1).collect()
                } else {
                    vec![
                        0,
                        1,
                        n / 2 - 1,
                        n / 2,
                        n / 2 + 1,
                        n - 1,
                        n,
                        n + 1,
                        usize::MAX,
                        rng.random(0..=n),
                    ]
                };
                for m in ms {
                    query.push((n, m, prefix[m.min(n)]));
                }
            }
            rng.shuffle(&mut query);
            query.extend_from_within(..query.len().min(128));
            let mut solver = BinomialPrefixSum::<M>::with_capacity(query.len());
            let mut callback_solver = BinomialPrefixSum::<M>::with_capacity(query.len());
            for (i, &(n, m, _)) in query.iter().enumerate() {
                assert_eq!(solver.push(n, m), i);
                assert_eq!(callback_solver.push(n, m), i);
            }
            let expected: Vec<_> = query.iter().map(|&(_, _, x)| x).collect();
            assert_eq!(solver.solve(), expected);
            let mut visits = vec![0; query.len()];
            callback_solver.for_each(|i, x| {
                assert_eq!(x, expected[i]);
                visits[i] += 1;
            });
            assert!(visits.iter().all(|&count| count == 1));

            for &(n, m, expected) in query.iter().step_by(128) {
                for count in [1, 3] {
                    let mut solver = BinomialPrefixSum::<M>::new();
                    for _ in 0..count {
                        solver.push(n, m);
                    }
                    assert_eq!(solver.solve(), vec![expected; count]);
                }
            }
            assert!(BinomialPrefixSum::<M>::new().solve().is_empty());
            let mut visits = 0;
            BinomialPrefixSum::<M>::new().for_each(|_, _| visits += 1);
            assert_eq!(visits, 0);
        }
        check::<Modulo998244353>(4096);
        check::<Modulo1000000007>(4096);
        for modulus in [3, 17, 97] {
            DynModuloU32::set_mod(modulus);
            check::<DynModuloU32>(modulus as usize - 1);
        }
        DynModuloU32::set_mod(1_000_000_007);
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
                let inputs: Vec<_> = (0..=N)
                    .flat_map(|n| (0..=N * 2).map(move |m| (n, m)))
                    .chain((0..Q).map(|_| rng.random((0..=N, 0..=N * 2))))
                    .collect();
                for (i, (n, m)) in inputs.into_iter().enumerate() {
                    let coef = match i % 3 {
                        0 => [MInt998244353::zero(); K],
                        1 => [MInt998244353::one(); K],
                        _ => std::array::from_fn(|_| rng.random(..)),
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
                assert!(
                    BinomialPolynomialPrefixSum::<Modulo998244353, K>::new()
                        .solve()
                        .is_empty()
                );
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
