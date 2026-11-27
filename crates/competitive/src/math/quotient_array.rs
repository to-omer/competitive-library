use super::{AbelianGroup, Invertible, PrimeList, Ring, Zero};
use std::ops::{Index, IndexMut};

/// Values at the distinct positive quotients of `n`, stored in increasing order.
#[derive(Debug, Clone)]
pub struct QuotientArray<T> {
    n: u64,
    root: usize,
    data: Vec<T>,
}

impl<T> QuotientArray<T> {
    /// Iterates over the distinct quotients of a positive `n` in increasing order.
    pub fn quotients(n: u64) -> impl DoubleEndedIterator<Item = u64> {
        assert!(n > 0);
        let root = n.isqrt();
        let large = n / (root + 1);
        (1..=root).chain((1..=large).rev().map(move |d| n / d))
    }

    pub fn from_fn(n: u64, f: impl FnMut(u64) -> T) -> Self {
        Self {
            n,
            root: n.isqrt() as usize,
            data: Self::quotients(n).map(f).collect(),
        }
    }

    pub fn zeros(n: u64) -> Self
    where
        T: Zero,
    {
        Self::from_fn(n, |_| T::zero())
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the position of a member of the quotient set in ascending order.
    pub fn quotient_index(&self, value: u64) -> usize {
        if value <= self.root as u64 {
            assert!(value > 0);
            value as usize - 1
        } else {
            let d = self.n / value;
            assert!(d != 0 && self.n / d == value, "value is not a quotient");
            self.data.len() - d as usize
        }
    }

    pub fn map<U>(&self, f: impl FnMut(&T) -> U) -> QuotientArray<U> {
        QuotientArray {
            n: self.n,
            root: self.root,
            data: self.data.iter().map(f).collect(),
        }
    }

    /// Converts inclusive prefix sums of a completely multiplicative function to prime sums.
    /// `mul(weight, sum)` multiplies a function value by a sum in the coefficient ring.
    pub fn lucy_dp<G>(mut self, mul: impl Fn(&T, &T) -> T) -> Self
    where
        T: Clone,
        G: AbelianGroup<T = T>,
    {
        let one = self.data[0].clone();
        for x in &mut self.data {
            G::rinv_operate_assign(x, &one);
        }
        self.sieve::<G, true>(0, &mul, |_, _, first, previous| mul(first, previous));
        self
    }

    /// Converts prime sums to inclusive prefix sums of a multiplicative function.
    /// `prime_power(p, e)` supplies its value at prime powers with `e >= 2`.
    pub fn min_25_sieve<R>(mut self, mut prime_power: impl FnMut(u64, u32) -> T) -> Self
    where
        T: Clone,
        R: Ring<T = T, Additive: Invertible>,
    {
        self.sieve::<R::Additive, false>(0, R::mul, |p, e, _, _| prime_power(p, e));
        let one = R::one();
        for x in &mut self.data {
            R::add_assign(x, &one);
        }
        self
    }

    /// Returns the multiplicative prefix sum at `n` from prime sums and the values
    /// `prime_power(p, e)` at prime powers with `e >= 2`.
    pub fn multiplicative_sum<R>(mut self, mut prime_power: impl FnMut(u64, u32) -> T) -> T
    where
        T: Clone,
        R: Ring<T = T, Additive: Invertible>,
    {
        let cutoff = self.root.isqrt().isqrt().min(43);
        let powers: Vec<Vec<_>> = PrimeList::new(cutoff as u32)
            .primes()
            .map(|p| {
                let p = u64::from(p);
                let i = p as usize - 1;
                let mut values = vec![(p, R::sub(&self.data[i], &self.data[i - 1]))];
                let mut power = p;
                while power <= self.n / p {
                    power *= p;
                    values.push((power, prime_power(p, values.len() as u32 + 1)));
                }
                values
            })
            .collect();
        self.sieve::<R::Additive, false>(cutoff, R::mul, |p, e, _, _| prime_power(p, e));
        self.smooth_sum::<R>(&powers, 1, self.n, cutoff, &self.data[cutoff - 1])
    }

    fn smooth_sum<R>(
        &self,
        powers: &[Vec<(u64, T)>],
        d: u64,
        limit: u64,
        cutoff: usize,
        before: &T,
    ) -> T
    where
        R: Ring<T = T, Additive: Invertible>,
    {
        let mut sum = R::one();
        if limit > cutoff as u64 {
            let i = if limit <= self.root as u64 {
                limit as usize - 1
            } else {
                self.data.len() - d as usize
            };
            R::add_assign(&mut sum, &R::sub(&self.data[i], before));
        }
        for (j, powers_of_p) in powers.iter().enumerate() {
            if powers_of_p[0].0 > limit {
                break;
            }
            for (power, weight) in powers_of_p {
                if *power > limit {
                    break;
                }
                let rest = self.smooth_sum::<R>(
                    &powers[j + 1..],
                    d * power,
                    limit / power,
                    cutoff,
                    before,
                );
                R::add_assign(&mut sum, &R::mul(weight, &rest));
            }
        }
        sum
    }

    fn sieve<G, const REMOVE: bool>(
        &mut self,
        cutoff: usize,
        mul: impl Fn(&T, &T) -> T,
        mut next: impl FnMut(u64, u32, &T, &T) -> T,
    ) where
        T: Clone,
        G: AbelianGroup<T = T>,
    {
        let primes: Vec<_> = PrimeList::new(self.root as u32)
            .primes()
            .map(|p| {
                let i = p as usize - 1;
                (
                    u64::from(p),
                    G::rinv_operate(&self.data[i], &self.data[i - 1]),
                )
            })
            .collect();
        let fourth = self.root.isqrt();
        let small_problem = self.root <= 128;
        let dense = if small_problem {
            self.root
        } else {
            (fourth.isqrt() / 2).max(2)
        };
        let end = if small_problem {
            primes.len()
        } else {
            primes.partition_point(|x| x.0 <= fourth as u64)
        };
        let (small, large) = primes.split_at(end);
        if !REMOVE && !large.is_empty() {
            self.large_primes::<G, REMOVE>(large, &mul, &mut next);
        }
        let max_k = self.cofactor_limit(dense as u64 + 1);
        let mut least = Vec::new();
        let large_len = self.data.len() - self.root;
        let quotients: Vec<_> = (1..=large_len).map(|d| self.n / d as u64).collect();
        let mut powers = Vec::new();
        let bands = small.last().map_or(0, |x| x.0.ilog2());
        for i in 1..=bands {
            let shift = if REMOVE { i } else { bands + 1 - i };
            let first = 1u64 << shift;
            let start = small.partition_point(|x| x.0 < first || x.0 <= cutoff as u64);
            let end = small.partition_point(|x| x.0 < first * 2);
            if start >= end {
                continue;
            }
            let band = &small[start..end];
            if first <= dense as u64 {
                for i in 0..band.len() {
                    let (p, weight) = &band[if REMOVE { i } else { band.len() - 1 - i }];
                    powers.clear();
                    powers.push((*p, weight.clone()));
                    if !REMOVE {
                        while let Some((power, old)) = powers.last() {
                            if *power > self.n / p {
                                break;
                            }
                            powers
                                .push((power * p, next(*p, powers.len() as u32 + 1, weight, old)));
                        }
                    }
                    self.dense_prime::<G, REMOVE>(&powers, &quotients, &mul);
                }
            } else {
                if least.is_empty() {
                    // A composite cofactor up to sqrt(n) has a least prime factor smaller than 2^16.
                    least.resize(max_k + 1, u16::MAX);
                    for &(p, _) in small.iter().rev() {
                        if p * p <= max_k as u64 {
                            for k in (p as usize * p as usize..=max_k).step_by(p as usize) {
                                least[k] = p as u16;
                            }
                        }
                    }
                }
                let mut groups = band.chunks(32);
                while let Some(batch) = if REMOVE {
                    groups.next()
                } else {
                    groups.next_back()
                } {
                    self.prime_band::<G, REMOVE>(batch, &least, &mul, &mut next);
                }
            }
        }
        if REMOVE && !large.is_empty() {
            self.large_primes::<G, REMOVE>(large, &mul, &mut next);
        }
    }

    fn dense_prime<G, const REMOVE: bool>(
        &mut self,
        powers: &[(u64, T)],
        quotients: &[u64],
        mul: &impl Fn(&T, &T) -> T,
    ) where
        T: Clone,
        G: AbelianGroup<T = T>,
    {
        let (p, weight) = &powers[0];
        let p = *p;
        let before = self.data[p as usize - if REMOVE { 2 } else { 1 }].clone();
        let len = self.data.len();
        let large = quotients.len();
        let delta = |v: &T, e: usize| {
            let mut x = mul(&powers[e].1, &G::rinv_operate(v, &before));
            if !REMOVE {
                G::operate_assign(&mut x, &powers[e + 1].1);
            }
            x
        };
        let last = (self.n / (p * p)).min(large as u64) as usize;
        let split = last.min(large / p as usize);
        let repeated = if REMOVE {
            0
        } else {
            (self.n / (p * p) / p).min(last as u64) as usize
        };
        for d in 1..=repeated {
            let q = quotients[d - 1];
            let mut x = G::unit();
            for (e, pair) in powers.windows(2).enumerate() {
                if pair[1].0 > q {
                    break;
                }
                let power = pair[0].0;
                let index = if power <= (large / d) as u64 {
                    len - d * power as usize
                } else {
                    (q / power) as usize - 1
                };
                G::operate_assign(&mut x, &delta(&self.data[index], e));
            }
            G::operate_assign(&mut self.data[len - d], &x);
        }
        for d in repeated + 1..=split {
            let x = delta(&self.data[len - d * p as usize], 0);
            update::<G, REMOVE>(&mut self.data[len - d], &x);
        }
        for d in repeated.max(split) + 1..=last {
            let q = quotients[d - 1];
            let v = q / p;
            let x = delta(&self.data[v as usize - 1], 0);
            update::<G, REMOVE>(&mut self.data[len - d], &x);
        }
        for v in (p..=self.root as u64 / p).rev() {
            let mut x = mul(
                weight,
                &G::rinv_operate(&self.data[v as usize - 1], &before),
            );
            if !REMOVE {
                G::operate_assign(&mut x, &powers[1].1);
                for e in 1..powers.len() - 1 {
                    if powers[e].0 > v {
                        break;
                    }
                    let index = (v / powers[e - 1].0) as usize - 1;
                    G::operate_assign(&mut x, &delta(&self.data[index], e));
                }
            }
            let from = (p * v) as usize - 1;
            let end = from.saturating_add(p as usize).min(self.root);
            for cell in &mut self.data[from..end] {
                update::<G, REMOVE>(cell, &x);
            }
        }
    }

    fn cofactor_limit(&self, prime: u64) -> usize {
        ((self.n / prime).isqrt() * 2).min(self.root as u64) as usize
    }

    fn prime_band<G, const REMOVE: bool>(
        &mut self,
        primes: &[(u64, T)],
        least: &[u16],
        mul: &impl Fn(&T, &T) -> T,
        next: &mut impl FnMut(u64, u32, &T, &T) -> T,
    ) where
        T: Clone,
        G: AbelianGroup<T = T>,
    {
        let first = primes[0].0;
        let rough = if REMOVE {
            first
        } else {
            primes.last().unwrap().0 + 1
        };
        let max_k = self.cofactor_limit(first);
        let keys = (rough as usize..=max_k)
            .filter(|&k| least[k] == u16::MAX || u64::from(least[k]) >= rough)
            .map(|k| {
                (
                    k as u64,
                    G::rinv_operate(&self.data[k - 1], &self.data[k - 2]),
                )
            })
            .collect();
        let offset = self.ceiling_index(first * first);
        let mut band = PrimeBand {
            input: self,
            rough,
            max_k,
            keys,
            offset,
            difference: vec![G::unit(); self.data.len() - offset],
            high: Vec::new(),
        };
        // Removing a block uses its squarefree Möbius weights. Inserting it uses all its prime powers.
        band.products::<G, REMOVE>(primes, 1, self.n, None, mul, next);
        let PrimeBand {
            difference, high, ..
        } = band;
        let mut sum = G::unit();
        for (x, change) in self.data[offset..].iter_mut().zip(difference) {
            G::operate_assign(&mut sum, &change);
            G::operate_assign(x, &sum);
        }
        for (x, change) in self.data.iter_mut().rev().zip(high) {
            G::operate_assign(x, &change);
        }
    }

    fn ceiling_index(&self, x: u64) -> usize {
        if x <= self.root as u64 {
            x as usize - 1
        } else {
            self.data.len() - (self.n / x) as usize
        }
    }

    fn large_primes<G, const REMOVE: bool>(
        &mut self,
        primes: &[(u64, T)],
        mul: &impl Fn(&T, &T) -> T,
        next_weight: &mut impl FnMut(u64, u32, &T, &T) -> T,
    ) where
        T: Clone,
        G: AbelianGroup<T = T>,
    {
        let n = self.n;
        let root = self.root;
        let len = self.data.len();
        let large = len - root;
        let lo = &self.data[..root];
        let first = primes[0].0;
        // Four primes above the fourth root exceed n, and a triple cannot contain a prime above the square root.
        let mut pairs = vec![G::unit(); large];
        let mut squares = Vec::with_capacity(primes.len());
        let mut explicit_end = primes.len();
        for (i, (p, weight)) in primes.iter().enumerate() {
            let p = *p;
            let m = n / p;
            let square = next_weight(p, 2, weight, weight);
            G::operate_assign(&mut pairs[(m / p) as usize - 1], &square);
            squares.push(square);
            while explicit_end > 0 && (explicit_end as u64) * primes[explicit_end - 1].0 > m {
                explicit_end -= 1;
            }
            if let Some(&(bound, _)) = primes.get(explicit_end.max(i + 1)) {
                let split = (m / root as u64) as usize;
                let end = (m / bound) as usize;
                G::operate_assign(&mut pairs[split - 1], &mul(weight, &lo[root - 1]));
                G::rinv_operate_assign(&mut pairs[end - 1], &mul(weight, &lo[bound as usize - 2]));
            }
            for (q, other_weight) in &primes[i + 1..explicit_end.max(i + 1)] {
                G::operate_assign(&mut pairs[(m / q) as usize - 1], &mul(weight, other_weight));
            }
        }
        for i in (1..pairs.len()).rev() {
            pairs[i - 1] = G::operate(&pairs[i - 1], &pairs[i]);
        }
        explicit_end = primes.len();
        for (i, (p, weight)) in primes.iter().enumerate() {
            let m = n / p;
            while explicit_end > 0 && (explicit_end as u64) * primes[explicit_end - 1].0 > m {
                explicit_end -= 1;
            }
            let Some(&(bound, _)) = primes.get(explicit_end.max(i + 1)) else {
                continue;
            };
            let split = (m / root as u64) as usize;
            for d in split + 1..=(m / bound) as usize {
                let q = (m / d as u64) as usize;
                let x = &lo[q - 1];
                G::operate_assign(&mut pairs[d - 1], &mul(weight, x));
            }
        }
        let mut triples = vec![G::unit(); (n / first / first / first) as usize];
        for (i, (p, weight)) in primes.iter().enumerate() {
            if *p > n / p / p {
                break;
            }
            let cube = next_weight(*p, 3, weight, &squares[i]);
            let m = n / p;
            let mut end = primes.partition_point(|x| x.0 <= m.isqrt());
            for d in 1..=(m / p / p) as usize {
                let v = m / d as u64;
                while primes[end - 1].0 * primes[end - 1].0 > v {
                    end -= 1;
                }
                let prefix = i + 1 < end - (i + 1);
                let partners = if prefix { 0..i + 1 } else { i + 1..end };
                let mut sum = G::unit();
                for j in partners {
                    let (q, other_weight) = &primes[j];
                    let other = G::rinv_operate(&lo[(v / q) as usize - 1], &lo[*q as usize - 1]);
                    G::operate_assign(
                        &mut sum,
                        &G::operate(&mul(other_weight, &other), &squares[j]),
                    );
                }
                if prefix {
                    sum = G::rinv_operate(&pairs[d * *p as usize - 1], &sum);
                }
                let higher = G::rinv_operate(&lo[(v / p) as usize - 1], &lo[*p as usize - 1]);
                let x = G::operate(&mul(weight, &sum), &mul(&squares[i], &higher));
                G::operate_assign(&mut triples[d - 1], &G::operate(&x, &cube));
            }
        }
        for (x, t) in pairs.iter_mut().zip(triples) {
            G::operate_assign(x, &t);
        }
        let before = self.data[root - 1].clone();
        for i in 0..large {
            let d = if REMOVE { large - i } else { i + 1 };
            let mut total = pairs[d - 1].clone();
            for (p, weight) in primes {
                if *p > (large / d) as u64 {
                    break;
                }
                let other = G::rinv_operate(&self.data[len - d * *p as usize], &before);
                G::operate_assign(&mut total, &mul(weight, &other));
            }
            update::<G, REMOVE>(&mut self.data[len - d], &total);
        }
    }
}

struct PrimeBand<'a, T> {
    input: &'a QuotientArray<T>,
    rough: u64,
    max_k: usize,
    keys: Vec<(u64, T)>,
    offset: usize,
    difference: Vec<T>,
    high: Vec<T>,
}

impl<T> PrimeBand<'_, T> {
    fn products<G, const REMOVE: bool>(
        &mut self,
        primes: &[(u64, T)],
        product: u64,
        limit: u64,
        parent: Option<(&T, bool)>,
        mul: &impl Fn(&T, &T) -> T,
        next: &mut impl FnMut(u64, u32, &T, &T) -> T,
    ) where
        T: Clone,
        G: AbelianGroup<T = T>,
    {
        for (j, (p, w)) in primes.iter().enumerate() {
            if *p > limit {
                break;
            }
            let mut t = product;
            let mut m = limit;
            let mut weight = w.clone();
            let mut e = 1;
            loop {
                t *= p;
                m /= p;
                let (val, subtract) = match parent {
                    Some((x, negative)) => (mul(x, &weight), REMOVE && !negative),
                    None => (weight.clone(), REMOVE),
                };
                let prime = parent.is_none() && e == 1;
                if subtract {
                    self.term::<G, true>(t, m, &val, prime, mul);
                } else {
                    self.term::<G, false>(t, m, &val, prime, mul);
                }
                if *p > m {
                    break;
                }
                self.products::<G, REMOVE>(
                    &primes[j + 1..],
                    t,
                    m,
                    Some((&val, subtract)),
                    mul,
                    next,
                );
                if REMOVE {
                    break;
                }
                e += 1;
                weight = next(*p, e, w, &weight);
            }
        }
    }

    fn term<G, const SUB: bool>(
        &mut self,
        product: u64,
        limit: u64,
        weight: &T,
        prime: bool,
        mul: &impl Fn(&T, &T) -> T,
    ) where
        T: Clone,
        G: AbelianGroup<T = T>,
    {
        let input = self.input;
        let root = input.root as u64;
        let len = input.data.len();
        if !prime {
            let pos = if product <= root {
                product as usize - 1
            } else {
                len - limit as usize
            };
            update::<G, SUB>(&mut self.difference[pos - self.offset], weight);
        }
        if limit < self.rough {
            return;
        }
        // Enumerate small rough cofactors once; for the tail, read their prefix sum at each remaining quotient.
        let small = root / product;
        let split = if small < self.rough {
            0
        } else {
            self.keys.partition_point(|x| x.0 <= small)
        };
        let mut left = split;
        let max_count = usize::try_from(limit / self.rough).unwrap_or(usize::MAX);
        let mut right = self.keys.len().min(max_count);
        while left < right {
            let mid = left + (right - left) / 2;
            if (mid as u64 + 1) * self.keys[mid].0 <= limit {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        for (k, v) in &self.keys[..split] {
            update::<G, SUB>(
                &mut self.difference[(product * k) as usize - 1 - self.offset],
                &mul(weight, v),
            );
        }
        for (k, v) in &self.keys[split..left] {
            let d = limit / k;
            update::<G, SUB>(
                &mut self.difference[len - d as usize - self.offset],
                &mul(weight, v),
            );
        }
        let cut = if left == self.keys.len() {
            self.max_k as u64
        } else if left == 0 {
            self.rough - 1
        } else {
            self.keys[left - 1].0
        };
        let cut = cut.max(self.rough - 1).max(small);
        let end = (limit / (cut + 1)) as usize;
        if end == 0 {
            return;
        }
        let previous = mul(weight, &input.data[cut as usize - 1]);
        if SUB {
            G::operate_assign(&mut self.difference[len - end - self.offset], &previous);
        } else {
            G::rinv_operate_assign(&mut self.difference[len - end - self.offset], &previous);
        }
        if self.high.len() < end {
            self.high.resize(end, G::unit());
        }
        let big = (((len - input.root) as u64 / product).min(end as u64)) as usize;
        for (d, x) in self.high[..big].iter_mut().enumerate() {
            update::<G, SUB>(
                x,
                &mul(weight, &input.data[len - (d + 1) * product as usize]),
            );
        }
        for d in big + 1..=end {
            update::<G, SUB>(
                &mut self.high[d - 1],
                &mul(weight, &input.data[(limit / d as u64) as usize - 1]),
            );
        }
    }
}

fn update<G, const SUB: bool>(value: &mut G::T, delta: &G::T)
where
    G: AbelianGroup,
{
    if SUB {
        G::rinv_operate_assign(value, delta);
    } else {
        G::operate_assign(value, delta);
    }
}

impl<T> Index<u64> for QuotientArray<T> {
    type Output = T;
    fn index(&self, value: u64) -> &T {
        &self.data[self.quotient_index(value)]
    }
}

impl<T> IndexMut<u64> for QuotientArray<T> {
    fn index_mut(&mut self, value: u64) -> &mut T {
        let i = self.quotient_index(value);
        &mut self.data[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AddMulOperation, AdditiveOperation, ArrayOperation},
        math::PrimeTable,
        num::montgomery::MInt469762049 as M,
        tools::{Xorshift, testutil::sample_usize},
    };

    fn inputs(max: usize, samples: usize) -> Vec<usize> {
        let mut cases = sample_usize(&mut Xorshift::default(), 512, 1..=max, samples);
        for k in [2usize, 3, 5, 7, 11, 17, 31, 32, 127, 128, 129] {
            for boundary in [k * k, k * (k + 1), k.pow(3), k.pow(4)] {
                cases.extend((boundary - 1..=boundary + 1).filter(|&n| n <= max));
            }
        }
        cases.sort_unstable();
        cases.dedup();
        cases
    }

    #[test]
    fn quotient_indices() {
        for n in 1..=1024 {
            let mut quotients: Vec<_> = (1..=n).rev().map(|i| n / i).collect();
            quotients.dedup();
            let array = QuotientArray::from_fn(n, |q| q);
            assert_eq!(array.as_slice(), quotients);
            let mut indices = QuotientArray::zeros(n);
            for (i, &q) in quotients.iter().enumerate() {
                assert_eq!(array.quotient_index(q), i);
                indices[q] = i;
            }
            assert!(indices.as_slice().iter().copied().eq(0..quotients.len()));
        }
    }

    #[test]
    fn prime_sums() {
        const MAX: usize = 4_000_000;
        let table = PrimeTable::new(MAX as u32);
        let mut expected = vec![[0i64; 2]; MAX + 1];
        for k in 1..=MAX {
            expected[k] = expected[k - 1];
            if table.is_prime(k as u32) {
                expected[k][0] += 1;
                expected[k][1] += [0, 1, -1][k % 3];
            }
        }
        let mul = |a: &[i64; 2], b: &[i64; 2]| [a[0] * b[0], a[1] * b[1]];
        for n in inputs(MAX, 600) {
            let sums = QuotientArray::from_fn(n as u64, |q| [q as i64, i64::from(q % 3 == 1)])
                .lucy_dp::<ArrayOperation<AdditiveOperation<_>, 2>>(mul);
            let expected = QuotientArray::from_fn(n as u64, |q| expected[q as usize]);
            assert_eq!(sums.as_slice(), expected.as_slice(), "n = {n}");
        }
    }

    #[test]
    fn divisor_sums() {
        const MAX: usize = 2_000_000;
        let mut sigma = vec![0u64; MAX + 1];
        for d in 1..=MAX {
            for k in (d..=MAX).step_by(d) {
                sigma[k] += d as u64;
            }
        }
        let table = PrimeTable::new(MAX as u32);
        let mut primes = vec![0u64; MAX + 1];
        for k in 1..=MAX {
            sigma[k] += sigma[k - 1];
            primes[k] = primes[k - 1];
            if table.is_prime(k as u32) {
                primes[k] += k as u64 + 1;
            }
        }
        let prime_power = |p, e| M::from((0..e).fold(1u64, |sum, _| sum * p + 1));
        for n in inputs(MAX, 500) {
            let sums = QuotientArray::from_fn(n as u64, |q| M::from(primes[q as usize]));
            let all = sums.clone().min_25_sieve::<AddMulOperation<_>>(prime_power);
            let expected = QuotientArray::from_fn(n as u64, |q| M::from(sigma[q as usize]));
            assert_eq!(all.as_slice(), expected.as_slice(), "n = {n}");
            let top = sums.multiplicative_sum::<AddMulOperation<_>>(prime_power);
            assert_eq!(top, expected[n as u64], "n = {n}");
        }
    }

    #[test]
    fn square_indicator() {
        let prime_power = |_, e: u32| i64::from(e.is_multiple_of(2));
        let boundary = 257u64.pow(4);
        let cases = inputs(8usize.pow(8) + 1, 120)
            .into_iter()
            .map(|n| n as u64)
            .chain(boundary - 1..=boundary + 1);
        for n in cases {
            let sums = QuotientArray::<i64>::zeros(n);
            let all = sums.clone().min_25_sieve::<AddMulOperation<_>>(prime_power);
            let expected = QuotientArray::from_fn(n, |q| q.isqrt() as i64);
            assert_eq!(all.as_slice(), expected.as_slice(), "n = {n}");
            let top = sums.multiplicative_sum::<AddMulOperation<_>>(prime_power);
            assert_eq!(top, expected[n], "n = {n}");
        }
    }
}
