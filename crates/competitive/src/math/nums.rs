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

/// return: (y,z)
///
/// forall (a,b,m), ax = b mod m, where x = y mod z
#[codesnip::entry(include("gcd", "modinv"))]
pub fn linear_congruence<I>(abm: I) -> Option<(u64, u64)>
where
    I: IntoIterator<Item = (u64, u64, u64)>,
{
    let mut x = 0u64;
    let mut m0 = 1u64;
    for (a, b, m) in abm {
        let mut b = b + m - a * x % m;
        if b >= m {
            b -= m;
        }
        let a = a * m0;
        let g = gcd(a, m);
        if b % g != 0 {
            return None;
        }
        let (a, b, m) = (a / g, b / g, m / g);
        x += b * modinv(a, m) % m * m0;
        m0 *= m;
    }
    x %= m0;
    Some((x, m0))
}

#[test]
fn test_linear_congruence() {
    use crate::math::lcm;
    use crate::tools::Xorshift;
    const N: usize = 5;
    const Q: usize = 1_000;
    let mut rng = Xorshift::time();
    for _ in 0..Q {
        let abm: Vec<_> = (0..N)
            .map(|_| {
                let m = rng.gen(2u64..=20);
                (rng.gen(1..m), rng.gen(0..m), m)
            })
            .collect();
        if let Some((x, m0)) = linear_congruence(abm.iter().cloned()) {
            assert!(x < m0);
            for (a, b, m) in abm.iter().cloned() {
                assert!(a * x % m == b);
            }
        } else {
            let m0 = abm[1..].iter().fold(abm[0].2, |x, y| lcm(x, y.2));
            let x = (0..m0).find(|&x| abm.iter().cloned().all(|(a, b, m)| a * x % m == b));
            assert_eq!(x, None);
        }
    }
}
