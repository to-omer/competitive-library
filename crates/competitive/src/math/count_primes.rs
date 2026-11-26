use super::PrimeList;

const RESIDUES: [u8; 8] = [1, 7, 11, 13, 17, 19, 23, 29];
const SMALL_PRIMES: [u32; 6] = [2, 3, 5, 7, 11, 13];
const PATTERN: [u8; 7 * 11 * 13] = {
    let mut pattern = [0; 7 * 11 * 13];
    let mut i = 0usize;
    while i < pattern.len() {
        let mut j = 0;
        while j < 8 {
            let v = 30 * i + RESIDUES[j] as usize;
            if !v.is_multiple_of(7) && !v.is_multiple_of(11) && !v.is_multiple_of(13) {
                pattern[i] |= 1 << j;
            }
            j += 1;
        }
        i += 1;
    }
    pattern
};
const MASK: [u64; 240] = {
    let mut masks = [0; 240];
    let mut mask = 0;
    let mut i = 0;
    let mut bit = 0;
    while i < 240 {
        if i == bit / 8 * 30 + RESIDUES[bit % 8] as usize {
            mask |= 1 << bit;
            bit += 1;
        }
        masks[i] = mask;
        i += 1;
    }
    masks
};
const WHEEL_COUNTS: [u16; PATTERN.len() + 1] = {
    let mut counts = [0; PATTERN.len() + 1];
    let mut i = 0;
    while i < PATTERN.len() {
        counts[i + 1] = counts[i] + PATTERN[i].count_ones() as u16;
        i += 1;
    }
    counts
};

/// Counts the primes less than or equal to `n` using the Meissel formula
/// and O(sqrt(n)) space.
pub fn count_primes(n: u64) -> u64 {
    if n < 1 << 16 {
        return PrimeList::new(n as u32).len() as u64;
    }
    let root = n.isqrt() as u32;
    let y = (4 * (n as f64).cbrt() as u32).min(root);
    let table = PrimeCounts::new(root);
    let primes = &table.primes[..table.count(y)];
    let factors = signed_factors(y, primes);
    let split = table.count(root.isqrt());

    // Meissel's formula: pi(n) = pi(y) - 1 + phi(n, pi(y)) - P2(n, pi(y)).
    let mut sum = primes.len() as i128 - 1 + i128::from(tiny_phi(n));
    for (i, &factor) in factors.iter().enumerate().skip(1) {
        if factor != 0 {
            sum += i128::from(factor.signum()) * i128::from(tiny_phi(n / (2 * i + 1) as u64));
        }
    }
    sum += easy_leaves(n, &table, primes.len(), split);
    sum += sieved_leaves_minus_semiprimes(n, y, &table, &factors, split);
    sum.try_into().unwrap()
}

fn tiny_phi(n: u64) -> u64 {
    let byte = n / 30;
    let i = (byte % PATTERN.len() as u64) as usize;
    let cycles = byte / PATTERN.len() as u64;
    cycles * u64::from(WHEEL_COUNTS[PATTERN.len()])
        + u64::from(WHEEL_COUNTS[i])
        + u64::from((PATTERN[i] & MASK[(n % 30) as usize] as u8).count_ones())
}

// At odd m > 1, store mu(m) * least_prime_factor(m) if m is coprime to the wheel.
fn signed_factors(y: u32, primes: &[u32]) -> Vec<i32> {
    let mut factors = vec![0i32; (y as usize).div_ceil(2)];
    for &p in primes[SMALL_PRIMES.len()..].iter().rev() {
        let end = ((y / p) as usize).div_ceil(2);
        for i in ((p as usize).div_ceil(2)..end).rev() {
            let f = factors[i];
            if f != 0 {
                factors[(2 * i + 1) * p as usize / 2] = -f.signum() * p as i32;
            }
        }
        factors[(p / 2) as usize] = -(p as i32);
    }
    factors
}

fn easy_leaves(n: u64, table: &PrimeCounts, a: usize, split: usize) -> i128 {
    let remaining = (a - split) as i128;
    let mut sum = remaining * (remaining - 1) / 2;
    for (i, &p) in table.primes[..a].iter().enumerate().skip(split) {
        let t = n / u64::from(p);
        let mid = table.count(t.isqrt() as u32);
        if mid <= i {
            break;
        }
        let end = mid.min(a);
        let count = table.count((t / u64::from(p)) as u32);
        sum += (count.min(a) - end) as i128;
        for &q in &table.primes[i + 1..end] {
            let count = table.count((t / u64::from(q)) as u32);
            sum += (count - i + count.min(a) - end) as i128;
        }
    }
    sum
}

fn sieved_leaves_minus_semiprimes(
    n: u64,
    y: u32,
    table: &PrimeCounts,
    factors: &[i32],
    split: usize,
) -> i128 {
    let z = n / u64::from(y);
    // Every full segment starts at the same phase of the presieve.
    const PAGE_SIZE: u64 = 240 * 4 * PATTERN.len() as u64;
    let mut partial_counts = vec![0u64; split];
    let mut prime_count = (SMALL_PRIMES.len() - 1) as u64;
    let a = table.count(y);
    let mut upper = table.primes.len();
    let mut sum = 0;
    for low in (0..=z).step_by(PAGE_SIZE as usize) {
        let high = z.min(low + PAGE_SIZE - 1);
        let mut segment = Segment::new(low, high);
        let sieve_end = table.count(high.isqrt() as u32);
        let partial_end = split.min(table.count((n / low.max(1)).isqrt() as u32));
        for (i, base) in partial_counts[..partial_end]
            .iter_mut()
            .enumerate()
            .skip(SMALL_PRIMES.len())
        {
            let prime = table.primes[i];
            let p = u64::from(prime);
            let t = n / p;
            let count = segment.build_prefix(high.min(t / p));
            // Sieving from p^2 retains the earlier primes as well as 1.
            let offset = *base as i64 + SMALL_PRIMES.len() as i64 - i as i64;
            sum += table.special_leaves(&segment, t, prime, y, factors, offset);
            *base += u64::from(count);
            segment.cross_off(p);
        }
        for &p in table.primes[..sieve_end]
            .iter()
            .skip(partial_end.max(SMALL_PRIMES.len()))
        {
            segment.cross_off(u64::from(p));
        }
        let count = segment.build_prefix(high);
        while upper > a {
            let x = n / u64::from(table.primes[upper - 1]);
            if x > high {
                break;
            }
            upper -= 1;
            let pi = prime_count + u64::from(segment.count(x));
            sum -= i128::from(pi) - upper as i128;
        }
        prime_count += u64::from(count);
    }
    sum
}

struct PrimeCounts {
    primes: Vec<u32>,
    sieve: Segment,
}

impl PrimeCounts {
    fn new(limit: u32) -> Self {
        let mut sieve = Segment::new(0, u64::from(limit));
        for p in PrimeList::new(limit.isqrt())
            .primes()
            .skip(SMALL_PRIMES.len())
        {
            sieve.cross_off(u64::from(p));
        }
        sieve.words[0][0] &= !1;
        let last = sieve.words.last_mut().unwrap();
        *last = (u64::from_le_bytes(*last) & MASK[limit as usize % 240]).to_le_bytes();
        sieve.build_prefix(u64::from(limit));

        let mut primes = SMALL_PRIMES.to_vec();
        for (i, &word) in sieve.words.iter().enumerate() {
            let mut bits = u64::from_le_bytes(word);
            while bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                let prime = 240 * i + 30 * (bit / 8) + usize::from(RESIDUES[bit % 8]);
                primes.push(prime as u32);
                bits &= bits - 1;
            }
        }
        Self { primes, sieve }
    }

    fn count(&self, n: u32) -> usize {
        let small = if n >= 13 {
            SMALL_PRIMES.len()
        } else {
            SMALL_PRIMES.partition_point(|&p| p <= n)
        };
        small + self.sieve.count(u64::from(n)) as usize
    }

    fn special_leaves(
        &self,
        segment: &Segment,
        quotient: u64,
        prime: u32,
        y: u32,
        factors: &[i32],
        offset: i64,
    ) -> i128 {
        let min_m = (quotient / (segment.high + 1)).min(u64::from(y)) as u32;
        let max_m = (quotient / segment.low.max(1)).min(u64::from(y)) as u32;
        if u64::from(prime).pow(2) <= u64::from(y) {
            let mut sum = 0;
            let start = (min_m.max(y / prime) as usize).div_ceil(2);
            let end = (max_m as usize).div_ceil(2);
            for (i, &f) in factors[start..end].iter().enumerate() {
                if f.unsigned_abs() > prime {
                    let m = 2 * (start + i) + 1;
                    let x = quotient / m as u64;
                    let phi = offset + i64::from(segment.count(x));
                    sum -= i128::from(f.signum()) * i128::from(phi);
                }
            }
            sum
        } else {
            let start = self.count(min_m.max(prime));
            let end = self.count(max_m);
            let mut count = 0u64;
            for &q in &self.primes[start..end] {
                let x = quotient / u64::from(q);
                count += u64::from(segment.count(x));
            }
            i128::from(count) + i128::from(offset) * (end - start) as i128
        }
    }
}

struct Segment {
    low: u64,
    high: u64,
    words: Vec<[u8; 8]>,
    prefix: Vec<u32>,
}

impl Segment {
    fn new(low: u64, high: u64) -> Self {
        let len = ((high - low) / 240 + 1) as usize;
        let mut words = vec![[0; 8]; len];
        for chunk in words.as_flattened_mut().chunks_mut(PATTERN.len()) {
            chunk.copy_from_slice(&PATTERN[..chunk.len()]);
        }
        Self {
            low,
            high,
            words,
            prefix: vec![0; len],
        }
    }

    fn cross_off(&mut self, p: u64) {
        let low = self.low;
        let first = low.div_ceil(p).max(p);
        let cycle = first / 30 * 30;
        let bytes = self.words.as_flattened_mut();
        for &r in &RESIDUES {
            let mut m = cycle + u64::from(r);
            if m < first {
                m += 30;
            }
            let v = p * m;
            let residue = (v % 30) as usize;
            let keep = !(MASK[residue] ^ MASK[residue - 1]) as u8;
            for i in (((v - low) / 30) as usize..bytes.len()).step_by(p as usize) {
                bytes[i] &= keep;
            }
        }
    }

    fn build_prefix(&mut self, high: u64) -> u32 {
        let last = (high - self.low) as usize;
        let mut count = 0;
        for (&word, prefix) in self.words[..=last / 240].iter().zip(&mut self.prefix) {
            *prefix = count;
            count += u64::from_le_bytes(word).count_ones();
        }
        count
    }

    fn count(&self, n: u64) -> u32 {
        let x = (n - self.low) as usize;
        let word = x / 240;
        self.prefix[word] + (u64::from_le_bytes(self.words[word]) & MASK[x % 240]).count_ones()
    }
}

#[cfg(test)]
mod tests {
    use super::count_primes;
    use crate::{algebra::AdditiveOperation, math::QuotientArray, tools::Xorshift};

    #[test]
    fn prime_counts() {
        let limit = (1 << 16) + 256;
        let mut prime = vec![true; limit + 1];
        prime[..2].fill(false);
        for p in 2..=limit.isqrt() {
            if prime[p] {
                for j in (p * p..=limit).step_by(p) {
                    prime[j] = false;
                }
            }
        }
        let mut count = 0;
        for (n, prime) in prime.into_iter().enumerate() {
            count += u64::from(prime);
            assert_eq!(count_primes(n as u64), count, "n = {n}");
        }

        let mut rng = Xorshift::default();
        let boundaries = [
            257u64.pow(2),
            43u64.pow(3),
            17u64.pow(4),
            73u64.pow(3),
            241u64.pow(3),
            101u64.pow(4),
        ];
        for n in boundaries
            .into_iter()
            .flat_map(|n| n - 1..=n + 1)
            .chain(rng.random_iter(1u64 << 16..=1 << 29).take(28))
        {
            let counts = QuotientArray::from_fn(n, |i| i as i64 - 1)
                .lucy_dp::<AdditiveOperation<_>>(|x, _| x);
            assert_eq!(count_primes(n), counts[n] as u64, "n = {n}");
        }

        // n/y first reaches a new segment at 7_533_926_400. The semiprime
        // 7841 * 960961 queries the first prime in that segment.
        for (n, expected) in [
            (7_533_926_399, 347_318_478),
            (7_533_926_400, 347_318_478),
            (7_533_926_401, 347_318_479),
            (7_534_895_201, 347_361_125),
            (1_000_000_000_000, 37_607_912_018),
        ] {
            assert_eq!(count_primes(n), expected, "n = {n}");
        }
    }
}
