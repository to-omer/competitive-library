use std::{cell::UnsafeCell, mem::replace, ops::Range};

const WHEEL_PRIMES: [u32; 4] = [2, 3, 5, 7];
const PERIOD: u32 = 210;
const COPRIME: usize = 48;
const SQRT_THRESHOLD: u32 = 1 << 16;

const fn coprime_to_wheel(x: u32) -> bool {
    !x.is_multiple_of(2) && !x.is_multiple_of(3) && !x.is_multiple_of(5) && !x.is_multiple_of(7)
}

const fn residues() -> [u8; COPRIME] {
    let mut result = [0; COPRIME];
    let mut i = 1;
    let mut j = 0;
    while i < PERIOD {
        if coprime_to_wheel(i) {
            result[j] = i as u8;
            j += 1;
        }
        i += 2;
    }
    result
}

const RESIDUES: [u8; COPRIME] = residues();

const fn states() -> [u8; PERIOD as usize] {
    let mut result = [0; PERIOD as usize];
    let mut i = 0;
    let mut j = 0;
    while i < PERIOD {
        result[i as usize] = j;
        if coprime_to_wheel(i) {
            j += 1;
        }
        i += 1;
    }
    result
}

const STATES: [u8; PERIOD as usize] = states();

const fn additions() -> [u8; PERIOD as usize] {
    let mut result = [0; PERIOD as usize];
    let mut i = 0;
    while i < PERIOD {
        let mut add = 1;
        while !coprime_to_wheel(i + add) {
            add += 1;
        }
        result[i as usize] = add as u8;
        i += 1;
    }
    result
}

const ADDITIONS: [u8; PERIOD as usize] = additions();

const fn gaps() -> [u8; COPRIME] {
    let mut result = [0; COPRIME];
    let mut i = 0;
    while i < COPRIME {
        result[i] = ADDITIONS[RESIDUES[i] as usize];
        i += 1;
    }
    result
}

const GAPS: [u8; COPRIME] = gaps();

const fn to_ordinal(x: u32) -> u32 {
    x / PERIOD * COPRIME as u32 + STATES[(x % PERIOD) as usize] as u32
}

const fn to_value(x: u32) -> u32 {
    x / COPRIME as u32 * PERIOD + RESIDUES[x as usize % COPRIME] as u32
}

const fn ordinal_to_value() -> [u16; 256] {
    let mut result = [0; 256];
    let mut i = 0;
    while i < result.len() {
        result[i] = to_value(i as u32) as u16;
        i += 1;
    }
    result
}

const ORDINAL_TO_VALUE: [u16; 256] = ordinal_to_value();

const fn sqrt_bits() -> [u64; SQRT_THRESHOLD as usize / 128] {
    let mut result = [!0; SQRT_THRESHOLD as usize / 128];
    let ordinal = to_ordinal(1) as usize;
    result[ordinal / 64] &= !(1 << (ordinal % 64));
    let mut i = RESIDUES[1] as u32;
    while to_ordinal(i * i) < SQRT_THRESHOLD / 2 {
        let ordinal = to_ordinal(i) as usize;
        if result[ordinal / 64] >> (ordinal % 64) & 1 != 0 {
            let mut k = i;
            while to_ordinal(i * k) < SQRT_THRESHOLD / 2 {
                let ordinal = to_ordinal(i * k) as usize;
                result[ordinal / 64] &= !(1 << (ordinal % 64));
                k += ADDITIONS[(k % PERIOD) as usize] as u32;
            }
        }
        i += ADDITIONS[(i % PERIOD) as usize] as u32;
    }
    result
}

const SQRT_BITS: [u64; SQRT_THRESHOLD as usize / 128] = sqrt_bits();

const fn count_sqrt_primes() -> usize {
    let mut result = 0;
    let mut i = RESIDUES[1] as u32;
    while i < SQRT_THRESHOLD {
        let ordinal = to_ordinal(i) as usize;
        result += (SQRT_BITS[ordinal / 64] >> (ordinal % 64) & 1) as usize;
        i += ADDITIONS[(i % PERIOD) as usize] as u32;
    }
    result
}

const SQRT_PRIME_COUNT: usize = count_sqrt_primes();

const fn sqrt_primes() -> [u32; SQRT_PRIME_COUNT] {
    let mut result = [0; SQRT_PRIME_COUNT];
    let mut i = RESIDUES[1] as u32;
    let mut j = 0;
    while i < SQRT_THRESHOLD {
        let ordinal = to_ordinal(i) as usize;
        if SQRT_BITS[ordinal / 64] >> (ordinal % 64) & 1 != 0 {
            result[j] = i;
            j += 1;
        }
        i += ADDITIONS[(i % PERIOD) as usize] as u32;
    }
    result
}

static SQRT_PRIMES: [u32; SQRT_PRIME_COUNT] = sqrt_primes();

struct Wheel {
    mask: Vec<u64>,
    product: u32,
}

impl Wheel {
    fn new(primes: &[u32], product: u32) -> Self {
        let mut mask = vec![!0; to_ordinal(product) as usize / 64];
        for &p in primes {
            let mut k = 1;
            while p * k < product {
                let ordinal = to_ordinal(p * k) as usize;
                mask[ordinal / 64] &= !(1 << (ordinal % 64));
                k += ADDITIONS[(k % PERIOD) as usize] as u32;
            }
        }
        Self { mask, product }
    }
}

fn make_wheels() -> (Vec<Wheel>, usize) {
    const MAX_WHEEL_SIZE: u32 = 1 << 20;
    const BASE: u32 = (PERIOD * 64) >> (WHEEL_PRIMES.len() - 2);
    let mut product = BASE;
    let mut current = vec![];
    let mut wheels = vec![];
    for (i, &p) in SQRT_PRIMES.iter().enumerate() {
        if product * p > MAX_WHEEL_SIZE {
            wheels.push(Wheel::new(&current, product));
            current.clear();
            current.push(p);
            product = BASE * p;
            if product > MAX_WHEEL_SIZE {
                return (wheels, i);
            }
        } else {
            current.push(p);
            product *= p;
        }
    }
    unreachable!()
}

fn sieve_dense(bits: &mut [u64], l: u32, r: u32, wheel: &Wheel) {
    let mut left = l as usize / 64;
    let right = (r as usize).div_ceil(64);
    while left + wheel.mask.len() <= right {
        for (value, &mask) in bits[left..left + wheel.mask.len()]
            .iter_mut()
            .zip(&wheel.mask)
        {
            *value &= mask;
        }
        left += wheel.mask.len();
    }
    for (value, &mask) in bits[left..right].iter_mut().zip(&wheel.mask) {
        *value &= mask;
    }
}

fn ordinal_steps() -> Vec<[u32; COPRIME * 2]> {
    SQRT_PRIMES
        .iter()
        .map(|&p| {
            let mut result = [0; COPRIME * 2];
            let mut last = to_ordinal(p);
            for i in 0..COPRIME {
                let next = to_ordinal(p * (RESIDUES[i] as u32 + GAPS[i] as u32));
                result[i] = next - last;
                result[i + COPRIME] = next - last;
                last = next;
            }
            result
        })
        .collect()
}

fn sieve_sparse(
    bits: &mut [u64],
    mut left: u32,
    right: u32,
    prime_index: usize,
    mut state: u8,
    steps: &[[u32; COPRIME * 2]],
) -> (u32, u8) {
    let p = SQRT_PRIMES[prime_index];
    while left + p * COPRIME as u32 <= right {
        for _ in 0..COPRIME {
            let ordinal = left as usize;
            bits[ordinal / 64] &= !(1 << (ordinal % 64));
            left += steps[prime_index][state as usize];
            state += 1;
        }
        state -= COPRIME as u8;
    }
    while left < right {
        let ordinal = left as usize;
        bits[ordinal / 64] &= !(1 << (ordinal % 64));
        left += steps[prime_index][state as usize];
        state += 1;
    }
    if state >= COPRIME as u8 {
        state -= COPRIME as u8;
    }
    (left, state)
}

#[derive(Debug, Clone)]
pub struct PrimeList {
    bits: Vec<u64>,
    bit_len: usize,
    max_n: u32,
    prime_count: usize,
}

impl Default for PrimeList {
    fn default() -> Self {
        Self {
            bits: vec![],
            bit_len: 0,
            max_n: 1,
            prime_count: 0,
        }
    }
}

impl PrimeList {
    pub fn new(max_n: u32) -> Self {
        let mut self_: Self = Default::default();
        self_.reserve(max_n);
        self_
    }
    pub fn primes(&self) -> PrimeListIter<'_> {
        self.primes_lte(self.max_n)
    }
    pub fn len(&self) -> usize {
        self.prime_count
    }
    pub fn is_empty(&self) -> bool {
        self.prime_count == 0
    }
    pub fn primes_lte(&self, n: u32) -> PrimeListIter<'_> {
        assert!(n <= self.max_n, "expected `n={} <= {}`", n, self.max_n);
        let bit_len = to_ordinal(n.saturating_add(1)) as usize;
        let words = &self.bits[..self.bits.len().min(bit_len.div_ceil(64))];
        let last_mask = if bit_len.is_multiple_of(64) {
            !0
        } else {
            (1 << (bit_len % 64)) - 1
        };
        let (front_word, middle_words, back_word) = match words {
            [] => (0, words, 0),
            [word] => (*word & last_mask, &words[1..], 0),
            [front, middle @ .., back] => (*front, middle, *back & last_mask),
        };
        let back_word_ordinal = words.len().saturating_sub(1) as u32 * 64;
        PrimeListIter {
            wheel_indices: 0..WHEEL_PRIMES.partition_point(|&p| p <= n) as u8,
            middle_words,
            front_word_base: 0,
            front_word_phase: 0,
            front_word,
            back_word_base: back_word_ordinal / COPRIME as u32 * PERIOD,
            back_word_phase: (back_word_ordinal % COPRIME as u32) as u8,
            back_word,
        }
    }
    pub fn is_prime(&self, n: u32) -> bool {
        assert!(n <= self.max_n, "expected `n={} <= {}`", n, self.max_n);
        if WHEEL_PRIMES.contains(&n) {
            true
        } else if !coprime_to_wheel(n) {
            false
        } else {
            let ordinal = to_ordinal(n) as usize;
            ordinal < self.bit_len && self.bits[ordinal / 64] >> (ordinal % 64) & 1 != 0
        }
    }
    pub fn trial_division(&self, n: u64) -> PrimeListTrialDivision<'_> {
        let bound = u64::from(self.max_n).pow(2);
        assert!(n <= bound, "expected `n={} <= {}`", n, bound);
        PrimeListTrialDivision {
            primes: self.primes(),
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
            let mut acc = 1;
            for _ in 0..c {
                acc *= p;
                for i in 0..k {
                    d.push(d[i] * acc);
                }
            }
        }
        d.sort_unstable();
        d
    }
    /// Extends the prime list up to `max_n`.
    pub fn reserve(&mut self, max_n: u32) {
        if max_n <= self.max_n || max_n < 2 {
            return;
        }
        let limit = max_n.saturating_add(1);
        self.bit_len = to_ordinal(limit) as usize;
        if limit <= SQRT_THRESHOLD {
            self.bits = SQRT_BITS[..self.bit_len.div_ceil(64)].to_vec();
        } else {
            self.bits = vec![!0; self.bit_len.div_ceil(64)];
            let (wheels, medium_primes_begin) = make_wheels();
            const DENSE_BLOCK: u32 = 1 << 25;
            for start in (0..limit).step_by(DENSE_BLOCK as usize) {
                let right = start.saturating_add(DENSE_BLOCK).min(limit);
                for wheel in &wheels {
                    let left = start / wheel.product * wheel.product;
                    sieve_dense(&mut self.bits, to_ordinal(left), to_ordinal(right), wheel);
                }
            }

            let steps = ordinal_steps();
            let mut positions: Vec<_> = SQRT_PRIMES.iter().map(|&p| to_ordinal(p * p)).collect();
            let mut states: Vec<_> = SQRT_PRIMES
                .iter()
                .map(|&p| STATES[(p % PERIOD) as usize])
                .collect();
            const SPARSE_BLOCK: u32 = 1 << 22;
            for start in (0..limit).step_by(SPARSE_BLOCK as usize) {
                let right = to_ordinal(start.saturating_add(SPARSE_BLOCK).min(limit));
                for i in medium_primes_begin..SQRT_PRIME_COUNT {
                    (positions[i], states[i]) =
                        sieve_sparse(&mut self.bits, positions[i], right, i, states[i], &steps);
                }
            }
            for (value, &sqrt_bits) in self.bits.iter_mut().zip(&SQRT_BITS) {
                *value = sqrt_bits;
            }
        }

        self.prime_count = WHEEL_PRIMES.partition_point(|&p| p <= max_n);
        if let Some((&last, rest)) = self.bits.split_last() {
            self.prime_count += rest
                .iter()
                .map(|word| word.count_ones() as usize)
                .sum::<usize>();
            let last_mask = if self.bit_len.is_multiple_of(64) {
                !0
            } else {
                (1 << (self.bit_len % 64)) - 1
            };
            self.prime_count += (last & last_mask).count_ones() as usize;
        }
        self.max_n = max_n;
    }
}

#[derive(Clone, Debug)]
pub struct PrimeListIter<'a> {
    wheel_indices: Range<u8>,
    middle_words: &'a [u64],
    front_word_base: u32,
    front_word_phase: u8,
    front_word: u64,
    back_word_base: u32,
    back_word_phase: u8,
    back_word: u64,
}

impl PrimeListIter<'_> {
    #[inline(always)]
    fn load_front_word(&mut self) -> bool {
        if let Some((&word, words)) = self.middle_words.split_first() {
            self.middle_words = words;
            if self.front_word_phase == 32 {
                self.front_word_base += PERIOD * 2;
                self.front_word_phase = 0;
            } else {
                self.front_word_base += PERIOD;
                self.front_word_phase += 16;
            }
            self.front_word = word;
            true
        } else if self.back_word != 0 {
            self.front_word_base = self.back_word_base;
            self.front_word_phase = self.back_word_phase;
            self.front_word = replace(&mut self.back_word, 0);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    fn load_back_word(&mut self) -> bool {
        if let Some((&word, words)) = self.middle_words.split_last() {
            self.middle_words = words;
            if self.back_word_phase == 0 {
                self.back_word_base -= PERIOD * 2;
                self.back_word_phase = 32;
            } else {
                self.back_word_base -= PERIOD;
                self.back_word_phase -= 16;
            }
            self.back_word = word;
            true
        } else if self.front_word != 0 {
            self.back_word_base = self.front_word_base;
            self.back_word_phase = self.front_word_phase;
            self.back_word = replace(&mut self.front_word, 0);
            true
        } else {
            false
        }
    }
}

impl Iterator for PrimeListIter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.wheel_indices.next() {
            return Some(WHEEL_PRIMES[index as usize]);
        }
        loop {
            if self.front_word != 0 {
                let bit = self.front_word.trailing_zeros();
                self.front_word &= self.front_word - 1;
                return Some(
                    self.front_word_base
                        + ORDINAL_TO_VALUE[(self.front_word_phase + bit as u8) as usize] as u32,
                );
            }
            if !self.load_front_word() {
                return None;
            }
        }
    }

    fn nth(&mut self, mut n: usize) -> Option<Self::Item> {
        if n == 0 {
            return self.next();
        }
        for index in self.wheel_indices.by_ref() {
            if n == 0 {
                return Some(WHEEL_PRIMES[index as usize]);
            }
            n -= 1;
        }
        loop {
            let count = self.front_word.count_ones() as usize;
            if n < count {
                for _ in 0..n {
                    self.front_word &= self.front_word - 1;
                }
                return self.next();
            }
            n -= count;
            self.front_word = 0;
            if !self.load_front_word() {
                return None;
            }
        }
    }
}

impl DoubleEndedIterator for PrimeListIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.back_word != 0 {
                let bit = 63 - self.back_word.leading_zeros();
                self.back_word -= 1 << bit;
                return Some(
                    self.back_word_base
                        + ORDINAL_TO_VALUE[(self.back_word_phase + bit as u8) as usize] as u32,
                );
            }
            if !self.load_back_word() {
                return self
                    .wheel_indices
                    .next_back()
                    .map(|index| WHEEL_PRIMES[index as usize]);
            }
        }
    }

    fn nth_back(&mut self, mut n: usize) -> Option<Self::Item> {
        if n == 0 {
            return self.next_back();
        }
        loop {
            let count = self.back_word.count_ones() as usize;
            if n < count {
                for _ in 0..n {
                    let bit = 63 - self.back_word.leading_zeros();
                    self.back_word -= 1 << bit;
                }
                return self.next_back();
            }
            n -= count;
            self.back_word = 0;
            if !self.load_back_word() {
                for index in self.wheel_indices.by_ref().rev() {
                    if n == 0 {
                        return Some(WHEEL_PRIMES[index as usize]);
                    }
                    n -= 1;
                }
                return None;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrimeListTrialDivision<'p> {
    primes: PrimeListIter<'p>,
    n: u64,
}
impl Iterator for PrimeListTrialDivision<'_> {
    type Item = (u64, u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.n <= 1 {
            return None;
        }
        for p in self.primes.by_ref() {
            let p = u64::from(p);
            if p * p > self.n {
                break;
            }
            if self.n.is_multiple_of(p) {
                let mut cnt = 1u32;
                self.n /= p;
                while self.n.is_multiple_of(p) {
                    cnt += 1;
                    self.n /= p;
                }
                return Some((p, cnt));
            }
        }
        if self.n > 1 {
            return Some((replace(&mut self.n, 1), 1));
        }
        None
    }
}

pub fn with_prime_list<F>(max_n: u32, f: F)
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
    use crate::math::prime_factors;
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

    pub fn divisors(n: u64) -> Vec<u64> {
        let mut res = vec![];
        for i in 1..(n as f32).sqrt() as u64 + 1 {
            if n.is_multiple_of(i) {
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

        for n in (0..1000).chain(rng.random_iter(0..=20000).take(100)) {
            let pl = PrimeList::new(n);
            let ps: Vec<_> = primes(n as _).into_iter().map(|p| p as u32).collect();
            assert_eq!(pl.len(), ps.len());
            assert_eq!(pl.primes().collect::<Vec<_>>(), ps);
        }

        for _ in 0..100 {
            let b = rng.randf() * 0.0001;
            let mut pl = PrimeList::new(0);
            for n in (0..20_000).filter(|_| rng.gen_bool(b)) {
                pl.reserve(n);
                let ps: Vec<_> = primes(n as _).into_iter().map(|p| p as u32).collect();
                assert_eq!(pl.len(), ps.len());
                assert_eq!(pl.primes().collect::<Vec<_>>(), ps);
            }
        }

        let pl = PrimeList::new(100_000);
        for n in (0..1000).chain(rng.random_iter(0..=1_000_000_000).take(100)) {
            assert_eq!(prime_factors(n), pl.prime_factors(n));
        }
    }

    #[test]
    fn test_primes() {
        let pl = PrimeList::new(2000);
        for i in 0..=2000 {
            assert_eq!(
                primes(i),
                (2..=i).filter(|&i| pl.is_prime(i as _)).collect::<Vec<_>>(),
            );
            assert_eq!(
                primes(i).iter().map(|&p| p as u32).collect::<Vec<_>>(),
                pl.primes_lte(i as _).collect::<Vec<_>>()
            );
            assert_eq!(
                primes(i)
                    .iter()
                    .rev()
                    .map(|&p| p as u32)
                    .collect::<Vec<_>>(),
                pl.primes_lte(i as _).rev().collect::<Vec<_>>()
            );
        }
        let ps = primes(2000)
            .into_iter()
            .map(|p| p as u32)
            .collect::<Vec<_>>();
        for skip in (0..10).chain([ps.len(), ps.len() + 1]) {
            for step in 1..10 {
                assert_eq!(
                    ps.iter()
                        .copied()
                        .skip(skip)
                        .step_by(step)
                        .collect::<Vec<_>>(),
                    pl.primes().skip(skip).step_by(step).collect::<Vec<_>>()
                );
                assert_eq!(
                    ps.iter()
                        .rev()
                        .copied()
                        .skip(skip)
                        .step_by(step)
                        .collect::<Vec<_>>(),
                    pl.primes()
                        .rev()
                        .skip(skip)
                        .step_by(step)
                        .collect::<Vec<_>>()
                );
            }
        }
        let mut ps = std::collections::VecDeque::from(ps);
        let mut iter = pl.primes();
        while !ps.is_empty() {
            assert_eq!(iter.next(), ps.pop_front());
            assert_eq!(iter.next_back(), ps.pop_back());
        }
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_divisors() {
        let mut rng = Xorshift::default();
        let pl = PrimeList::new(20000);
        for n in (1..1000).chain(rng.random_iter(1..=20000000).take(100)) {
            assert_eq!(pl.divisors(n), divisors(n));
        }
    }
}
