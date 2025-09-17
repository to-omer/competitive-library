use super::Unsigned;

/// return: (y,z)
///
/// ax = b mod m, where x = y mod z
pub fn solve_linear_congruence<T>(a: T, b: T, m: T) -> Option<(T, T)>
where
    T: Unsigned,
{
    let g = a.gcd(m);
    if b % g != T::zero() {
        return None;
    }
    let (a, b, m) = (a / g, b / g, m / g);
    Some((b * a.mod_inv(m) % m, m))
}

/// return: (y,z)
///
/// forall (a,b,m), ax = b mod m, where x = y mod z
pub fn solve_simultaneous_linear_congruence<T, I>(abm: I) -> Option<(T, T)>
where
    T: Unsigned,
    I: IntoIterator<Item = (T, T, T)>,
{
    let mut x = T::zero();
    let mut m0 = T::one();
    for (a, b, m) in abm {
        let mut b = b + m - a * x % m;
        if b >= m {
            b -= m;
        }
        let a = a * m0;
        let (y, z) = solve_linear_congruence(a, b, m)?;
        x += y * m0;
        m0 *= z;
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
    let mut rng = Xorshift::new();
    for _ in 0..Q {
        let abm: Vec<_> = (0..N)
            .map(|_| {
                let m = rng.random(2u64..=20);
                (rng.random(1..m), rng.random(0..m), m)
            })
            .collect();
        if let Some((x, m0)) = solve_simultaneous_linear_congruence(abm.iter().cloned()) {
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
