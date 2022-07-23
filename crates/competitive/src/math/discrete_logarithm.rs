use super::{gcd_binary, primitive_root, BarrettReduction, PrimeList, Xorshift};
use std::{cell::UnsafeCell, collections::HashMap, mem::swap};

fn inv(x: u64, p: u64) -> u64 {
    let (mut a, mut b) = (x as i64, p as i64);
    let (mut u, mut x) = (1, 0);
    while a != 0 {
        let k = b / a;
        x -= k * u;
        b -= k * a;
        swap(&mut x, &mut u);
        swap(&mut b, &mut a);
    }
    (if x < 0 { x + p as i64 } else { x }) as u64
}

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

#[derive(Debug)]
struct IndexCalculus {
    primes: PrimeList,
    ic: HashMap<u64, IndexCalculusWithPrimitiveRoot>,
}

impl IndexCalculus {
    fn new() -> Self {
        Self {
            primes: PrimeList::new(2),
            ic: Default::default(),
        }
    }
    fn discrete_logarithm(&mut self, a: u64, b: u64, p: u64) -> Option<u64> {
        let lim = (((p as f64).log2() * (p as f64).log2().log2()).sqrt() / 2. + 1.).exp2() as u64;
        self.primes.reserve(lim);
        let primes = self.primes.primes_lte(lim);
        self.ic
            .entry(p)
            .or_insert_with(|| IndexCalculusWithPrimitiveRoot::new(p, primes))
            .discrete_logarithm(a, b, primes)
    }
}

fn index_calculus_for_primitive_root(p: u64, phi: u64, g: u64, primes: &[u64]) -> Vec<u64> {
    let br = BarrettReduction::<u128>::new(phi as u128);
    let mul = |x: u64, y: u64| br.rem(x as u128 * y as u128) as u64;
    let sub = |x: u64, y: u64| if x < y { x + phi - y } else { x - y };

    let pc = primes.len();
    let mut mat: Vec<Vec<u64>> = vec![];
    let mut rows: Vec<Vec<u64>> = vec![];

    let mut _stat = (0usize, 0usize, 0usize);
    let mut rng = Xorshift::default();
    let br = BarrettReduction::<u128>::new(p as u128);

    for i in 0..pc {
        for ri in 0usize.. {
            while ri >= rows.len() {
                let k = rng.rand(phi - 1) + 1;
                let gk = pow(g, k, &br);

                _stat.0 += 1;
                let mut row = vec![0u64; pc + 1];
                let mut x = gk;
                for (j, &q) in primes.iter().enumerate() {
                    if j == 9 && x >= 1_000_000_000_000_000 || j == 29 && x >= 1_000_000_000_000 {
                        break;
                    }
                    while x % q == 0 {
                        row[j] += 1;
                        x /= q;
                    }
                }
                row[pc] = k;
                if x == 1 {
                    _stat.1 += 1;
                    rows.push(row);
                }
            }
            let row = &mut rows[ri];
            for j in 0..i {
                if row[j] != 0 {
                    let b = mul(inv(mat[j][j], phi), row[j]);
                    for (r, a) in row[j..].iter_mut().zip(&mat[j][j..]) {
                        *r = sub(*r, mul(*a, b));
                    }
                }
                assert_eq!(row[j], 0);
            }
            if gcd_binary(row[i], phi) == 1 {
                _stat.2 += 1;
                let last = rows.len() - 1;
                rows.swap(ri, last);
                mat.push(rows.pop().unwrap());
                break;
            }
        }
        // eprintln!("_stat = {:?}", _stat);
        // eprintln!("rows.len() = {:?}", rows.len());
    }
    for i in (0..pc).rev() {
        for j in i + 1..pc {
            mat[i][pc] = sub(mat[i][pc], mul(mat[i][j], mat[j][pc]));
        }
        mat[i][pc] = mul(mat[i][pc], inv(mat[i][i], phi));
    }
    (0..pc).map(|i| (mat[i][pc])).collect()
}

#[derive(Debug)]
struct IndexCalculusWithPrimitiveRoot {
    p: u64,
    phi: u64,
    g: u64,
    coeff: Vec<u64>,
}

impl IndexCalculusWithPrimitiveRoot {
    fn new(p: u64, primes: &[u64]) -> Self {
        let phi = p - 1;
        let g = primitive_root(p);
        let coeff = index_calculus_for_primitive_root(p, phi, g, primes);
        Self { p, phi, g, coeff }
    }
    fn index_calculus(&self, a: u64, primes: &[u64]) -> Option<u64> {
        let p = self.p;
        let phi = self.phi;
        let g = self.g;
        let br = BarrettReduction::<u128>::new(p as u128);
        let a = br.rem(a as _) as u64;
        if a == 1 {
            return Some(0);
        }
        if p == 2 {
            return None;
        }

        let mut rng = Xorshift::time();
        loop {
            let k = rng.rand(phi - 1) + 1;
            let gk = pow(g, k, &br);

            let mut x = br.rem(gk as u128 * a as u128) as u64;
            for (j, &q) in primes.iter().enumerate() {
                if j == 9 && x >= 1_000_000_000_000_000 || j == 29 && x >= 1_000_000_000_000 {
                    break;
                }
                while x % q == 0 {
                    x /= q;
                }
            }
            if x == 1 {
                let mut x = br.rem(gk as u128 * a as u128) as u64;
                let mut res = phi - k;
                for (j, &q) in primes.iter().enumerate() {
                    while x % q == 0 {
                        res += self.coeff[j];
                        if res >= phi {
                            res -= phi;
                        }
                        x /= q;
                    }
                }
                return Some(res);
            }
        }
    }
    fn discrete_logarithm(&self, a: u64, b: u64, primes: &[u64]) -> Option<u64> {
        let p = self.p;
        let phi = self.phi;
        let br = BarrettReduction::<u128>::new(p as u128);
        let a = br.rem(a as _) as u64;
        let b = br.rem(b as _) as u64;
        if b == 1 {
            return Some(0);
        }
        if a == 0 {
            return if b == 0 { Some(1) } else { None };
        }
        if b == 0 {
            return None;
        }

        if let (Some(x), Some(y)) = (
            self.index_calculus(a, primes),
            self.index_calculus(b, primes),
        ) {
            let d = gcd_binary(x, phi);
            if y % d != 0 {
                return None;
            }
            let q = phi / d;
            Some(((y / d) as u128 * inv(x / d, q) as u128 % q as u128) as u64)
        } else {
            None
        }
    }
}

thread_local!(
    static IC: UnsafeCell<IndexCalculus> = UnsafeCell::new(IndexCalculus::new());
);

pub fn discrete_logarithm_prime_mod(a: u64, b: u64, p: u64) -> Option<u64> {
    IC.with(|ic| unsafe { &mut *ic.get() }.discrete_logarithm(a, b, p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::MultiplicativeOperation, algorithm::BabyStepGiantStep, num::mint_basic::DynMIntU64,
    };

    #[test]
    fn test_ic_small_prime() {
        let mut ic = IndexCalculus::new();
        let pl = PrimeList::new(30);
        for &p in pl.primes() {
            for a in 1..p {
                for b in 1..p {
                    let l = ic.discrete_logarithm(a, b, p);
                    if let Some(l) = l {
                        assert_eq!(b, pow(a, l, &BarrettReduction::<u128>::new(p as _)));
                    }
                    DynMIntU64::set_mod(p);
                    let res = if b == 1 {
                        Some(0)
                    } else {
                        BabyStepGiantStep::<MultiplicativeOperation<_>>::new(
                            p as usize + 1,
                            DynMIntU64::new(a),
                        )
                        .solve(DynMIntU64::new(b))
                        .map(|x| x as u64)
                    };
                    assert_eq!(res, l);
                }
            }
        }
    }

    #[test]
    fn test_ic_medium_prime() {
        let mut ic = IndexCalculus::new();
        let mut rng = Xorshift::default();
        for &p in [998_244_353, 1_000_000_007].iter() {
            for _ in 0..20 {
                let (a, b) = rng.gen((1..p, 1..p));
                let l = ic.discrete_logarithm(a, b, p);
                if let Some(l) = l {
                    assert_eq!(b, pow(a, l, &BarrettReduction::<u128>::new(p as _)));
                }
                DynMIntU64::set_mod(p);
                let res = if b == 1 {
                    Some(0)
                } else {
                    BabyStepGiantStep::<MultiplicativeOperation<_>>::new(
                        p as usize + 1,
                        DynMIntU64::new(a),
                    )
                    .solve(DynMIntU64::new(b))
                    .map(|x| x as u64)
                };
                assert_eq!(res, l);
            }
        }
    }

    #[test]
    fn test_ic_large_prime() {
        let mut ic = IndexCalculus::new();
        let mut rng = Xorshift::default();
        let p = 1_000_000_000_000 - 11;
        for _ in 0..20 {
            let (a, b) = rng.gen((1..p, 1..p));
            let l = ic.discrete_logarithm(a, b, p);
            if let Some(l) = l {
                assert_eq!(b, pow(a, l, &BarrettReduction::<u128>::new(p as _)));
            }
        }
    }
}
