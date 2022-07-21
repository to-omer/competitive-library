use super::gcd_binary;

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
    let primes = super::PrimeTable::new(N);
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
    let primes = super::PrimeTable::new(N);
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
/// [(hcn, #divisor)]
pub fn highly_composite_number(n: u128) -> Vec<(u128, u128)> {
    let mut dp = vec![(1u128, 1u128)];
    let mut acc = 1u128;
    let mut table = vec![false; 110];
    for p in 2u128.. {
        if !table[p as usize] {
            for i in (p..110).step_by(p as _) {
                table[i as usize] = true;
            }
            acc = acc.saturating_mul(p);
            if acc > n {
                break;
            }
            let m = dp.len();
            for i in 0..m {
                let (mut a, b) = dp[i];
                for j in 2.. {
                    a = a.saturating_mul(p);
                    let nb = b.saturating_mul(j);
                    if a > n {
                        break;
                    }
                    dp.push((a, nb));
                }
            }
            dp.sort_unstable();
            let mut ndp = vec![];
            let mut mx = 0u128;
            for (a, b) in dp {
                if b > mx {
                    mx = b;
                    ndp.push((a, b));
                }
            }
            dp = ndp;
        }
    }
    dp
}
