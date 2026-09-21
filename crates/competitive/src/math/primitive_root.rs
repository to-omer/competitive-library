use super::{BarrettReduction, Xorshift, prime_factors};

pub fn primitive_root(p: u64) -> u64 {
    if p == 2 {
        return 1;
    }
    let phi = p - 1;
    let pf = prime_factors(phi);
    let br = BarrettReduction::<u128>::new(p as _);
    for g in 2..=3.min(p - 1) {
        if check_primitive_root(g, phi, &br, &pf) {
            return g;
        }
    }
    let mut rng = Xorshift::default();
    loop {
        let g = ((rng.rand64() as u128 * (p - 2) as u128) >> 64) as u64 + 2;
        if check_primitive_root(g, phi, &br, &pf) {
            return g;
        }
    }
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
        let primes: Vec<_> = PrimeList::new(1000).primes().map(u64::from).collect();
        for p in primes {
            let g = primitive_root(p);
            let mut powers = vec![false; p as usize];
            let mut x = 1;
            for _ in 0..p - 1 {
                assert!(!powers[x as usize]);
                powers[x as usize] = true;
                x = x * g % p;
            }
            assert_eq!(x, 1);
            assert!(powers[1..].iter().all(|&seen| seen));
        }
    }

    #[test]
    fn test_check_primitive_root() {
        let mut rng = Xorshift::default();
        let primes: Vec<_> = PrimeList::new(30).primes().skip(1).map(u64::from).collect();
        for _ in 0..1000 {
            let p = primes[rng.random(0..primes.len())];
            let exponent = rng.random(1..=3);
            let n = p.pow(exponent);
            let phi = n - n / p;
            let factors = prime_factors(phi);
            let br = BarrettReduction::<u128>::new(n as _);
            let g = rng.random(1..n);
            if g % p == 0 {
                continue;
            }
            let mut x = 1;
            let order = (1..=phi)
                .find(|_| {
                    x = x * g % n;
                    x == 1
                })
                .unwrap();
            assert_eq!(check_primitive_root(g, phi, &br, &factors), order == phi);
        }
    }
}
