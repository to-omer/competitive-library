use super::{gcd_binary, miller_rabin_with_br, BarrettReduction};

fn find_factor(n: u64) -> Option<u64> {
    let br = BarrettReduction::<u128>::new(n as u128);
    if miller_rabin_with_br(n, &br) {
        return None;
    }
    let m = 1u64 << ((63 - n.leading_zeros()) / 5);
    let sub = |x: u64, y: u64| if x > y { x - y } else { y - x };
    let mul = |x: u64, y: u64| br.rem(x as u128 * y as u128) as u64;
    for c in 12.. {
        let f = |x: u64| (br.rem(x as u128 * x as u128) + c) as u64;
        let (mut x, mut y, mut r, mut g, mut k, mut ys) = (0, 2, 1, 1, 0, 0);
        while g == 1 {
            x = y;
            for _ in 0..r {
                y = f(y);
            }
            while r > k && g == 1 {
                ys = y;
                let mut q = 1;
                for _ in 0..m.min(r - k) {
                    y = f(y);
                    q = mul(q, sub(x, y));
                }
                g = gcd_binary(q, n);
                k += m;
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
            return Some(g);
        }
    }
    unreachable!();
}

pub fn prime_factors_flatten(mut n: u64) -> Vec<u64> {
    let k = n.trailing_zeros();
    let mut res = vec![2; k as usize];
    n >>= k;
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
        if let Some((p, len)) = res.last_mut() {
            if p == &a {
                *len += 1;
                continue;
            }
        }
        res.push((a, 1));
    }
    res
}

pub fn divisors(n: u64) -> Vec<u64> {
    let mut d = vec![1u64];
    for (p, c) in prime_factors(n) {
        let k = d.len();
        let mut acc = p;
        for _ in 0..c {
            for i in 0..k {
                d.push(d[i] * acc);
            }
            acc *= p;
        }
    }
    d.sort_unstable();
    d
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
