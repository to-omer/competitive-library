use super::{with_prime_list, Group, Invertible, One, Ring, Zero};
use std::ops::{Index, IndexMut};

/// store with index $\{\lfloor\frac{n}{i}\rfloor \mid i=1,2,\ldots,n\}$
#[derive(Debug, Clone)]
pub struct QuotientArray<T> {
    n: u64,
    isqrtn: u64,
    data: Vec<T>,
}

impl<T> QuotientArray<T>
where
    T: Zero,
{
    pub fn zeros(n: u64) -> Self {
        Self::from_fn(n, |_| T::zero())
    }
}

impl<T> QuotientArray<T> {
    pub fn index_iter(n: u64, isqrtn: u64) -> impl Iterator<Item = u64> {
        (1..=isqrtn)
            .map(move |i| n / i)
            .chain((1..n / isqrtn).rev())
    }

    pub fn map<U>(&self, f: impl FnMut(&T) -> U) -> QuotientArray<U> {
        let data = self.data.iter().map(f).collect();
        QuotientArray {
            n: self.n,
            isqrtn: self.isqrtn,
            data,
        }
    }

    pub fn quotient_index(&self, i: u64) -> usize {
        assert!(
            i <= self.n,
            "index out of bounds: the len is {} but the index is {}",
            self.n,
            i
        );
        assert_ne!(i, 0, "index out of bounds: the index is 0");
        if i <= self.isqrtn {
            self.data.len() - i as usize
        } else {
            (self.n / i) as usize - 1
        }
    }

    pub fn from_fn(n: u64, f: impl FnMut(u64) -> T) -> Self {
        let isqrtn = (n as f64).sqrt().floor() as u64;
        let data = Self::index_iter(n, isqrtn).map(f).collect();
        Self { n, isqrtn, data }
    }

    /// convert $\sum_{i\leq n} f(i)$ to $\sum_{i\leq n, i\text{ is prime}} f(i)$
    ///
    /// constraints: $\mathrm{mul_p}(f(x))=f(px)$
    pub fn lucy_dp<G>(mut self, mut mul_p: impl FnMut(T, u64) -> T) -> Self
    where
        G: Group<T = T>,
    {
        with_prime_list(self.isqrtn, |pl| {
            for &p in pl.primes_lte(self.isqrtn) {
                let k = self.quotient_index(p - 1);
                let p2 = p * p;
                for (i, q) in Self::index_iter(self.n, self.isqrtn).enumerate() {
                    if q < p2 {
                        break;
                    }
                    let diff = mul_p(G::rinv_operate(&self[q / p], &self.data[k]), p);
                    G::rinv_operate_assign(&mut self.data[i], &diff);
                }
            }
        });
        self
    }

    /// convert $\sum_{i\leq n, i\text{ is prime}} f(i)$ to $\sum_{i\leq n} f(i)$
    pub fn min_25_sieve<R>(&self, f: impl Fn(u64, u32) -> T) -> Self
    where
        T: Clone + One,
        R: Ring<T = T>,
        R::Additive: Invertible,
    {
        let mut dp = self.clone();
        with_prime_list(self.isqrtn, |pl| {
            for &p in pl.primes_lte(self.isqrtn).iter().rev() {
                let k = self.quotient_index(p);
                for (i, q) in Self::index_iter(self.n, self.isqrtn).enumerate() {
                    let mut pc = p;
                    if pc * p > q {
                        break;
                    }
                    let mut c = 1;
                    while q / p >= pc {
                        let x = R::mul(&f(p, c), &(R::sub(&dp[q / pc], &self.data[k])));
                        let x = R::add(&x, &f(p, c + 1));
                        dp.data[i] = R::add(&dp.data[i], &x);
                        c += 1;
                        pc *= p;
                    }
                }
            }
        });
        for x in &mut dp.data {
            *x = R::add(x, &T::one());
        }
        dp
    }
}

impl<T> Index<u64> for QuotientArray<T> {
    type Output = T;
    fn index(&self, i: u64) -> &Self::Output {
        unsafe { self.data.get_unchecked(self.quotient_index(i)) }
    }
}

impl<T> IndexMut<u64> for QuotientArray<T> {
    fn index_mut(&mut self, index: u64) -> &mut Self::Output {
        let i = self.quotient_index(index);
        unsafe { self.data.get_unchecked_mut(i) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AddMulOperation, AdditiveOperation, ArrayOperation},
        math::{PrimeList, PrimeTable},
        tools::Xorshift,
    };

    #[test]
    fn prime_count() {
        let mut rng = Xorshift::default();
        let pl = PrimeList::new(100_000);
        for n in 1..=100 {
            let n = if n <= 10 { n } else { rng.gen(1..10_000) };
            let qa = QuotientArray::from_fn(n, |i| i as i64 - 1);
            let qa = qa.lucy_dp::<AdditiveOperation<_>>(|x, _p| x);
            assert_eq!(pl.primes_lte(n).len(), qa[n] as usize);
        }
    }

    #[test]
    fn divisor_sum() {
        let mut rng = Xorshift::default();
        let pt = PrimeTable::new(10_000);
        for n in 1..=100 {
            let n = if n <= 10 { n } else { rng.gen(1..10_000) };
            let qa = QuotientArray::from_fn(n, |i| [i as i64, i as i64 * (i as i64 + 1) / 2])
                .map(|[x, y]| [x - 1, y - 1]);
            let qa = qa
                .lucy_dp::<ArrayOperation<AdditiveOperation<_>, 2>>(|[x, y], p| [x, y * p as i64]);
            let qa = qa.map(|[x, y]| x + y);
            let qa = qa.min_25_sieve::<AddMulOperation<_>>(|p, c| {
                let mut x = 1;
                let mut s = 1;
                for _ in 0..c {
                    x *= p as i64;
                    s += x;
                }
                s
            });
            assert_eq!(
                (1..=n)
                    .flat_map(|i| pt.divisors(i as _))
                    .map(|d| d as u64)
                    .sum::<u64>(),
                qa[n] as u64
            );
        }
    }
}
