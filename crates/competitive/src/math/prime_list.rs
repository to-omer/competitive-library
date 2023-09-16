use std::{cell::UnsafeCell, mem::replace, slice::Iter};

#[derive(Debug, Clone, Default)]
pub struct PrimeList {
    primes: Vec<u64>,
    max_n: u64,
}
impl PrimeList {
    pub fn new(max_n: u64) -> Self {
        let mut self_: Self = Default::default();
        self_.reserve(max_n);
        self_
    }
    pub fn primes(&self) -> &[u64] {
        self.primes.as_slice()
    }
    pub fn primes_lte(&self, n: u64) -> &[u64] {
        assert!(n <= self.max_n, "expected `n={} <= {}`", n, self.max_n);
        let i = self.primes.binary_search(&n).unwrap_or_else(|i| i);
        &self.primes[..i]
    }
    pub fn is_prime(&self, n: u64) -> bool {
        assert!(n <= self.max_n, "expected `n={} <= {}`", n, self.max_n);
        self.primes.binary_search(&n).is_ok()
    }
    pub fn trial_division(&self, n: u64) -> PrimeListTrialDivision<'_> {
        let bound = self.max_n.saturating_mul(self.max_n);
        assert!(n <= bound, "expected `n={} <= {}`", n, bound);
        PrimeListTrialDivision {
            primes: self.primes.iter(),
            n,
        }
    }
    pub fn prime_factors(&self, n: u64) -> Vec<(u64, u32)> {
        self.trial_division(n).collect()
    }
    pub fn count_divisors(&self, n: u64) -> u64 {
        let mut divisor_cnt = 1u64;
        for (_, cnt) in self.trial_division(n) {
            divisor_cnt *= cnt as u64 + 1;
        }
        divisor_cnt
    }
    pub fn divisors(&self, n: u64) -> Vec<u64> {
        let mut d = vec![1u64];
        for (p, c) in self.trial_division(n) {
            let k = d.len();
            let mut acc = p;
            for _ in 0..c {
                for i in 0..k {
                    d.push(d[i] * acc);
                }
                acc *= p;
            }
        }
        d.sort_unstable();
        d
    }
    /// list primes less than or equal to `max_n` by segmented sieve
    pub fn reserve(&mut self, max_n: u64) {
        if max_n <= self.max_n || max_n < 2 {
            return;
        }

        if self.primes.is_empty() {
            self.primes.push(2);
            self.max_n = 2;
        }
        if max_n == 2 {
            return;
        }

        let max_n = (max_n + 1) / 2 * 2; // odd
        let sqrt_n = ((max_n as f64).sqrt() as usize + 1) / 2 * 2; // even
        let mut table = Vec::with_capacity(sqrt_n >> 1);
        if self.max_n < sqrt_n as u64 {
            let start = (self.max_n as usize + 1) | 1; // odd
            let end = sqrt_n + 1;
            let sqrt_end = (sqrt_n as f64).sqrt() as usize;
            let plen = self.primes[1..]
                .binary_search(&(sqrt_end as u64 + 1))
                .unwrap_or_else(|x| x);
            table.resize(end / 2 - start / 2, false);
            for &p in self.primes.iter().skip(1).take(plen) {
                let y = p.max((start as u64 + p - 1) / (2 * p) * 2 + 1) * p / 2;
                (y as usize - start / 2..end / 2 - start / 2)
                    .step_by(p as usize)
                    .for_each(|i| table[i] = true);
            }
            for i in 0..=(sqrt_end / 2).saturating_sub(start / 2) {
                if !table[i] {
                    let p = (i + start / 2) * 2 + 1;
                    for j in (p * p / 2 - start / 2..sqrt_n / 2 - start / 2).step_by(p) {
                        table[j] = true;
                    }
                }
            }
            self.primes
                .extend(table.iter().cloned().enumerate().filter_map(|(i, b)| {
                    if !b {
                        Some((i + start / 2) as u64 * 2 + 1)
                    } else {
                        None
                    }
                }));
            self.max_n = sqrt_n as u64;
        }

        let sqrt_n = sqrt_n as u64;
        for start in (self.max_n + 1..=max_n).step_by(sqrt_n as usize) {
            let end = (start + sqrt_n).min(max_n + 1);
            let sqrt_end = (end as f64).sqrt() as u64;
            let length = end - start;
            let plen = self.primes[1..]
                .binary_search(&(sqrt_end + 1))
                .unwrap_or_else(|x| x);
            table.clear();
            table.resize(length as usize / 2, false);
            for &p in self.primes.iter().skip(1).take(plen) {
                let y = p.max((start + p - 1) / (2 * p) * 2 + 1) * p / 2;
                ((y - start / 2) as usize..length as usize / 2)
                    .step_by(p as usize)
                    .for_each(|i| table[i] = true);
            }
            self.primes
                .extend(table.iter().cloned().enumerate().filter_map(|(i, b)| {
                    if !b {
                        Some((i as u64 + start / 2) * 2 + 1)
                    } else {
                        None
                    }
                }));
        }
        self.max_n = max_n;
    }
}

#[derive(Debug, Clone)]
pub struct PrimeListTrialDivision<'p> {
    primes: Iter<'p, u64>,
    n: u64,
}
impl Iterator for PrimeListTrialDivision<'_> {
    type Item = (u64, u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.n <= 1 {
            return None;
        }
        loop {
            match self.primes.next() {
                Some(&p) if p * p <= self.n => {
                    if self.n % p == 0 {
                        let mut cnt = 1u32;
                        self.n /= p;
                        while self.n % p == 0 {
                            cnt += 1;
                            self.n /= p;
                        }
                        return Some((p, cnt));
                    }
                }
                _ => break,
            }
        }
        if self.n > 1 {
            return Some((replace(&mut self.n, 1), 1));
        }
        None
    }
}

pub fn with_prime_list<F>(max_n: u64, f: F)
where
    F: FnOnce(&PrimeList),
{
    thread_local!(static PRIME_LIST: UnsafeCell<PrimeList> = Default::default());
    PRIME_LIST.with(|cell| {
        unsafe {
            let pl = &mut *cell.get();
            pl.reserve(max_n);
            f(pl);
        };
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{prime_factors, PrimeTable};
    use crate::tools::Xorshift;

    fn primes(n: usize) -> Vec<usize> {
        if n < 2 {
            return vec![];
        }
        let mut res = vec![2];
        let sqrt_n = (n as f32).sqrt() as usize | 1;
        let mut seive = vec![true; n / 2];
        for i in (3..=sqrt_n).step_by(2) {
            if seive[i / 2 - 1] {
                res.push(i);
                for j in (i * i..=n).step_by(i * 2) {
                    seive[j / 2 - 1] = false;
                }
            }
        }
        for i in (std::cmp::max(3, sqrt_n + 2)..=n).step_by(2) {
            if seive[i / 2 - 1] {
                res.push(i);
            }
        }
        res
    }

    fn segmented_sieve_primes(n: usize) -> Vec<usize> {
        if n < 2 {
            return Vec::new();
        }
        let seg_size = ((n as f32).sqrt() as usize + 2) >> 1;
        let mut primes = vec![2];
        let mut table = vec![true; seg_size];
        for i in 1..seg_size {
            if table[i] {
                let p = i * 2 + 1;
                primes.push(p);
                for j in (p * p / 2..seg_size).step_by(p) {
                    table[j] = false;
                }
            }
        }
        for s in (seg_size..=n / 2).step_by(seg_size) {
            let m = seg_size.min((n + 1) / 2 - s);
            table.clear();
            table.resize(m, true);
            let plen = primes[1..]
                .binary_search(&((((s + m) * 2 + 1) as f32).sqrt() as usize + 1))
                .unwrap_or_else(|x| x);
            for &p in primes[1..plen + 1].iter() {
                for k in (((s * 2 + p * 3) / (p * 2) * p * 2 - p) / 2 - s..m).step_by(p) {
                    table[k] = false;
                }
            }
            primes.extend((s..m + s).filter(|k| table[k - s]).map(|k| k * 2 + 1));
        }
        primes
    }

    pub fn divisors(n: u64) -> Vec<u64> {
        let mut res = vec![];
        for i in 1..(n as f32).sqrt() as u64 + 1 {
            if n % i == 0 {
                res.push(i);
                if i * i != n {
                    res.push(n / i);
                }
            }
        }
        res.sort_unstable();
        res
    }

    #[test]
    fn test_prime_list() {
        let mut rng = Xorshift::default();

        for n in (0..1000).chain(rng.gen_iter(0..=20000).take(100)) {
            let pl = PrimeList::new(n);
            let ps: Vec<_> = primes(n as _).into_iter().map(|p| p as u64).collect();
            assert_eq!(pl.primes(), ps.as_slice());
        }

        for _ in 0..100 {
            let b = rng.randf() * 0.0001;
            let mut pl = PrimeList::new(0);
            for n in 0..20000 {
                if rng.gen_bool(b) {
                    pl.reserve(n);
                    let ps: Vec<_> = primes(n as _).into_iter().map(|p| p as u64).collect();
                    assert_eq!(pl.primes(), ps.as_slice());
                }
            }
        }

        let pl = PrimeList::new(100_000);
        for n in (0..1000).chain(rng.gen_iter(0..=1_000_000_000).take(100)) {
            assert_eq!(prime_factors(n), pl.prime_factors(n));
        }
    }

    #[test]
    fn test_primes() {
        let t = PrimeTable::new(2000);
        for i in 0..2000 {
            assert_eq!(
                primes(i),
                (2..=i)
                    .filter(|&i| t.is_prime(i as u32))
                    .collect::<Vec<_>>(),
            );
        }
    }

    #[test]
    fn test_segmented_sieve_primes() {
        for i in 0..300 {
            assert_eq!(primes(i), segmented_sieve_primes(i));
        }
        assert_eq!(primes(1_000_000), segmented_sieve_primes(1_000_000));
    }

    #[test]
    fn test_divisors() {
        let mut rng = Xorshift::default();
        let pl = PrimeList::new(20000);
        for n in (1..1000).chain(rng.gen_iter(1..=20000000).take(100)) {
            assert_eq!(pl.divisors(n), divisors(n));
        }
    }
}
