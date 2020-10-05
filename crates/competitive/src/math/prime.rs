use super::gcd_binary;

#[snippet::entry("prime")]
#[derive(Clone, Debug)]
pub struct PrimeTable {
    table: Vec<usize>,
}
#[snippet::entry("prime")]
impl PrimeTable {
    pub fn new(max_n: usize) -> Self {
        let mut table = vec![1; max_n + 1];
        table[0] = 0;
        table[1] = 0;
        for i in 2..=(max_n as f32).sqrt() as usize {
            if table[i] == 1 {
                for j in (i * i..=max_n).step_by(i) {
                    if table[j] == 1 {
                        table[j] = i;
                    }
                }
            }
        }
        PrimeTable { table }
    }
    pub fn is_prime(&self, n: usize) -> bool {
        self.table[n] == 1
    }
    pub fn prime_factors(&self, mut n: usize) -> Vec<(usize, usize)> {
        let mut factors = vec![];
        while self.table[n] > 1 {
            let p = self.table[n];
            let mut cnt = 1;
            n /= p;
            while self.table[n] == p {
                n /= p;
                cnt += 1;
            }
            if n == p {
                cnt += 1;
                n /= p;
            }
            factors.push((p, cnt));
        }
        if n > 1 {
            factors.push((n, 1));
        }
        factors
    }
    pub fn count_divisors(&self, mut n: usize) -> usize {
        let mut divisor_cnt = 1;
        while self.table[n] > 1 {
            let p = self.table[n];
            let mut cnt = 1;
            n /= p;
            while self.table[n] == p {
                n /= p;
                cnt += 1;
            }
            if n == p {
                cnt += 1;
                n /= p;
            }
            divisor_cnt *= cnt + 1;
        }
        if n > 1 {
            divisor_cnt *= 2;
        }
        divisor_cnt
    }
}

#[test]
fn test_prime_table() {
    const N: usize = 100_000;
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
                .product::<usize>()
        );
        assert_eq!(
            primes
                .prime_factors(i)
                .into_iter()
                .map(|(_, c)| c + 1)
                .product::<usize>(),
            primes.count_divisors(i)
        );
    }
}

#[snippet::entry]
pub fn prime_factors(mut n: usize) -> Vec<(usize, usize)> {
    let mut factors = vec![];
    for i in 2..=(n as f32).sqrt() as usize {
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

    const N: usize = 100_000;
    let primes = PrimeTable::new(N);
    for i in 1..=N {
        assert_eq!(primes.prime_factors(i), prime_factors(i));
    }
}

#[snippet::entry]
pub fn divisors(n: usize) -> Vec<usize> {
    let mut res = vec![];
    for i in 1..(n as f32).sqrt() as usize + 1 {
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

#[snippet::entry]
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
            (2..=i).filter(|&i| t.is_prime(i)).collect::<Vec<_>>(),
        );
    }
}

#[snippet::entry("miller_rabin")]
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

#[snippet::entry("miller_rabin")]
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
    const N: usize = 1_000_000;
    let primes = PrimeTable::new(N);
    for i in 2..=N {
        assert_eq!(primes.is_prime(i), miller_rabin(i as u64), "{}", i);
    }
    assert!(miller_rabin(1_000_000_007));
    assert!(!miller_rabin(1_000_000_011));
}

#[snippet::entry("prime_factors_rho")]
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

#[snippet::entry("prime_factors_rho", include("miller_rabin", "gcd_binary"))]
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
    let mut rand = Xorshift::time();
    for _ in 0..Q {
        let x = rand.rand64();
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
