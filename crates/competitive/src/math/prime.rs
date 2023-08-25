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
    let mut table = [false; 110];
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
