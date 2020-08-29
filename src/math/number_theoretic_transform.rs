use crate::num::{modulus, MInt, Modulus, One, Zero};

#[cargo_snippet::snippet("NumberTheoreticTransform")]
pub struct NumberTheoreticTransform<M: Modulus>(std::marker::PhantomData<fn() -> M>);
#[cargo_snippet::snippet("NumberTheoreticTransform")]
pub trait NTTModulus: Modulus {
    fn primitive_root() -> u32;
}
#[cargo_snippet::snippet("NumberTheoreticTransform")]
mod number_theoretic_transform_impls {
    use super::*;
    impl NTTModulus for modulus::Modulo998244353 {
        fn primitive_root() -> u32 {
            const G: u32 = 3;
            G
        }
    }
    macro_rules! make_ntt_modulus {
        ($t:ident, $m:expr, $g:expr) => {
            pub struct $t {}
            impl Modulus for $t {
                #[inline]
                fn get_modulus() -> u32 {
                    const MODULUS: u32 = $m;
                    MODULUS
                }
            }
            impl NTTModulus for $t {
                #[inline]
                fn primitive_root() -> u32 {
                    const G: u32 = $g;
                    G
                }
            }
        };
    }
    make_ntt_modulus!(Modulo2113929217, 2_113_929_217, 5); // 25
    make_ntt_modulus!(Modulo1811939329, 1_811_939_329, 13); // 26
    make_ntt_modulus!(Modulo2013265921, 2_013_265_921, 31); // 27
}
#[cargo_snippet::snippet("NumberTheoreticTransform")]
impl<M: NTTModulus> NumberTheoreticTransform<M> {
    pub fn convert<T: Into<MInt<M>>, I: IntoIterator<Item = T>>(iter: I) -> Vec<MInt<M>> {
        iter.into_iter().map(|x| x.into()).collect()
    }
    pub fn ntt(mut f: Vec<MInt<M>>, inv: bool) -> Vec<MInt<M>> {
        let n = f.len();
        debug_assert!(n.count_ones() == 1);
        let q = MInt::<M>::get_mod() as usize - 1;
        debug_assert!(n.trailing_zeros() <= q.trailing_zeros());
        let mask = n - 1;
        let omega = MInt::<M>::new_unchecked(M::primitive_root()).pow(q / n);
        let omega = if inv { omega.inv() } else { omega };
        let mut g = vec![MInt::<M>::zero(); n];
        let mut i = n / 2;
        while i >= 1 {
            let t = omega.pow(i);
            let mut w = MInt::<M>::one();
            for j in (0..n).step_by(i) {
                for k in 0..i {
                    g[j + k] = f[(j * 2 & mask) + k] + w * f[(j * 2 + i & mask) + k];
                }
                w *= t;
            }
            i /= 2;
            std::mem::swap(&mut f, &mut g);
        }
        if inv {
            let u = MInt::<M>::new(n as u32).inv();
            for a in f.iter_mut() {
                *a *= u;
            }
        }
        f
    }
    pub fn convolve(mut a: Vec<MInt<M>>, mut b: Vec<MInt<M>>) -> Vec<MInt<M>> {
        let m = a.len() + b.len() - 1;
        let n = 1usize << format!("{:b}", m).len();
        a.resize_with(n, MInt::<M>::zero);
        b.resize_with(n, MInt::<M>::zero);
        let a = Self::ntt(a, false);
        let b = Self::ntt(b, false);
        let c: Vec<_> = a
            .into_iter()
            .zip(b.into_iter())
            .map(|(a, b)| a * b)
            .collect();
        let mut c = Self::ntt(c, true);
        c.truncate(m);
        c
    }
    pub fn convolve_it<T: Into<MInt<M>>, I: IntoIterator<Item = T>>(
        iter1: I,
        iter2: I,
    ) -> Vec<MInt<M>> {
        Self::convolve(Self::convert(iter1), Self::convert(iter2))
    }
}

#[test]
fn test_ntt998244353() {
    use crate::tools::Xorshift;
    const N: usize = 3_000;
    let mut rand = Xorshift::time();
    type M = MInt<modulus::Modulo998244353>;
    pub type NTT = NumberTheoreticTransform<modulus::Modulo998244353>;
    let a: Vec<_> = NTT::convert((0..N).map(|_| rand.rand(M::get_mod() as u64)));
    let b: Vec<_> = NTT::convert((0..N).map(|_| rand.rand(M::get_mod() as u64)));
    let mut c = vec![M::zero(); N * 2 - 1];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += a[i] * b[j];
        }
    }
    let d = NTT::convolve(a, b);
    assert_eq!(c, d);
}

#[cargo_snippet::snippet("NumberTheoreticTransform")]
/// max(a.len(), b.len()) * max(a) * max(b) < 3.64 * 10^18
pub fn convolve2(mut a: Vec<u64>, mut b: Vec<u64>) -> Vec<u64> {
    let m = a.len() + b.len() - 1;
    let n = 1usize << format!("{:b}", m).len();
    a.resize_with(n, Default::default);
    b.resize_with(n, Default::default);
    type M1 = number_theoretic_transform_impls::Modulo2013265921;
    type M2 = number_theoretic_transform_impls::Modulo1811939329;
    let c1 = NumberTheoreticTransform::<M1>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c2 = NumberTheoreticTransform::<M2>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let p1 = M1::get_modulus() as u64;
    let p1_inv = MInt::<M2>::new(M1::get_modulus()).inv();
    c1.into_iter()
        .zip(c2.into_iter())
        .take(m)
        .map(|(c1, c2)| {
            c1.inner() as u64 + p1 * ((c2 - MInt::<M2>::from(c1.inner())) * p1_inv).inner() as u64
        })
        .collect()
}

#[test]
fn test_convolve2() {
    use crate::tools::Xorshift;
    const N: usize = 3_000;
    let mut rand = Xorshift::time();
    let m: u64 = ((std::u64::MAX / N as u64 / 100) as f64).sqrt() as u64;
    let a: Vec<_> = (0..N).map(|_| rand.rand(m)).collect();
    let b: Vec<_> = (0..N).map(|_| rand.rand(m)).collect();
    let mut c = vec![0u64; N * 2 - 1];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += a[i] * b[j];
        }
    }
    let d = convolve2(a, b);
    assert_eq!(c, d);
}

#[cargo_snippet::snippet("NumberTheoreticTransform")]
/// max(a.len(), b.len()) * max(a) * max(b) < 1.81 * 10^27
pub fn convolve3<M: Modulus>(mut a: Vec<u64>, mut b: Vec<u64>) -> Vec<MInt<M>> {
    let m = a.len() + b.len() - 1;
    let n = 1usize << format!("{:b}", m).len();
    a.resize_with(n, Default::default);
    b.resize_with(n, Default::default);
    type M1 = number_theoretic_transform_impls::Modulo2013265921;
    type M2 = number_theoretic_transform_impls::Modulo1811939329;
    type M3 = number_theoretic_transform_impls::Modulo2113929217;
    let c1 = NumberTheoreticTransform::<M1>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c2 = NumberTheoreticTransform::<M2>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c3 = NumberTheoreticTransform::<M3>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let t1 = MInt::<M2>::new(M1::get_modulus()).inv();
    let m1 = MInt::<M>::new(M1::get_modulus());
    let m13 = MInt::<M3>::new(M1::get_modulus());
    let t2 = (MInt::<M3>::new(M1::get_modulus()) * MInt::<M3>::new(M2::get_modulus())).inv();
    let m2 = m1 * MInt::<M>::new(M2::get_modulus());
    c1.into_iter()
        .zip(c2.into_iter())
        .zip(c3.into_iter())
        .take(m)
        .map(|((c1, c2), c3)| {
            let x = MInt::<M3>::new(c1.inner())
                + MInt::<M3>::new(((c2 - MInt::<M2>::from(c1.inner())) * t1).inner()) * m13;
            MInt::<M>::new(c1.inner())
                + MInt::<M>::new(((c2 - MInt::<M2>::from(c1.inner())) * t1).inner()) * m1
                + MInt::<M>::new(((c3 - MInt::<M3>::from(x.inner())) * t2).inner()) * m2
        })
        .collect()
}

#[test]
fn test_convolve3() {
    use crate::tools::Xorshift;
    const N: usize = 3_000;
    type M = MInt<modulus::Modulo1000000009>;
    let mut rand = Xorshift::time();
    let a: Vec<_> = (0..N).map(|_| rand.rand(std::u32::MAX as u64)).collect();
    let b: Vec<_> = (0..N).map(|_| rand.rand(std::u32::MAX as u64)).collect();
    let mut c = vec![M::zero(); N * 2 - 1];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += M::from(a[i] * b[j]);
        }
    }
    let d = convolve3::<modulus::Modulo1000000009>(a, b);
    assert_eq!(c, d);
}

#[cargo_snippet::snippet("NumberTheoreticTransform")]
/// max(a.len(), b.len()) * max(a) * max(b) < 1.81 * 10^27
pub fn convolve3_128(mut a: Vec<u64>, mut b: Vec<u64>) -> Vec<u128> {
    let m = a.len() + b.len() - 1;
    let n = 1usize << format!("{:b}", m).len();
    a.resize_with(n, Default::default);
    b.resize_with(n, Default::default);
    type M1 = number_theoretic_transform_impls::Modulo2013265921;
    type M2 = number_theoretic_transform_impls::Modulo1811939329;
    type M3 = number_theoretic_transform_impls::Modulo2113929217;
    let c1 = NumberTheoreticTransform::<M1>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c2 = NumberTheoreticTransform::<M2>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c3 = NumberTheoreticTransform::<M3>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let p1 = M1::get_modulus();
    let t1 = MInt::<M2>::new(p1).inv();
    let m1 = p1 as u64;
    let p2 = M2::get_modulus();
    let t2 = (MInt::<M3>::new(p1) * MInt::<M3>::new(p2)).inv();
    let m2 = m1 as u128 * p2 as u128;
    c1.into_iter()
        .zip(c2.into_iter())
        .zip(c3.into_iter())
        .take(m)
        .map(|((c1, c2), c3)| {
            let x =
                c1.inner() as u64 + ((c2 - MInt::<M2>::from(c1.inner())) * t1).inner() as u64 * m1;
            x as u128 + ((c3 - MInt::<M3>::from(x)) * t2).inner() as u128 * m2
        })
        .collect()
}

#[test]
fn test_convolve3_128() {
    use crate::tools::Xorshift;
    const N: usize = 3_000;
    let mut rand = Xorshift::time();
    let a: Vec<_> = (0..N).map(|_| rand.rand(std::u32::MAX as u64)).collect();
    let b: Vec<_> = (0..N).map(|_| rand.rand(std::u32::MAX as u64)).collect();
    let mut c = vec![0u128; N * 2 - 1];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += (a[i] * b[j]) as u128;
        }
    }
    let d = convolve3_128(a, b);
    assert_eq!(c, d);
}

// #[test]
#[allow(dead_code)]
fn find_proth() {
    use crate::math::{divisors, prime_factors_rho};
    struct DM {}
    static mut MOD: u32 = 2;
    impl Modulus for DM {
        #[inline]
        fn get_modulus() -> u32 {
            unsafe { MOD }
        }
    }
    // p = a * 2^b + 1 (b >= 1, a < 2^b)
    for b in 22..32 {
        for a in (1..1u64 << b).step_by(2) {
            let p = a * (1u64 << b) + 1;
            if 1 << 31 < p {
                break;
            }
            if p < 1 << 29 {
                continue;
            }
            let f = prime_factors_rho(p);
            if f.len() == 1 && f[0] == p {
                unsafe { MOD = p as u32 };
                for g in (3..).step_by(2) {
                    let g = MInt::<DM>::new(g);
                    if divisors(p as usize - 1)
                        .into_iter()
                        .filter(|&d| d != p as usize - 1)
                        .all(|d| g.pow(d) != MInt::<DM>::one())
                    {
                        println!("(p,a,b,g) = {:?}", (p, a, b, g));
                        break;
                    }
                }
            }
        }
    }
    // (p,a,b,g) = (666894337, 159, 22, 5)
    // (p,a,b,g) = (683671553, 163, 22, 3)
    // (p,a,b,g) = (918552577, 219, 22, 5)
    // (p,a,b,g) = (935329793, 223, 22, 3)
    // (p,a,b,g) = (943718401, 225, 22, 7)
    // (p,a,b,g) = (985661441, 235, 22, 3)
    // (p,a,b,g) = (1161822209, 277, 22, 3)
    // (p,a,b,g) = (1212153857, 289, 22, 3)
    // (p,a,b,g) = (1321205761, 315, 22, 11)
    // (p,a,b,g) = (1438646273, 343, 22, 3)
    // (p,a,b,g) = (1572864001, 375, 22, 13)
    // (p,a,b,g) = (1790967809, 427, 22, 13)
    // (p,a,b,g) = (1866465281, 445, 22, 3)
    // (p,a,b,g) = (2025848833, 483, 22, 11)
    // (p,a,b,g) = (595591169, 71, 23, 3)
    // (p,a,b,g) = (645922817, 77, 23, 3)
    // (p,a,b,g) = (880803841, 105, 23, 37)
    // (p,a,b,g) = (897581057, 107, 23, 3)
    // (p,a,b,g) = (998244353, 119, 23, 3)
    // (p,a,b,g) = (1300234241, 155, 23, 3)
    // (p,a,b,g) = (1484783617, 177, 23, 5)
    // (p,a,b,g) = (2088763393, 249, 23, 5)
    // (p,a,b,g) = (754974721, 45, 24, 11)
    // (p,a,b,g) = (1224736769, 73, 24, 3)
    // (p,a,b,g) = (2130706433, 127, 24, 3)
    // (p,a,b,g) = (1107296257, 33, 25, 31)
    // (p,a,b,g) = (1711276033, 51, 25, 29)
    // (p,a,b,g) = (2113929217, 63, 25, 5)
    // (p,a,b,g) = (1811939329, 27, 26, 13)
    // (p,a,b,g) = (2013265921, 15, 27, 31)
}
