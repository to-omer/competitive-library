pub fn gcd_loop(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        a %= b;
        std::mem::swap(&mut a, &mut b);
    }
    a
}

#[codesnip::entry]
/// binary gcd
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    let u = a.trailing_zeros();
    let v = b.trailing_zeros();
    a >>= u;
    b >>= v;
    let k = std::cmp::min(u, v);
    while a != b {
        if a < b {
            std::mem::swap(&mut a, &mut b);
        }
        a -= b;
        a >>= a.trailing_zeros();
    }
    a << k
}

#[codesnip::entry(include("gcd"))]
pub fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}

// ax + by = gcd(a, b)
// a, b -> gcd(a, b), x, y
pub fn extgcd_recurse(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = extgcd_recurse(b, a % b);
        (g, y, x - (a / b) * y)
    }
}

#[codesnip::entry]
pub fn extgcd(mut a: i64, mut b: i64) -> (i64, i64, i64) {
    let (mut u, mut v, mut x, mut y) = (1, 0, 0, 1);
    while a != 0 {
        let k = b / a;
        x -= k * u;
        y -= k * v;
        b -= k * a;
        std::mem::swap(&mut x, &mut u);
        std::mem::swap(&mut y, &mut v);
        std::mem::swap(&mut b, &mut a);
    }
    (b, x, y)
}

/// Requires nonnegative inputs.
pub fn extgcd_binary(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        return (a, 1, 0);
    }
    if a == 0 {
        return (b, 0, 1);
    }
    let k = (a | b).trailing_zeros();
    let (mut c, mut d) = ((a >> k) as u64, (b >> k) as u64);
    let swapped = d & 1 == 0;
    if swapped {
        std::mem::swap(&mut c, &mut d);
    }
    let (mut a, mut b) = (c, d);
    // c*u == a and c*s == b modulo the odd d, with 0 <= u,s <= d.
    let (mut u, mut s) = (1u64, 0u64);
    let shift = a.trailing_zeros();
    a >>= shift;
    for _ in 0..shift {
        if u & 1 == 1 {
            u += d;
        }
        u /= 2;
    }
    while a != b {
        if b < a {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut u, &mut s);
        }
        b -= a;
        if s < u {
            s += d;
        }
        s -= u;
        let shift = b.trailing_zeros();
        b >>= shift;
        for _ in 0..shift {
            if s & 1 == 1 {
                s += d;
            }
            s /= 2;
        }
    }
    let x = s as i64;
    let y = ((a as i128 - c as i128 * s as i128) / d as i128) as i64;
    let g = (a << k) as i64;
    if swapped { (g, y, x) } else { (g, x, y) }
}

/// Requires 0 < a < m < 2^63 and gcd(a, m) == 1.
pub fn modinv_recurse(a: u64, m: u64) -> u64 {
    extgcd_recurse(a as i64, m as i64).1.rem_euclid(m as i64) as u64
}

#[codesnip::entry(include("extgcd"))]
/// Requires 0 < a < m < 2^63 and gcd(a, m) == 1.
pub fn modinv(a: u64, m: u64) -> u64 {
    let (mut a, mut b) = (a as i64, m as i64);
    let (mut u, mut x) = (1, 0);
    while a != 0 {
        let k = b / a;
        x -= k * u;
        b -= k * a;
        std::mem::swap(&mut x, &mut u);
        std::mem::swap(&mut b, &mut a);
    }
    (if x < 0 { x + m as i64 } else { x }) as _
}

/// 0 < a < p < 2^63, gcd(a, p) == 1, p is odd
pub fn modinv_extgcd_binary(mut a: u64, p: u64) -> u64 {
    let (mut b, mut u, mut s) = (p, 1, 0);
    let k = a.trailing_zeros();
    a >>= k;
    for _ in 0..k {
        if u & 1 == 1 {
            u += p;
        }
        u /= 2;
    }
    while a != b {
        if b < a {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut u, &mut s);
        }
        b -= a;
        if s < u {
            s += p;
        }
        s -= u;
        let k = b.trailing_zeros();
        b >>= k;
        for _ in 0..k {
            if s & 1 == 1 {
                s += p;
            }
            s /= 2;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        Xorshift,
        testutil::{integer_cases, integer_pairs},
    };
    const A: i64 = i64::MAX;

    #[test]
    fn test_gcd() {
        let mut rng = Xorshift::default();
        let pairs = integer_pairs(0..=u64::MAX).chain(
            rng.random_iter((0..=u64::MAX, 0..=u64::MAX))
                .take(3_000_000),
        );
        for (a, b) in (0..=30)
            .flat_map(|a| (0..=30).map(move |b| (a, b)))
            .chain(pairs)
        {
            assert_eq!(gcd_loop(a, b), gcd(a, b));
        }
    }

    #[test]
    fn test_extgcd() {
        let mut rng = Xorshift::default();
        for (a, b) in integer_pairs(-A..=A).chain(rng.random_iter((-A..=A, -A..=A)).take(1_400_000))
        {
            for (g, x, y) in [extgcd(a, b), extgcd_recurse(a, b)] {
                assert_eq!(
                    g.unsigned_abs(),
                    gcd_loop(a.unsigned_abs(), b.unsigned_abs())
                );
                assert_eq!(a as i128 * x as i128 + b as i128 * y as i128, g as i128);
            }
        }
    }

    #[test]
    fn test_extgcd_binary() {
        let mut rng = Xorshift::default();
        for (a, b) in integer_pairs(0..=A).chain(rng.random_iter((0..=A, 0..=A)).take(1_500_000)) {
            let (g, x, y) = extgcd_binary(a, b);
            assert_eq!(g as u64, gcd_loop(a as u64, b as u64));
            assert_eq!(a as i128 * x as i128 + b as i128 * y as i128, g as i128);
        }
    }

    #[test]
    fn test_modinv() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        let boundaries = integer_cases(2..=A as u64).flat_map(|m| [(1, m), (m - 1, m)]);
        let general = integer_cases(2..=A as u64)
            .chain(input_rng.random_iter(2..=A as u64).take(1_500_000))
            .map(|m| {
                let a = rng
                    .random_iter(1..m)
                    .find(|&a| gcd_loop(a, m) == 1)
                    .unwrap();
                (a, m)
            });
        for (a, m) in boundaries.chain(general) {
            for x in [modinv(a, m), modinv_recurse(a, m)] {
                assert!(x < m);
                assert_eq!(a as u128 * x as u128 % m as u128, 1);
            }
        }
    }

    #[test]
    fn test_modinv_extgcd_binary() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        let boundaries = integer_cases(3..=A as u64)
            .filter(|&m| m & 1 == 1)
            .flat_map(|m| [(1, m), (m - 1, m)]);
        let general = integer_cases(3..=A as u64)
            .chain(input_rng.random_iter(3..=A as u64).take(2_500_000))
            .filter(|&m| m & 1 == 1)
            .map(|m| {
                let a = rng
                    .random_iter(1..m)
                    .find(|&a| gcd_loop(a, m) == 1)
                    .unwrap();
                (a, m)
            });
        for (a, m) in boundaries.chain(general) {
            let x = modinv_extgcd_binary(a, m);
            assert!(x < m);
            assert_eq!(a as u128 * x as u128 % m as u128, 1);
        }
    }
}
