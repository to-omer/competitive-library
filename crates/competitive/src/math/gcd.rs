#[codesnip::entry]
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        a %= b;
        std::mem::swap(&mut a, &mut b);
    }
    a
}

#[codesnip::entry]
pub fn gcd_binary(mut a: u64, mut b: u64) -> u64 {
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

#[test]
fn test_gcd() {
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    const Q: usize = 10_000;
    for _ in 0..Q {
        let a = rand.rand64();
        let b = rand.rand64();
        assert_eq!(gcd(a, b), gcd_binary(a, b));
    }
    assert_eq!(gcd(0, 0), gcd_binary(0, 0));
    assert_eq!(gcd(0, 100), gcd_binary(0, 100));
}

#[codesnip::entry(include("gcd"))]
pub fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}

// ax + by = gcd(a, b)
// a, b -> gcd(a, b), x, y
#[codesnip::entry]
pub fn extgcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = extgcd(b, a % b);
        (g, y, x - (a / b) * y)
    }
}

pub fn extgcd_loop(mut a: i64, mut b: i64) -> (i64, i64, i64) {
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

#[test]
fn test_extgcd() {
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    const Q: usize = 10_000;
    for _ in 0..Q {
        let a = rand.rand(1_000_000_007) as i64;
        let b = rand.rand(1_000_000_007) as i64;
        let (g, x, y) = extgcd_loop(a, b);
        assert_eq!(a * x + b * y, g);
    }
}

pub fn extgcd_binary(mut a: i64, mut b: i64) -> (i64, i64, i64) {
    if b == 0 {
        return (a, 1, 0);
    } else if a == 0 {
        return (b, 1, 0);
    }
    let k = (a | b).trailing_zeros();
    a >>= k;
    b >>= k;
    let (c, d) = (a, b);
    let (mut u, mut v, mut s, mut t) = (1, 0, 0, 1);
    while a & 1 == 0 {
        a /= 2;
        if u & 1 == 1 || v & 1 == 1 {
            u += d;
            v -= c;
        }
        u /= 2;
        v /= 2;
    }
    while a != b {
        if b & 1 == 0 {
            b /= 2;
            if s & 1 == 1 || t & 1 == 1 {
                s += d;
                t -= c;
            }
            s /= 2;
            t /= 2;
        } else if b < a {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut u, &mut s);
            std::mem::swap(&mut v, &mut t);
        } else {
            b -= a;
            s -= u;
            t -= v;
        }
    }
    (a << k, s, t)
}

#[test]
fn test_extgcd_binary() {
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    const Q: usize = 10_000;
    for _ in 0..Q {
        let a = rand.rand(1_000_000_007) as i64;
        let b = rand.rand(1_000_000_007) as i64;
        let (g, x, y) = extgcd_binary(a, b);
        assert_eq!(a * x + b * y, g);
    }
}

#[codesnip::entry(include("extgcd"))]
pub fn modinv(a: i64, m: i64) -> i64 {
    (extgcd(a, m).1 % m + m) % m
}

pub fn modinv_loop(a: i64, m: i64) -> i64 {
    (extgcd_loop(a, m).1 % m + m) % m
}

#[test]
fn test_modinv() {
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    const Q: usize = 10_000;
    for _ in 0..Q {
        let m = rand.rand(1_000_000_009) + 1;
        let a = rand.rand(m - 1) + 1;
        let g = gcd(a, m);
        let m = m / g;
        let a = a / g;
        let x = modinv(a as i64, m as i64) as u64;
        assert!(x < m);
        assert_eq!(a * x % m, 1);
    }
}

/// 0 < a < p, gcd(a, p) == 1, p is prime > 2
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

#[test]
fn test_modinv_extgcd_binary() {
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    const Q: usize = 10_000;
    for _ in 0..Q {
        let m = rand.rand(1_000_000_009) + 1;
        let m = m >> m.trailing_zeros();
        let a = rand.rand(m - 1) + 1;
        let g = gcd(a, m);
        let m = m / g;
        let a = a / g;
        let x = modinv_extgcd_binary(a, m);
        assert!(x < m);
        assert_eq!(a * x % m, 1);
    }
}
