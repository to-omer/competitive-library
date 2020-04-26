use crate::tools::random::Xorshift;
use cargo_snippet::snippet;

#[snippet("prime")]
#[derive(Clone, Debug)]
pub struct PrimeTable {
    table: Vec<usize>,
}
#[snippet("prime")]
impl PrimeTable {
    pub fn new(max_n: usize) -> Self {
        let mut table = vec![1; max_n + 1];
        table[0] = 0;
        table[1] = 0;
        for i in 2..(max_n as f32).sqrt() as usize + 1 {
            if table[i] == 1 {
                for j in ((i * i)..(max_n + 1)).step_by(i) {
                    if table[j] == 1 {
                        table[j] = i;
                    }
                }
            }
        }
        PrimeTable { table: table }
    }
    pub fn is_prime(&self, n: usize) -> bool {
        self.table[n] == 1
    }
    pub fn prime_factors(&self, n: usize) -> std::collections::HashMap<usize, usize> {
        let mut factors = std::collections::HashMap::new();
        let mut i = n;
        while self.table[i] != 1 {
            *factors.entry(self.table[i]).or_insert(0) += 1;
            i /= self.table[i];
        }
        *factors.entry(i).or_insert(0) += 1;
        factors
    }
}

#[test]
fn test_primes() {
    let primes = PrimeTable::new(100000);
    assert!(!primes.is_prime(100000));
    assert!(primes.is_prime(99991));

    let factors = primes.prime_factors(99991);
    assert_eq!(factors.len(), 1);
    assert_eq!(factors[&99991], 1);
    let factors = primes.prime_factors(2016);
    assert_eq!(factors.keys().collect::<Vec<_>>().len(), 3);
    assert_eq!(factors[&2], 5);
    assert_eq!(factors[&3], 2);
    assert_eq!(factors[&7], 1);

    let factors = prime_factors(99991);
    assert_eq!(factors.len(), 1);
    assert_eq!(factors[&99991], 1);
    let factors = prime_factors(2016);
    assert_eq!(factors.keys().collect::<Vec<_>>().len(), 3);
    assert_eq!(factors[&2], 5);
    assert_eq!(factors[&3], 2);
    assert_eq!(factors[&7], 1);
}

#[snippet("prime_factors")]
pub fn prime_factors(n: usize) -> std::collections::HashMap<usize, usize> {
    let mut factors = std::collections::HashMap::new();
    let mut n = n;
    for i in 2..(n as f32).sqrt() as usize + 1 {
        while n % i == 0 {
            n /= i;
            *factors.entry(i).or_insert(0) += 1;
        }
    }
    if n > 1 {
        *factors.entry(n).or_insert(0) += 1;
    }
    factors
}
#[snippet("divisors")]
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

#[snippet("miller_rabin")]
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

#[snippet("miller_rabin")]
#[snippet(include = "Xorshift")]
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
