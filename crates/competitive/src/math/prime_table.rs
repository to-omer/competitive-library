#[derive(Clone, Debug)]
pub struct PrimeTable {
    table: Vec<u32>,
}

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
