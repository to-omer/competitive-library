use crate::math::gcd::modinv;

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

#[cargo_snippet::snippet("BabyStepGiantStep")]
#[cargo_snippet::snippet(include = "modinv")]
#[derive(Clone, Debug)]
pub struct BabyStepGiantStep {
    p: u64,
    r: u64,
    baby: std::collections::HashMap<u64, u64>,
}
#[cargo_snippet::snippet("BabyStepGiantStep")]
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
#[cargo_snippet::snippet]
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
    let x_max = y_max * m - b;
    if y_max == 0 {
        return ans;
    }
    ans += (n - (x_max + a - 1) / a) * y_max;
    ans += floor_sum(y_max, a, m, (a - x_max % a) % a);
    return ans;
}
