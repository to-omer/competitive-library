use crate::math::gcd::{gcd, modinv};

pub fn binary_exponentiation<T: Clone + std::ops::MulAssign>(
    mut x: T,
    mut y: usize,
    mut one: T,
) -> T {
    while y > 0 {
        if y & 1 == 1 {
            one *= x.clone();
        }
        x *= x.clone();
        y >>= 1;
    }
    one
}

#[codesnip::entry("BabyStepGiantStep", include("modinv"))]
#[derive(Clone, Debug)]
pub struct BabyStepGiantStep {
    p: u64,
    r: u64,
    baby: std::collections::HashMap<u64, u64>,
}
#[codesnip::entry("BabyStepGiantStep")]
impl BabyStepGiantStep {
    pub fn new(x: u64, p: u64) -> Self {
        let m = (p as f32).sqrt() as u64 + 1;
        let mut baby = std::collections::HashMap::new();
        let mut a = 1;
        for i in 0..m {
            baby.entry(a).or_insert(i);
            a = a * x % p;
        }
        BabyStepGiantStep {
            p,
            r: modinv(a as i64, p as i64) as u64,
            baby,
        }
    }
    // minimum i where x ** i = y mod p
    pub fn solve(&self, y: u64) -> Option<u64> {
        let m = self.baby.len() as u64;
        let mut y = y;
        for j in 0..m + 1 {
            if let Some(i) = self.baby.get(&y) {
                return Some(i + j * m);
            }
            y = y * self.r % self.p;
        }
        None
    }
}

/// Sum of Floor of Linear
///
/// $$\sum_{i=0}^{n-1}\left\lfloor\frac{a\times i+b}{m}\right\rfloor$$
#[codesnip::entry]
pub fn floor_sum(n: u64, m: u64, mut a: u64, mut b: u64) -> u64 {
    let mut ans = 0u64;
    if a >= m {
        ans += (n - 1) * n * (a / m) / 2;
        a %= m;
    }
    if b >= m {
        ans += n * (b / m);
        b %= m;
    }
    let y_max = (a * n + b) / m;
    if y_max == 0 {
        return ans;
    }
    let x_max = y_max * m - b;
    ans += (n - (x_max + a - 1) / a) * y_max;
    ans += floor_sum(y_max, a, m, (a - x_max % a) % a);
    ans
}

/// return: (y,z)
///
/// forall (a,b,m), ax = b mod m, where x = y mod z
#[codesnip::entry(include("gcd", "modinv"))]
pub fn linear_congruence(abm: impl IntoIterator<Item = (i64, i64, i64)>) -> Option<(i64, i64)> {
    let mut x = 0i64;
    let mut m0 = 1i64;
    for (a, b, m) in abm {
        let b = b - a * x;
        let a = a * m0;
        let g = gcd(a as u64, m as u64) as i64;
        if b % g != 0 {
            return None;
        }
        x += b / g * modinv(a / g, m / g) % (m / g) * m0;
        m0 *= m / g;
    }
    x %= m0;
    if x < 0 {
        x += m0;
    }
    Some((x, m0))
}

#[test]
fn test_linear_congruence() {
    use crate::math::lcm;
    use crate::tools::Xorshift;
    const N: usize = 5;
    const Q: usize = 1_000;
    let mut rand = Xorshift::time();
    for _ in 0..Q {
        let abm: Vec<_> = (0..N)
            .map(|_| {
                let m = rand.rand(21) + 1;
                (rand.rand(m) as i64, rand.rand(m) as i64, m as i64)
            })
            .collect();
        if let Some((x, m0)) = linear_congruence(abm.iter().cloned()) {
            assert!(x < m0);
            for (a, b, m) in abm.iter().cloned() {
                assert!(a * x % m == b);
            }
        } else {
            let m0 = abm[1..]
                .iter()
                .fold(abm[0].2, |x, y| lcm(x as u64, y.2 as u64) as i64);
            let x = (0..m0).find(|&x| abm.iter().cloned().all(|(a, b, m)| a * x % m == b));
            assert_eq!(x, None);
        }
    }
}
