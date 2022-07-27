use super::{
    check_primitive_root, gcd, lcm, modinv, prime_factors, primitive_root, BarrettReduction,
    PrimeList, Xorshift,
};
use std::{cell::UnsafeCell, collections::HashMap};

fn pow(x: u64, mut y: u64, br: &BarrettReduction<u128>) -> u64 {
    let mut x = x as u128;
    let mut z: u128 = 1;
    while y > 0 {
        if y & 1 == 1 {
            z = br.rem(z * x);
        }
        x = br.rem(x * x);
        y >>= 1;
    }
    z as u64
}

fn solve_linear_congruence(a: u64, b: u64, m: u64) -> Option<(u64, u64)> {
    let g = gcd(a, m);
    if b % g != 0 {
        return None;
    }
    let (a, b, m) = (a / g, b / g, m / g);
    Some(((b as u128 * modinv(a, m) as u128 % m as u128) as _, m))
}

fn solve_linear_congruences<I>(abm: I) -> Option<(u64, u64)>
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
        x += (b as u128 * modinv(a, m) as u128 % m as u128 * m0 as u128) as u64;
        m0 *= m;
    }
    Some((x, m0))
}

#[derive(Debug)]
struct IndexCalculus {
    primes: PrimeList,
    br_primes: Vec<BarrettReduction<u64>>,
    ic: HashMap<u64, IndexCalculusWithPrimitiveRoot>,
}

impl IndexCalculus {
    fn new() -> Self {
        Self {
            primes: PrimeList::new(2),
            br_primes: Default::default(),
            ic: Default::default(),
        }
    }
    fn discrete_logarithm(&mut self, a: u64, b: u64, p: u64) -> Option<(u64, u64)> {
        let lim = ((((p as f64).log2() * (p as f64).log2().log2()).sqrt() / 2.0 + 1.).exp2() * 0.9)
            as u64;
        self.primes.reserve(lim);
        let primes = self.primes.primes_lte(lim);
        while self.br_primes.len() < primes.len() {
            let br = BarrettReduction::<u64>::new(primes[self.br_primes.len()]);
            self.br_primes.push(br);
        }
        let br_primes = &self.br_primes[..primes.len()];
        self.ic
            .entry(p)
            .or_insert_with(|| IndexCalculusWithPrimitiveRoot::new(p, br_primes))
            .discrete_logarithm(a, b, br_primes)
    }
}

const A: [u32; 150] = [
    62, 61, 60, 60, 59, 58, 58, 58, 57, 56, 56, 56, 56, 55, 55, 55, 54, 54, 54, 53, 53, 53, 53, 52,
    52, 52, 52, 52, 52, 51, 50, 50, 50, 50, 49, 49, 49, 48, 48, 48, 48, 48, 47, 47, 47, 47, 47, 47,
    47, 47, 47, 47, 47, 47, 47, 47, 45, 42, 42, 41, 41, 41, 41, 41, 41, 41, 40, 40, 40, 40, 40, 40,
    40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 38, 38, 38, 38, 38, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 31, 31, 31, 31, 31,
    31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 22, 22, 22, 22,
    22, 22, 22, 22, 22, 22,
];

fn factorize_smooth(mut x: u64, row: &mut [u64], br_primes: &[BarrettReduction<u64>]) -> bool {
    for (j, (&br, r)) in br_primes.iter().zip(row).enumerate() {
        *r = 0;
        loop {
            let (div, rem) = br.div_rem(x);
            if rem != 0 {
                break;
            }
            *r += 1;
            x = div;
        }
        if j < 150 && x >= (1u64 << A[j]) {
            break;
        }
    }
    x == 1
}

#[derive(Debug)]
struct QdrtPowPrec {
    br_qdrt: BarrettReduction<u64>,
    p0: Vec<u64>,
    p1: Vec<u64>,
    p2: Vec<u64>,
    p3: Vec<u64>,
}

impl QdrtPowPrec {
    fn new(a: u64, ord: u64, br: &BarrettReduction<u128>) -> Self {
        let qdrt = (ord as f64).powf(0.25).ceil() as u64;
        let br_qdrt = BarrettReduction::<u64>::new(qdrt);
        let mut p0 = Vec::with_capacity(qdrt as usize);
        let mut p1 = Vec::with_capacity(qdrt as usize);
        let mut p2 = Vec::with_capacity(qdrt as usize);
        let mut p3 = Vec::with_capacity(qdrt as usize);
        let mut acc = 1u64;
        for _ in 0..qdrt {
            p0.push(acc);
            acc = br.rem(acc as u128 * a as u128) as u64;
        }
        let a = acc;
        acc = 1;
        for _ in 0..qdrt {
            p1.push(acc);
            acc = br.rem(acc as u128 * a as u128) as u64;
        }
        let a = acc;
        acc = 1;
        for _ in 0..qdrt {
            p2.push(acc);
            acc = br.rem(acc as u128 * a as u128) as u64;
        }
        let a = acc;
        acc = 1;
        for _ in 0..qdrt {
            p3.push(acc);
            acc = br.rem(acc as u128 * a as u128) as u64;
        }
        Self {
            br_qdrt,
            p0,
            p1,
            p2,
            p3,
        }
    }
    fn pow(&self, mut k: u64, br: &BarrettReduction<u128>) -> u64 {
        let (a, b) = self.br_qdrt.div_rem(k);
        let mut x = self.p0[b as usize];
        k = a;
        if k > 0 {
            let (a, b) = self.br_qdrt.div_rem(k);
            x = br.rem(x as u128 * self.p1[b as usize] as u128) as u64;
            k = a;
        }
        if k > 0 {
            let (a, b) = self.br_qdrt.div_rem(k);
            x = br.rem(x as u128 * self.p2[b as usize] as u128) as u64;
            k = a;
        }
        if k > 0 {
            let (_, b) = self.br_qdrt.div_rem(k);
            x = br.rem(x as u128 * self.p3[b as usize] as u128) as u64;
        }
        x
    }
}

fn index_calculus_for_primitive_root(
    p: u64,
    ord: u64,
    br_primes: &[BarrettReduction<u64>],
    prec: &QdrtPowPrec,
) -> Vec<u64> {
    let br_ord = BarrettReduction::<u128>::new(ord as u128);
    let mul = |x: u64, y: u64| br_ord.rem(x as u128 * y as u128) as u64;
    let sub = |x: u64, y: u64| if x < y { x + ord - y } else { x - y };

    let pc = br_primes.len();
    let mut mat: Vec<Vec<u64>> = vec![];
    let mut rows: Vec<Vec<u64>> = vec![];

    let mut rng = Xorshift::default();
    let br = BarrettReduction::<u128>::new(p as u128);

    for i in 0..pc {
        for ri in 0usize.. {
            let mut row = vec![0u64; pc + 1];
            let mut kk = rng.rand(ord - 1) + 1;
            let mut gkk = prec.pow(kk, &br);
            let mut k = kk;
            let mut gk = gkk;
            while ri >= rows.len() {
                row[pc] = k;
                if factorize_smooth(gk, &mut row, br_primes) {
                    rows.push(row);
                    break;
                }
                if k + kk < ord {
                    k += kk;
                    gk = br.rem(gk as u128 * gkk as u128) as u64;
                } else {
                    kk = rng.rand(ord - 1) + 1;
                    gkk = prec.pow(kk, &br);
                    k = kk;
                    gk = gkk;
                }
            }
            let row = &mut rows[ri];
            for j in 0..i {
                if row[j] != 0 {
                    let b = mul(modinv(mat[j][j], ord), row[j]);
                    for (r, a) in row[j..].iter_mut().zip(&mat[j][j..]) {
                        *r = sub(*r, mul(*a, b));
                    }
                }
                assert_eq!(row[j], 0);
            }
            if gcd(row[i], ord) == 1 {
                let last = rows.len() - 1;
                rows.swap(ri, last);
                mat.push(rows.pop().unwrap());
                break;
            }
        }
    }
    for i in (0..pc).rev() {
        for j in i + 1..pc {
            mat[i][pc] = sub(mat[i][pc], mul(mat[i][j], mat[j][pc]));
        }
        mat[i][pc] = mul(mat[i][pc], modinv(mat[i][i], ord));
    }
    (0..pc).map(|i| (mat[i][pc])).collect()
}

#[derive(Debug)]
struct IndexCalculusWithPrimitiveRoot {
    p: u64,
    ord: u64,
    prec: QdrtPowPrec,
    coeff: Vec<u64>,
}

impl IndexCalculusWithPrimitiveRoot {
    fn new(p: u64, br_primes: &[BarrettReduction<u64>]) -> Self {
        let ord = p - 1;
        let g = primitive_root(p);
        let br = BarrettReduction::<u128>::new(p as u128);
        let prec = QdrtPowPrec::new(g, ord, &br);
        let coeff = index_calculus_for_primitive_root(p, ord, br_primes, &prec);
        Self {
            p,
            ord,
            prec,
            coeff,
        }
    }
    fn index_calculus(&self, a: u64, br_primes: &[BarrettReduction<u64>]) -> Option<u64> {
        let p = self.p;
        let ord = self.ord;
        let br = BarrettReduction::<u128>::new(p as u128);
        let a = br.rem(a as _) as u64;
        if a == 1 {
            return Some(0);
        }
        if p == 2 {
            return None;
        }

        let mut rng = Xorshift::time();
        let mut row = vec![0u64; br_primes.len()];
        let mut kk = rng.rand(ord - 1) + 1;
        let mut gkk = self.prec.pow(kk, &br);
        let mut k = kk;
        let mut gk = br.rem(gkk as u128 * a as u128) as u64;
        loop {
            if factorize_smooth(gk, &mut row, br_primes) {
                let mut res = ord - k;
                for (&c, &r) in self.coeff.iter().zip(&row) {
                    for _ in 0..r {
                        res += c;
                        if res >= ord {
                            res -= ord;
                        }
                    }
                }
                return Some(res);
            }
            if k + kk < ord {
                k += kk;
                gk = br.rem(gk as u128 * gkk as u128) as u64;
            } else {
                kk = rng.rand(ord - 1) + 1;
                gkk = self.prec.pow(kk, &br);
                k = kk;
                gk = br.rem(gkk as u128 * a as u128) as u64;
            }
        }
    }
    fn discrete_logarithm(
        &self,
        a: u64,
        b: u64,
        br_primes: &[BarrettReduction<u64>],
    ) -> Option<(u64, u64)> {
        let p = self.p;
        let ord = self.ord;
        let br = BarrettReduction::<u128>::new(p as u128);
        let a = br.rem(a as _) as u64;
        let b = br.rem(b as _) as u64;
        if a == 0 {
            return if b == 0 { Some((1, 1)) } else { None };
        }
        if b == 0 {
            return None;
        }

        let x = self.index_calculus(a, br_primes)?;
        let y = self.index_calculus(b, br_primes)?;
        solve_linear_congruence(x, y, ord)
    }
}

thread_local!(
    static IC: UnsafeCell<IndexCalculus> = UnsafeCell::new(IndexCalculus::new());
);

pub fn discrete_logarithm_prime_mod(a: u64, b: u64, p: u64) -> Option<u64> {
    IC.with(|ic| unsafe { &mut *ic.get() }.discrete_logarithm(a, b, p))
        .map(|t| t.0)
}

/// a^x ≡ b (mod n), a has order p^e
fn pohlig_hellman_prime_power_order(a: u64, b: u64, n: u64, p: u64, e: u32) -> Option<u64> {
    let br = BarrettReduction::<u128>::new(n as u128);
    let mul = |x: u64, y: u64| br.rem(x as u128 * y as u128) as u64;
    let block_size = (p as f64).sqrt().ceil() as u64;
    let mut baby = HashMap::<u64, u64>::new();
    let g = pow(a, p.pow(e - 1), &br);
    let mut xj = 1;
    for j in 0..block_size {
        baby.entry(xj).or_insert(j);
        xj = mul(xj, g);
    }
    let xi = modinv(xj, n);
    let mut t = 0u64;
    for k in 0..e {
        let mut h = pow(mul(modinv(pow(a, t, &br), n), b), p.pow(e - 1 - k), &br);
        let mut ok = false;
        for i in (0..block_size * block_size).step_by(block_size as usize) {
            if let Some(j) = baby.get(&h) {
                t += (i + j) * p.pow(k);
                ok = true;
                break;
            }
            h = mul(h, xi);
        }
        if !ok {
            return None;
        }
    }
    Some(t)
}

/// a^x ≡ b (mod p^e)
fn discrete_logarithm_prime_power(a: u64, b: u64, p: u64, e: u32) -> Option<(u64, u64)> {
    assert_ne!(p, 0);
    assert_ne!(e, 0);
    let n = p.pow(e);
    assert!(a < n);
    assert!(b < n);
    assert_eq!(gcd(a, p), 1);
    if p == 1 {
        return Some((0, 1));
    }
    if a == 0 {
        return if b == 0 { Some((1, 1)) } else { None };
    }
    if b == 0 {
        return None;
    }
    if e == 1 {
        return IC.with(|ic| unsafe { &mut *ic.get() }.discrete_logarithm(a, b, p));
    }
    let br = BarrettReduction::<u128>::new(n as _);
    if p == 2 {
        if e >= 3 {
            if a % 4 == 1 && b % 4 != 1 {
                return None;
            }
            let aa = if a % 4 == 1 { a } else { n - a };
            let bb = if b % 4 == 1 { b } else { n - b };
            let g = 5;
            let ord = n / 4;
            let x = pohlig_hellman_prime_power_order(g, aa, n, p, e - 2)?;
            let y = pohlig_hellman_prime_power_order(g, bb, n, p, e - 2)?;
            let t = solve_linear_congruence(x, y, ord)?;
            match (a % 4 == 1, b % 4 == 1) {
                (true, true) => Some(t),
                (false, true) if t.0 % 2 == 0 => Some((t.0, lcm(t.1, 2))),
                (false, false) if t.0 % 2 == 1 => Some((t.0, lcm(t.1, 2))),
                _ => None,
            }
        } else if a == 1 {
            if b == 1 {
                Some((0, 1))
            } else {
                None
            }
        } else {
            assert_eq!(a, 3);
            if b == 1 {
                Some((0, 2))
            } else if b == 3 {
                Some((1, 2))
            } else {
                None
            }
        }
    } else {
        let ord = n - n / p;
        let pf_ord = prime_factors(ord);
        let g = (2..)
            .find(|&g| check_primitive_root(g, ord, &br, &pf_ord))
            .unwrap();
        let mut pf_p = prime_factors(p - 1);
        pf_p.push((p, e - 1));
        let mut abm = vec![];
        for (q, c) in pf_p {
            let m = q.pow(c);
            let d = ord / m;
            let gg = pow(g, d, &br);
            let aa = pow(a, d, &br);
            let bb = pow(b, d, &br);
            let x = pohlig_hellman_prime_power_order(gg, aa, n, q, c)?;
            let y = pohlig_hellman_prime_power_order(gg, bb, n, q, c)?;
            abm.push((x, y, m));
        }
        solve_linear_congruences(abm)
    }
}

/// a^x ≡ b (mod n)
pub fn discrete_logarithm(a: u64, b: u64, n: u64) -> Option<u64> {
    let a = a % n;
    let b = b % n;
    let d = 2.max(64 - n.leading_zeros() as u64);
    let mut pw = 1 % n;
    for i in 0..d {
        if pw == b {
            return Some(i);
        }
        pw = (pw as u128 * a as u128 % n as u128) as u64;
    }
    let g = gcd(pw, n);
    if b % g != 0 {
        return None;
    }
    let n = n / g;
    let b = (b as u128 * modinv(pw, n) as u128 % n as u128) as u64;
    let pf = prime_factors(n);
    let mut abm = vec![];
    for (p, e) in pf {
        let q = p.pow(e);
        let x = discrete_logarithm_prime_power(a % q, b % q, p, e)?;
        abm.push((1, x.0, x.1));
    }
    solve_linear_congruences(abm).map(|x| x.0 + d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::MultiplicativeOperation, algorithm::BabyStepGiantStep, num::mint_basic::DynMIntU64,
    };

    fn check(a: u64, b: u64, n: u64, l: Option<u64>, bsgs: bool) {
        if let Some(l) = l {
            let pw = pow(a, l, &BarrettReduction::<u128>::new(n as _));
            assert_eq!(
                b, pw,
                "expected {} ^ {} = {} mod {}, but {}",
                a, l, b, n, pw
            );
        }
        if !bsgs {
            return;
        }
        let res = if b == 1 {
            Some(0)
        } else {
            (|| {
                let d = 2.max(64 - n.leading_zeros() as u64);
                let mut pw = 1 % n;
                for i in 0..d {
                    if pw == b {
                        return Some(i);
                    }
                    pw = (pw as u128 * a as u128 % n as u128) as u64;
                }
                if pw == b {
                    return Some(d);
                }
                let g = gcd(pw, n);
                if b % g != 0 {
                    return None;
                }
                let n = n / g;
                DynMIntU64::set_mod(n);
                let b = (b as u128 * modinv(pw, n) as u128 % n as u128) as u64;
                BabyStepGiantStep::<MultiplicativeOperation<_>>::new(
                    n as usize + 1,
                    DynMIntU64::new(a),
                )
                .solve(DynMIntU64::new(b))
                .map(|x| x as u64 + d)
            })()
        };
        assert_eq!(res, l);
    }

    #[test]
    fn test_ic_small_prime() {
        let pl = PrimeList::new(30);
        for &p in pl.primes() {
            for a in 1..p {
                for b in 1..p {
                    let l = discrete_logarithm_prime_mod(a, b, p);
                    check(a, b, p, l, true);
                }
            }
        }
    }

    #[test]
    fn test_ic_medium_prime() {
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for &p in [998_244_353, 1_000_000_007].iter() {
            for i in 0..Q {
                let (a, b) = rng.gen((1..p, 1..p));
                let l = discrete_logarithm_prime_mod(a, b, p);
                check(a, b, p, l, i >= Q - 20);
            }
        }
    }

    #[test]
    fn test_ic_large_prime() {
        let mut rng = Xorshift::default();
        let p = 1_000_000_000_000 - 11;
        for _ in 0..20 {
            let (a, b) = rng.gen((1..p, 1..p));
            let l = discrete_logarithm_prime_mod(a, b, p);
            check(a, b, p, l, false);
        }
    }

    #[test]
    fn test_pohlig_hellman_prime_power_order() {
        let p = 2u64;
        for e in 3..40 {
            let a = 5;
            for b in (1..100.min(p.pow(e))).step_by(4) {
                let l = pohlig_hellman_prime_power_order(a, b, p.pow(e), p, e - 2);
                check(a, b, p.pow(e), l, false);
            }
        }
    }

    #[test]
    fn test_discrete_logarithm_prime_power_small() {
        let pl = PrimeList::new(30);
        for &p in pl.primes() {
            for e in 1.. {
                for a in 1..p {
                    for b in 1..p {
                        let l = discrete_logarithm_prime_power(a, b, p, e).map(|t| t.0);
                        check(a, b, p.pow(e), l, true);
                    }
                }
                if p.pow(e) >= 1_000 {
                    break;
                }
            }
        }
    }

    #[test]
    fn test_discrete_logarithm_small() {
        for n in 1..50 {
            for a in 1..n {
                for b in 1..n {
                    let l = discrete_logarithm(a, b, n);
                    check(a, b, n, l, true);
                }
            }
        }
    }

    #[test]
    fn test_discrete_logarithm_medium() {
        const Q: usize = 10_000;
        let mut rng = Xorshift::default();
        for i in 0..Q {
            let n = rng.gen(
                1..if i < Q - 1 {
                    1_000_000
                } else {
                    1_000_000_000_000
                },
            );
            let a = rng.gen(0..n);
            let b = rng.gen(0..n);
            let l = discrete_logarithm(a, b, n);
            check(a, b, n, l, i >= Q - 200);
        }
    }

    #[test]
    fn test_discrete_logarithm_large() {
        let mut rng = Xorshift::default();
        for _ in 0..20 {
            let n = rng.gen(1..1_000_000_000_000_000_000);
            let a = rng.gen(0..n);
            let b = rng.gen(0..n);
            let l = discrete_logarithm(a, b, n);
            check(a, b, n, l, false);
        }
    }
}
