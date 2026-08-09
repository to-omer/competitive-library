use super::{BarrettReduction, Xorshift, gcd, miller_rabin_with_br};

struct MontgomeryReduction64 {
    modulus: u64,
    inverse: u64,
}

impl MontgomeryReduction64 {
    fn new(modulus: u64) -> Self {
        let mut inverse = modulus;
        for _ in 0..6 {
            inverse = inverse.wrapping_mul(2u64.wrapping_sub(modulus.wrapping_mul(inverse)));
        }
        Self { modulus, inverse }
    }

    fn sub(&self, lhs: u64, rhs: u64) -> u64 {
        let (value, borrow) = lhs.overflowing_sub(rhs);
        value.wrapping_add((borrow as u64).wrapping_neg() & self.modulus)
    }

    fn mul(&self, lhs: u64, rhs: u64) -> u64 {
        let product = lhs as u128 * rhs as u128;
        let (value, borrow) = ((product >> 64) as u64).overflowing_sub(
            (((product as u64).wrapping_mul(self.inverse) as u128 * self.modulus as u128) >> 64)
                as u64,
        );
        value.wrapping_add((borrow as u64).wrapping_neg() & self.modulus)
    }
}

fn find_factor(n: u64) -> Option<u64> {
    let br = BarrettReduction::<u128>::new(n as u128);
    if miller_rabin_with_br(n, &br) {
        return None;
    }
    let mr = MontgomeryReduction64::new(n);
    let mut rng = Xorshift::default();
    let (mut y0, mut c) = (0, n - 1);
    loop {
        let (mut x, mut y, mut ys, mut g, mut q, mut r, mut k) = (0, y0, 0, 1, 1, 1, 0);
        while g == 1 && r <= 1 << 20 {
            x = y;
            while k < r && g == 1 {
                ys = y;
                for _ in 0..1024.min(r - k) {
                    y = mr.sub(mr.mul(y, y), c);
                    q = mr.mul(q, mr.sub(x, y));
                }
                g = gcd(q, n);
                k += 1024;
            }
            k = r;
            r <<= 1;
        }
        if g == n {
            g = 1;
            y = ys;
            while g == 1 {
                y = mr.sub(mr.mul(y, y), c);
                g = gcd(mr.sub(x, y), n);
            }
        }
        if g != 1 && g != n {
            return Some(g);
        }
        y0 = ((rng.rand64() as u128 * (n - 2) as u128) >> 64) as u64 + 2;
        c = ((rng.rand64() as u128 * (n - 1) as u128) >> 64) as u64 + 1;
    }
}

pub fn prime_factors_flatten(mut n: u64) -> Vec<u64> {
    if n == 0 {
        return vec![];
    }
    let k = n.trailing_zeros();
    let mut res = vec![2; k as usize];
    n >>= k;
    while n.is_multiple_of(3) {
        res.push(3);
        n /= 3;
    }
    if n != 1 {
        let mut c = vec![n];
        while let Some(n) = c.pop() {
            if let Some(m) = find_factor(n) {
                c.push(m);
                c.push(n / m);
            } else {
                res.push(n);
            }
        }
    }
    res.sort_unstable();
    res
}

pub fn prime_factors(n: u64) -> Vec<(u64, u32)> {
    let mut res = Vec::new();
    for a in prime_factors_flatten(n) {
        if let Some((p, len)) = res.last_mut()
            && p == &a
        {
            *len += 1;
            continue;
        }
        res.push((a, 1));
    }
    res
}

pub fn divisors(n: u64) -> Vec<u64> {
    let mut d = vec![1u64];
    for (p, c) in prime_factors(n) {
        let k = d.len();
        let mut acc = 1;
        for _ in 0..c {
            acc *= p;
            for i in 0..k {
                d.push(d[i] * acc);
            }
        }
    }
    d.sort_unstable();
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    pub fn naive_divisors(n: u64) -> Vec<u64> {
        let mut res = vec![];
        for i in 1..(n as f32).sqrt() as u64 + 1 {
            if n.is_multiple_of(i) {
                res.push(i);
                if i * i != n {
                    res.push(n / i);
                }
            }
        }
        res.sort_unstable();
        res
    }

    #[test]
    fn test_prime_factors_rho() {
        use crate::{math::miller_rabin, tools::Xorshift};
        const Q: usize = 2_000;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let x = rng.rand64();
            let factors = prime_factors_flatten(x);
            assert!(factors.iter().all(|&p| miller_rabin(p)));
            let p = factors.into_iter().product::<u64>();
            assert_eq!(x, p);
        }
    }

    #[test]
    fn test_divisors() {
        let mut rng = Xorshift::default();
        for n in (1..1000).chain(rng.random_iter(1..=20000000).take(100)) {
            assert_eq!(divisors(n), naive_divisors(n));
        }
    }
}
