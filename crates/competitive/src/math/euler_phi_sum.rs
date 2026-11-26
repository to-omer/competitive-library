use super::PrimeList;

const PERIOD: usize = 2 * 3 * 5 * 7 * 11 * 13;
const FIRST_PRIME: usize = 17;
static WHEEL: Wheel = Wheel::new();
const ODD_MOBIUS: [i8; PERIOD / 2] = {
    let mut values = [0; PERIOD / 2];
    let mut i = 0;
    while i < values.len() {
        values[i] = -((WHEEL.count[2 * i + 1] != WHEEL.count[2 * i]) as i8);
        i += 1;
    }
    values
};

/// Returns the exact sum of Euler's totient function on `1..=n`.
/// Uses $O(n^{2/3} \log\log n)$ time and $O(n^{2/3})$ space.
pub fn euler_phi_sum(n: u64) -> u128 {
    if n == 0 {
        return 0;
    }
    let root = n.isqrt() as usize;
    let limit = ((n as f64).cbrt().powi(2) * 0.5) as usize;
    let limit = limit.max(root);
    let mobius = sieved_mobius(limit);
    let mut small = Vec::with_capacity(root + 1);
    let mut sum = 0i32;
    for i in 0..=root {
        if i % 2 == 1 {
            sum += i32::from(mobius[i / 2]);
        }
        small.push((WHEEL.count(i as u64) as i32, sum));
    }
    let mut large = vec![(0i64, 0i64); root + 1];
    let mut pos = root.div_ceil(2);
    let mut sum = i64::from(sum);
    for (i, entry) in large.iter_mut().enumerate().skip(1).rev() {
        let x = n / i as u64;
        let end = (x.min(limit as u64) as usize).div_ceil(2);
        sum += mobius[pos..end].iter().map(|&v| i64::from(v)).sum::<i64>();
        pos = end;
        *entry = (WHEEL.count(x) as i64, sum);
    }

    let rough: Vec<_> = (2..=root)
        .filter(|&d| small[d].0 != small[d - 1].0)
        .map(|d| (d as u32, mobius[d / 2], u64::MAX / d as u64 + 1))
        .collect();
    // On integers coprime to the wheel, mu * 1 is the unit for Dirichlet convolution.
    for i in (1..=(n / (limit as u64 + 1)) as usize).rev() {
        let x = n / i as u64;
        let b = x.isqrt() as usize;
        let (count, mertens) = small[b];
        let mut sum = 1 - large[i].0 + i64::from(count) * i64::from(mertens);
        let split = small[root / i].0 as usize - 1;
        let end = count as usize - 1;
        for &(d, mu, _) in &rough[..split] {
            let (count, mertens) = large[i * d as usize];
            sum = sum.wrapping_sub(mertens + i64::from(mu) * count);
        }
        // ceil(2^64/d) gives the exact quotient when x*d < 2^64.
        let use_reciprocal = x <= u64::MAX / b as u64;
        for &(d, mu, reciprocal) in &rough[split..end] {
            let q = if use_reciprocal {
                ((u128::from(x) * u128::from(reciprocal)) >> 64) as usize
            } else {
                (x / u64::from(d)) as usize
            };
            let (count, mertens) = small[q];
            sum = sum.wrapping_sub(i64::from(mertens + i32::from(mu) * count));
        }
        large[i].1 = sum;
    }

    // phi = restricted mu * h, where h(e) = e * product_{p | gcd(e, PERIOD)} (1 - 1/p).
    let mut total = WHEEL.weight_sum(n) as i128;
    for (e, &(_, mertens)) in large.iter().enumerate().skip(1) {
        total = total.wrapping_add(i128::from(WHEEL.weight(e)) * i128::from(mertens));
    }
    for &(d, mu, _) in &rough {
        total = total.wrapping_add(i128::from(mu) * WHEEL.weight_sum(n / u64::from(d)) as i128);
    }
    total.wrapping_sub(WHEEL.weight_sum(root as u64) as i128 * i128::from(small[root].1)) as u128
}

fn sieved_mobius(limit: usize) -> Vec<i8> {
    let mut values = vec![0; limit.div_ceil(2)];
    for block in values.chunks_mut(ODD_MOBIUS.len()) {
        block.copy_from_slice(&ODD_MOBIUS[..block.len()]);
    }
    let cofactors: Vec<_> = (1..=limit / FIRST_PRIME)
        .step_by(2)
        .filter(|&k| ODD_MOBIUS[k % PERIOD / 2] != 0)
        .collect();
    let primes = PrimeList::new(limit.isqrt() as u32);
    for p in primes.primes().rev() {
        let p = p as usize;
        if p < FIRST_PRIME {
            break;
        }
        let start = WHEEL.count(p as u64) as usize - 1;
        let end = WHEEL.count((limit / p) as u64) as usize;
        for &k in &cofactors[start..end] {
            values[p * k / 2] = -values[k / 2];
        }
        for i in (p * p / 2..values.len()).step_by(p * p) {
            values[i] = 0;
        }
    }
    values[0] = 1;
    values
}

struct Wheel {
    count: [u16; PERIOD + 1],
    slope: [u32; PERIOD + 1],
    sum: [u32; PERIOD + 1],
}

impl Wheel {
    const fn new() -> Self {
        let mut weights = [PERIOD; PERIOD + 1];
        let primes = [2, 3, 5, 7, 11, 13];
        let mut j = 0;
        while j < primes.len() {
            let p = primes[j];
            let mut i = p;
            while i <= PERIOD {
                weights[i] = weights[i] / p * (p - 1);
                i += p;
            }
            j += 1;
        }
        let mut wheel = Self {
            count: [0; PERIOD + 1],
            slope: [0; PERIOD + 1],
            sum: [0; PERIOD + 1],
        };
        let mut i = 1;
        while i <= PERIOD {
            let w = weights[i];
            wheel.count[i] = wheel.count[i - 1] + (w == PERIOD) as u16;
            wheel.slope[i] = wheel.slope[i - 1] + w as u32;
            wheel.sum[i] = wheel.sum[i - 1] + (i * w / PERIOD) as u32;
            i += 1;
        }
        wheel
    }

    fn count(&self, x: u64) -> u64 {
        x / PERIOD as u64 * u64::from(self.count[PERIOD])
            + u64::from(self.count[(x % PERIOD as u64) as usize])
    }

    fn weight(&self, x: usize) -> u64 {
        let r = (x - 1) % PERIOD;
        x as u64 * u64::from(self.slope[r + 1] - self.slope[r]) / PERIOD as u64
    }

    fn weight_sum(&self, x: u64) -> u128 {
        let q = u128::from(x / PERIOD as u64);
        let r = (x % PERIOD as u64) as usize;
        q * q.saturating_sub(1) / 2 * u128::from(self.slope[PERIOD])
            + q * u128::from(self.sum[PERIOD] + self.slope[r])
            + u128::from(self.sum[r])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AddMulOperation, AdditiveOperation, ArrayOperation},
        math::{QuotientArray, euler_phi},
        tools::{Xorshift, testutil::sample_usize},
    };

    #[test]
    fn totient_prefix_sum() {
        let mut expected = 0;
        for n in 0..=4096 {
            if n > 0 {
                expected += euler_phi(n) as u128;
            }
            assert_eq!(euler_phi_sum(n as u64), expected, "n={n}");
        }

        let mut cases = sample_usize(&mut Xorshift::default(), 0, 1..=2_000_000, 40);
        for k in [17usize, 19, 31, 67, 101] {
            for boundary in [k * k, k.pow(3), k * (k + 1)] {
                cases.extend(boundary - 1..=boundary + 1);
            }
        }
        for threshold in [FIRST_PRIME.pow(2), PERIOD / 2, PERIOD] {
            let boundary = (2 * threshold as u64).pow(3).isqrt() as usize;
            cases.extend(boundary - 1..=boundary + 2);
        }
        cases.extend(PERIOD - 1..=PERIOD + 1);
        for n in cases {
            let n = n as u64;
            let phi = QuotientArray::from_fn(n, |x| {
                let x = i128::from(x);
                [x - 1, x * (x + 1) / 2 - 1]
            })
            .lucy_dp::<ArrayOperation<AdditiveOperation<_>, 2>>(|[c, s], p| [c, s * i128::from(p)])
            .map(|[c, s]| s - c)
            .min_25_sieve::<AddMulOperation<_>>(|p, e| {
                i128::from(p - 1) * i128::from(p).pow(e - 1)
            });
            assert_eq!(euler_phi_sum(n), phi[n] as u128, "n={n}");
        }
        assert_eq!(euler_phi_sum(10_000_000_000), 30_396_355_092_886_216_366);
    }
}
