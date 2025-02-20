use super::{BarrettReduction, prime_factors};

pub fn primitive_root(p: u64) -> u64 {
    if p == 2 {
        return 1;
    }
    let phi = p - 1;
    let pf = prime_factors(phi);
    let br = BarrettReduction::<u128>::new(p as _);
    (2..)
        .find(|&g| check_primitive_root(g, phi, &br, &pf))
        .unwrap()
}

pub fn check_primitive_root(
    g: u64,
    phi: u64,
    br: &BarrettReduction<u128>,
    pf: &[(u64, u32)],
) -> bool {
    pf.iter().all(|&(q, _)| {
        let mut g = g as u128;
        let mut k = phi / q;
        let mut r: u128 = 1;
        while k > 0 {
            if k & 1 == 1 {
                r = br.rem(r * g);
            }
            g = br.rem(g * g);
            k >>= 1;
        }
        r > 1
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::PrimeList;

    #[test]
    fn test_primitive_root() {
        assert_eq!(3, primitive_root(998244353));
        let pl = PrimeList::new(1000);
        for &p in pl.primes() {
            let g = primitive_root(p);
            let mut x = g;
            for _ in 1..p - 1 {
                assert_ne!(x, 1);
                x = x * g % p;
            }
            assert_eq!(x, 1);
        }
    }

    #[test]
    fn test_primitive_root_prime_power() {
        let pl = PrimeList::new(100);
        for &p in pl.primes().iter().skip(1) {
            for e in 1.. {
                let n = p.pow(e);
                let phi = n - n / p;
                let g = {
                    let pf = prime_factors(phi);
                    let br = BarrettReduction::<u128>::new(n as _);
                    (2..)
                        .find(|&g| check_primitive_root(g, phi, &br, &pf))
                        .unwrap()
                };
                let mut x = g;
                for _ in 1..phi {
                    assert_ne!(x, 1);
                    x = x * g % n;
                }
                assert_eq!(x, 1);
                if n >= 10_000 {
                    break;
                }
            }
        }
    }

    #[test]
    fn test_primitive_root_power_of_two() {
        for e in 3.. {
            let n = 2u64.pow(e);
            let phi = n / 4;
            let g = 5;
            let mut x = g;
            for _ in 1..phi {
                assert_ne!(x, 1);
                x = x * g % n;
            }
            assert_eq!(x, 1);
            if n >= 10_000 {
                break;
            }
        }
    }
}
