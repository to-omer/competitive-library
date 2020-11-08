#[codesnip::skip]
use crate::num::{mint_base, MInt, MIntBase, MIntConvert, One, Zero};

pub struct NumberTheoreticTransform<M: MIntBase>(std::marker::PhantomData<fn() -> M>);
pub trait NTTModulus: MIntBase {
    fn primitive_root() -> usize;
}
mod number_theoretic_transform_impls {
    use super::*;
    use mint_base::Modulo998244353;
    macro_rules! impl_ntt_modulus {
        ($([$name:ident, $t:ty, $g:expr]),*) => {
            $(impl NTTModulus for $name {
                fn primitive_root() -> $t {
                    $g
                }
            })*
        };
    }
    impl_ntt_modulus!(
        [Modulo998244353, usize, 3],
        [Modulo2113929217, usize, 5],
        [Modulo1811939329, usize, 13],
        [Modulo2013265921, usize, 31]
    );
    crate::define_basic_mint32!(
        [Modulo2113929217, 2_113_929_217, MInt2113929217], // 25
        [Modulo1811939329, 1_811_939_329, MInt1811939329], // 26
        [Modulo2013265921, 2_013_265_921, MInt2013265921]  // 27
    );
}
pub type NTT998244353 = NumberTheoreticTransform<mint_base::Modulo998244353>;
impl<M: NTTModulus + MIntConvert<usize>> NumberTheoreticTransform<M> {
    pub fn convert<T: Into<MInt<M>>, I: IntoIterator<Item = T>>(iter: I) -> Vec<MInt<M>> {
        iter.into_iter().map(|x| x.into()).collect()
    }
    pub fn ntt(mut f: Vec<MInt<M>>, inv: bool) -> Vec<MInt<M>> {
        let n = f.len();
        debug_assert!(n.count_ones() == 1);
        let q = M::mod_into() - 1;
        debug_assert!(n.trailing_zeros() <= q.trailing_zeros());
        let mask = n - 1;
        let omega = MInt::from(M::primitive_root()).pow(q / n);
        let omega = if inv { omega.inv() } else { omega };
        let mut g = vec![MInt::<M>::zero(); n];
        let mut i = n / 2;
        while i >= 1 {
            let t = omega.pow(i);
            let mut w = MInt::<M>::one();
            for j in (0..n).step_by(i) {
                for k in 0..i {
                    g[j + k] = f[((j * 2) & mask) + k] + w * f[((j * 2 + i) & mask) + k];
                }
                w *= t;
            }
            i /= 2;
            std::mem::swap(&mut f, &mut g);
        }
        if inv {
            let u = MInt::from(n).inv();
            for a in f.iter_mut() {
                *a *= u;
            }
        }
        f
    }
    pub fn convolve(mut a: Vec<MInt<M>>, mut b: Vec<MInt<M>>) -> Vec<MInt<M>> {
        let m = a.len() + b.len() - 1;
        let n = m.next_power_of_two();
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
    use crate::num::mint_base::MInt998244353;
    use crate::tools::Xorshift;
    const N: usize = 3_000;
    let mut rand = Xorshift::time();
    pub type NTT = NumberTheoreticTransform<mint_base::Modulo998244353>;
    let a: Vec<_> = NTT::convert((0..N).map(|_| rand.rand(MInt998244353::get_mod() as u64)));
    let b: Vec<_> = NTT::convert((0..N).map(|_| rand.rand(MInt998244353::get_mod() as u64)));
    let mut c = vec![MInt998244353::zero(); N * 2 - 1];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += a[i] * b[j];
        }
    }
    let d = NTT::convolve(a, b);
    assert_eq!(c, d);
}

/// max(a.len(), b.len()) * max(a) * max(b) < 3.64 * 10^18
pub fn convolve2<T>(mut a: Vec<T>, mut b: Vec<T>) -> Vec<u64>
where
    T: Into<number_theoretic_transform_impls::MInt2013265921>
        + Into<number_theoretic_transform_impls::MInt1811939329>
        + Clone
        + Zero,
{
    let m = a.len() + b.len() - 1;
    let n = m.next_power_of_two();
    a.resize_with(n, Zero::zero);
    b.resize_with(n, Zero::zero);
    type M1 = number_theoretic_transform_impls::Modulo2013265921;
    type M2 = number_theoretic_transform_impls::Modulo1811939329;
    let c1 = NumberTheoreticTransform::<M1>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c2 = NumberTheoreticTransform::<M2>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let p1: u64 = M1::mod_into();
    let p1_inv = MInt::<M2>::new(M1::get_mod()).inv();
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
    let a: Vec<_> = (0..N).map(|_| rand.rand(m) as u32).collect();
    let b: Vec<_> = (0..N).map(|_| rand.rand(m) as u32).collect();
    let mut c = vec![0u64; N * 2 - 1];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += a[i] as u64 * b[j] as u64;
        }
    }
    let d = convolve2(a, b);
    assert_eq!(c, d);
}

/// max(a.len(), b.len()) * max(a) * max(b) < 1.81 * 10^27
pub fn convolve3<M: MIntConvert<u32>, T>(mut a: Vec<T>, mut b: Vec<T>) -> Vec<MInt<M>>
where
    T: Into<number_theoretic_transform_impls::MInt2013265921>
        + Into<number_theoretic_transform_impls::MInt1811939329>
        + Into<number_theoretic_transform_impls::MInt2113929217>
        + Clone
        + Zero,
{
    let m = a.len() + b.len() - 1;
    let n = m.next_power_of_two();
    a.resize_with(n, Zero::zero);
    b.resize_with(n, Zero::zero);
    type M1 = number_theoretic_transform_impls::Modulo2013265921;
    type M2 = number_theoretic_transform_impls::Modulo1811939329;
    type M3 = number_theoretic_transform_impls::Modulo2113929217;
    let c1 = NumberTheoreticTransform::<M1>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c2 = NumberTheoreticTransform::<M2>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c3 = NumberTheoreticTransform::<M3>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let t1 = MInt::<M2>::new(M1::get_mod()).inv();
    let m1 = MInt::<M>::from(M1::get_mod());
    let m13 = MInt::<M3>::new(M1::get_mod());
    let t2 = (MInt::<M3>::new(M1::get_mod()) * MInt::<M3>::new(M2::get_mod())).inv();
    let m2 = m1 * MInt::<M>::from(M2::get_mod());
    c1.into_iter()
        .zip(c2.into_iter())
        .zip(c3.into_iter())
        .take(m)
        .map(|((c1, c2), c3)| {
            let x = MInt::<M3>::new(c1.inner())
                + MInt::<M3>::new(((c2 - MInt::<M2>::from(c1.inner())) * t1).inner()) * m13;
            MInt::<M>::from(c1.inner())
                + MInt::<M>::from(((c2 - MInt::<M2>::from(c1.inner())) * t1).inner()) * m1
                + MInt::<M>::from(((c3 - MInt::<M3>::from(x.inner())) * t2).inner()) * m2
        })
        .collect()
}

#[test]
fn test_convolve3() {
    use crate::tools::Xorshift;
    const N: usize = 3_000;
    type M = MInt<mint_base::Modulo1000000009>;
    let mut rand = Xorshift::time();
    let a: Vec<_> = (0..N)
        .map(|_| rand.rand(std::u32::MAX as u64) as u32)
        .collect();
    let b: Vec<_> = (0..N)
        .map(|_| rand.rand(std::u32::MAX as u64) as u32)
        .collect();
    let mut c = vec![M::zero(); N * 2 - 1];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += M::from(a[i] as u64 * b[j] as u64);
        }
    }
    let d = convolve3::<mint_base::Modulo1000000009, _>(a, b);
    assert_eq!(c, d);
}

/// max(a.len(), b.len()) * max(a) * max(b) < 1.81 * 10^27
pub fn convolve3_128<T>(mut a: Vec<T>, mut b: Vec<T>) -> Vec<u128>
where
    T: Into<MInt<number_theoretic_transform_impls::Modulo2013265921>>
        + Into<MInt<number_theoretic_transform_impls::Modulo1811939329>>
        + Into<MInt<number_theoretic_transform_impls::Modulo2113929217>>
        + Clone
        + Zero,
{
    let m = a.len() + b.len() - 1;
    let n = m.next_power_of_two();
    a.resize_with(n, Zero::zero);
    b.resize_with(n, Zero::zero);
    type M1 = number_theoretic_transform_impls::Modulo2013265921;
    type M2 = number_theoretic_transform_impls::Modulo1811939329;
    type M3 = number_theoretic_transform_impls::Modulo2113929217;
    let c1 = NumberTheoreticTransform::<M1>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c2 = NumberTheoreticTransform::<M2>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let c3 = NumberTheoreticTransform::<M3>::convolve_it(a.iter().cloned(), b.iter().cloned());
    let p1 = M1::get_mod();
    let t1 = MInt::<M2>::new(p1).inv();
    let m1 = p1 as u64;
    let p2 = M2::get_mod();
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
#[codesnip::skip]
#[allow(dead_code)]
fn find_proth() {
    use crate::math::{divisors, prime_factors_rho};
    static mut MOD: u32 = 2;
    crate::define_basic_mintbase!(
        DM,
        unsafe { MOD },
        u32,
        u64,
        [u32, u64, u128, usize],
        [i32, i64, i128, isize]
    );
    pub type DMInt = MInt<DM>;
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
