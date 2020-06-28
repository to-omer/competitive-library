use crate::tools::random::Xorshift;

#[cargo_snippet::snippet("prime")]
#[derive(Clone, Debug)]
pub struct PrimeTable {
    table: Vec<usize>,
}
#[cargo_snippet::snippet("prime")]
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
}

#[test]
fn test_prime_table() {
    const N: usize = 100000;
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
    }
}

#[cargo_snippet::snippet]
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

    const N: usize = 100000;
    let primes = PrimeTable::new(N);
    for i in 1..=N {
        assert_eq!(primes.prime_factors(i), prime_factors(i));
    }
}

#[cargo_snippet::snippet]
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
    res.sort();
    res
}

#[cargo_snippet::snippet]
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

#[cargo_snippet::snippet("miller_rabin")]
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

#[cargo_snippet::snippet("miller_rabin")]
#[cargo_snippet::snippet(include = "Xorshift")]
pub fn miller_rabin(p: u64, times: usize) -> bool {
    if p == 2 {
        return true;
    }
    if p == 1 || p & 1 == 0 {
        return false;
    }
    let mut rand = Xorshift::time();
    let mut d = p - 1;
    while d & 1 == 0 {
        d >>= 1;
    }
    for _ in 0..times {
        let a = rand.next();
        let mut t = d;
        let mut y = pow(a, t, p);
        while t != p - 1 && y != 1 && y != p - 1 {
            y = y * y % p;
            t <<= 1;
        }
        if y != p - 1 && t & 1 == 0 {
            return false;
        }
    }
    true
}

#[test]
fn test_miller_rabin() {
    assert!(miller_rabin(1000000007, 100));
    assert!(!miller_rabin(1000000011, 100));
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
        EulerPhiTable { table: table }
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
        for j in 0..m {
            if i & 1 << j != 0 {
                mu *= -1;
                d *= primes[j];
            }
        }
        res.insert(d, mu);
    }
    res
}
