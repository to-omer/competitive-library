use super::gcd_binary;

#[codesnip::entry("PrimeTable")]
#[derive(Clone, Debug)]
pub struct PrimeTable {
    table: Vec<u32>,
}
#[codesnip::entry("PrimeTable")]
impl PrimeTable {
    pub fn new(max_n: u32) -> Self {
        let mut table = vec![1; (max_n as usize + 1) / 2];
        table[0] = 0;
        for i in (3..).step_by(2) {
            let i2 = i * i;
            if i2 > max_n {
                break;
            }
            if table[i as usize >> 1] == 1 {
                for j in (i2..=max_n).step_by(i as usize * 2) {
                    if table[j as usize >> 1] == 1 {
                        table[j as usize >> 1] = i;
                    }
                }
            }
        }
        PrimeTable { table }
    }
    pub fn is_prime(&self, n: u32) -> bool {
        n == 2 || n % 2 == 1 && self.table[n as usize >> 1] == 1
    }
    pub fn trial_division<F>(&self, mut n: u32, mut f: F)
    where
        F: FnMut(u32, u32),
    {
        let k = n.trailing_zeros();
        if k > 0 {
            f(2, k);
        }
        n >>= k;
        while self.table[n as usize >> 1] > 1 {
            let p = self.table[n as usize >> 1];
            let mut cnt = 1;
            n /= p;
            while self.table[n as usize >> 1] == p {
                n /= p;
                cnt += 1;
            }
            if n == p {
                cnt += 1;
                n /= p;
            }
            f(p, cnt);
        }
        if n > 1 {
            f(n, 1);
        }
    }
    pub fn prime_factors(&self, n: u32) -> Vec<(u32, u32)> {
        let mut factors = vec![];
        self.trial_division(n, |p, c| factors.push((p, c)));
        factors
    }
    pub fn count_divisors(&self, n: u32) -> u32 {
        let mut divisor_cnt = 1;
        self.trial_division(n, |_, cnt| divisor_cnt *= cnt + 1);
        divisor_cnt
    }
}

#[test]
fn test_prime_table() {
    const N: u32 = 100_000;
    let primes = PrimeTable::new(N);
    assert!(!primes.is_prime(N));
    assert!(primes.is_prime(99991));

    let factors = primes.prime_factors(99991);
    assert_eq!(factors, vec![(99991, 1)]);
    let factors = primes.prime_factors(2016);
    assert_eq!(factors, vec![(2, 5), (3, 2), (7, 1)]);
    for i in 1..=N {
        assert_eq!(
            i,
            primes
                .prime_factors(i)
                .into_iter()
                .map(|(p, c)| p.pow(c as u32))
                .product::<u32>()
        );
        assert_eq!(
            primes
                .prime_factors(i)
                .into_iter()
                .map(|(_, c)| c + 1)
                .product::<u32>(),
            primes.count_divisors(i)
        );
    }
}

#[codesnip::entry("PrimeList")]
pub use prime_list::PrimeList;
#[codesnip::entry("PrimeList")]
pub mod prime_list {
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
        /// list primes less than or equal to `max_n` by segmented sieve
        fn reserve(&mut self, max_n: u64) {
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
                    let y = p.max((start as u64 + p - 1) / (2 * p) * 2 + 1) * p / 2;
                    ((y - start / 2) as usize..length as usize / 2)
                        .step_by(p as usize)
                        .for_each(|i| table[i] = true);
                }
                self.primes
                    .extend(table.iter().cloned().enumerate().filter_map(|(i, b)| {
                        if !b {
                            Some((i as u64 + start / 2) as u64 * 2 + 1)
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
        primes: std::slice::Iter<'p, u64>,
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
                return Some((std::mem::replace(&mut self.n, 1), 1));
            }
            None
        }
    }

    #[test]
    fn test_prime_list() {
        use super::{prime_factors, primes};
        use crate::tools::Xorshift;
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
            assert_eq!(prime_factors(n), pl.prime_factors(n as u64));
        }
    }
}

#[codesnip::entry]
pub fn prime_factors(mut n: u64) -> Vec<(u64, u32)> {
    let mut factors = vec![];
    let k = n.trailing_zeros();
    if n > 0 && k > 0 {
        n >>= k;
        factors.push((2, k));
    }
    for i in (3..=(n as f64).sqrt() as u64).step_by(2) {
        if n % i == 0 {
            let mut cnt = 1;
            n /= i;
            while n % i == 0 {
                cnt += 1;
                n /= i;
            }
            factors.push((i, cnt));
        }
    }
    if n > 1 {
        factors.push((n, 1));
    }
    factors
}

#[test]
fn test_prime_factors() {
    let factors = prime_factors(99991);
    assert_eq!(factors, vec![(99991, 1)]);
    let factors = prime_factors(2016);
    assert_eq!(factors, vec![(2, 5), (3, 2), (7, 1)]);

    const N: u32 = 100_000;
    let primes = PrimeTable::new(N);
    for i in 1..=N {
        assert_eq!(
            primes.prime_factors(i),
            prime_factors(i as _)
                .into_iter()
                .map(|(p, c)| (p as u32, c))
                .collect::<Vec<_>>()
        );
    }
}

#[codesnip::entry]
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

#[codesnip::entry]
pub fn primes(n: usize) -> Vec<usize> {
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

#[codesnip::entry("miller_rabin")]
pub fn pow(x: u64, y: u64, z: u64) -> u64 {
    let mut x = x as u128;
    let mut y = y as u128;
    let z = z as u128;
    let mut res: u128 = 1;
    while y > 0 {
        if y & 1 == 1 {
            res = res * x % z;
        }
        x = x * x % z;
        y >>= 1;
    }
    res as u64
}

#[codesnip::entry("miller_rabin")]
pub fn miller_rabin(p: u64) -> bool {
    if p == 2 {
        return true;
    }
    if p == 1 || p % 2 == 0 {
        return false;
    }
    let d = p - 1;
    let k = d.trailing_zeros();
    let d = d >> k;
    let a = if p < 4_759_123_141 {
        vec![2, 7, 61]
    } else {
        vec![2, 325, 9375, 28178, 450_775, 9_780_504, 1_795_265_022]
    };
    'outer: for &a in a.iter() {
        if a >= p {
            break;
        }
        let mut y = pow(a, d, p);
        if y == 1 || y == p - 1 {
            continue;
        }
        for _ in 0..k - 1 {
            y = (y as u128 * y as u128 % p as u128) as u64;
            if y == p - 1 {
                continue 'outer;
            }
        }
        return false;
    }
    true
}

#[test]
fn test_miller_rabin() {
    const N: u32 = 1_000_000;
    let primes = PrimeTable::new(N);
    for i in 2..=N {
        assert_eq!(primes.is_prime(i), miller_rabin(i as u64), "{}", i);
    }
    assert!(miller_rabin(1_000_000_007));
    assert!(!miller_rabin(1_000_000_011));
}

#[codesnip::entry("prime_factors_rho")]
pub fn find_factor(n: u64) -> u64 {
    const M: usize = 128;
    let sub = |x: u64, y: u64| if x > y { x - y } else { y - x };
    let mul = |x: u64, y: u64| (x as u128 * y as u128 % n as u128) as u64;
    for c in 12.. {
        let f = |x: u64| (x as u128 * x as u128 % n as u128 + c) as u64;
        let (mut x, mut y, mut r, mut g, mut k, mut ys) = (0, 2, 1, 1, 0, 0);
        while g == 1 {
            x = y;
            for _ in 0..r {
                y = f(y);
            }
            while r > k && g == 1 {
                ys = y;
                let mut q = 1;
                for _ in 0..M.min(r - k) {
                    y = f(y);
                    q = mul(q, sub(x, y));
                }
                g = gcd_binary(q, n);
                k += M;
            }
            r <<= 1;
        }
        if g == n {
            g = 1;
            while g == 1 {
                ys = f(ys);
                g = gcd_binary(sub(x, ys), n);
            }
        }
        if g < n {
            return g;
        }
    }
    unreachable!();
}

#[codesnip::entry("prime_factors_rho", include("miller_rabin", "gcd_binary"))]
pub fn prime_factors_rho(mut n: u64) -> Vec<u64> {
    let k = n.trailing_zeros();
    let mut res = vec![2; k as usize];
    n >>= k;
    if n != 1 {
        let mut c = vec![n];
        while let Some(n) = c.pop() {
            if miller_rabin(n) {
                res.push(n);
            } else {
                let m = find_factor(n);
                c.push(m);
                c.push(n / m);
            }
        }
    }
    res.sort_unstable();
    res
}

#[test]
fn test_prime_factors_rho() {
    use crate::tools::Xorshift;
    const Q: usize = 2_000;
    let mut rng = Xorshift::time();
    for _ in 0..Q {
        let x = rng.rand64();
        let factors = prime_factors_rho(x);
        assert!(factors.iter().all(|&p| miller_rabin(p)));
        let p = factors.into_iter().product::<u64>();
        assert_eq!(x, p);
    }
}

pub fn euler_phi(n: usize) -> usize {
    let mut n = n;
    let mut res = n;
    for i in 2..(n as f32).sqrt() as usize + 1 {
        if n % i == 0 {
            res = res / i * (i - 1);
            while n % i == 0 {
                n /= i;
            }
        }
    }
    if n != 1 {
        res = res / n * (n - 1);
    }
    res
}

#[derive(Clone, Debug)]
pub struct EulerPhiTable {
    table: Vec<usize>,
}
impl EulerPhiTable {
    pub fn new(max_n: usize) -> Self {
        let mut table = (0..max_n + 1).collect::<Vec<_>>();
        for i in 2..max_n + 1 {
            if table[i] == i {
                let mut j = i;
                while j <= max_n {
                    table[j] = table[j] / i * (i - 1);
                    j += i;
                }
            }
        }
        EulerPhiTable { table }
    }
    pub fn get(&self, n: usize) -> usize {
        self.table[n]
    }
}

#[codesnip::entry]
/// g(d) = Sigma mu(d) * f(n/d)
pub fn moebius(n: usize) -> std::collections::HashMap<usize, i64> {
    let mut res = std::collections::HashMap::new();
    let mut primes = vec![];
    let mut n = n;
    for i in 2..(n as f32).sqrt() as usize + 1 {
        if n % i == 0 {
            primes.push(i);
            while n % i == 0 {
                n /= i;
            }
        }
    }
    if n != 1 {
        primes.push(n);
    }
    let m = primes.len();
    for i in 0..1 << m {
        let (mut mu, mut d) = (1, 1);
        for (j, p) in primes.iter().enumerate() {
            if i & 1 << j != 0 {
                mu *= -1;
                d *= p;
            }
        }
        res.insert(d, mu);
    }
    res
}

#[codesnip::entry]
pub fn segmented_sieve_primes(n: usize) -> Vec<usize> {
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

#[test]
fn test_segmented_sieve_primes() {
    for i in 0..300 {
        assert_eq!(primes(i), segmented_sieve_primes(i));
    }
    assert_eq!(primes(1_000_000), segmented_sieve_primes(1_000_000));
}
