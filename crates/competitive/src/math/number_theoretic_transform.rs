#[codesnip::skip]
use crate::{
    impl_assoc_value,
    num::{mint_basic, MInt, MIntBase, MIntConvert, One, Zero},
    tools::AssociatedValue,
};

pub struct NumberTheoreticTransform<M: MIntBase>(std::marker::PhantomData<fn() -> M>);
pub trait NttModulus:
    Sized + MIntBase + AssociatedValue<T = number_theoretic_transform_impls::NttCache<Self>>
{
    fn primitive_root() -> MInt<Self>;
}
pub mod number_theoretic_transform_impls {
    use super::*;
    use mint_basic::Modulo998244353;
    macro_rules! impl_ntt_modulus {
        ($([$name:ident, $g:expr]),*) => {
            $(
                impl NttModulus for $name {
                    fn primitive_root() -> MInt<Self> {
                        MInt::new_unchecked($g)
                    }
                }
                impl_assoc_value!($name, NttCache<$name>, NttCache::new());
            )*
        };
    }
    impl_ntt_modulus!(
        [Modulo998244353, 3],
        [Modulo2113929217, 5],
        [Modulo1811939329, 13],
        [Modulo2013265921, 31]
    );
    crate::define_basic_mint32!(
        [Modulo2113929217, 2_113_929_217, MInt2113929217], // 25
        [Modulo1811939329, 1_811_939_329, MInt1811939329], // 26
        [Modulo2013265921, 2_013_265_921, MInt2013265921]  // 27
    );
    #[derive(Debug)]
    pub struct NttCache<M: NttModulus> {
        cache: Vec<MInt<M>>,
        icache: Vec<MInt<M>>,
    }
    impl<M: NttModulus> Clone for NttCache<M> {
        fn clone(&self) -> Self {
            Self {
                cache: self.cache.clone(),
                icache: self.icache.clone(),
            }
        }
    }
    impl<M: NttModulus + MIntConvert<usize>> NttCache<M> {
        fn new() -> Self {
            Self {
                cache: Vec::new(),
                icache: Vec::new(),
            }
        }
        fn ensure(&mut self, n: usize) {
            assert_eq!(n.count_ones(), 1, "call with power of two but {}", n);
            let mut m = self.cache.len();
            assert!(
                m.count_ones() <= 1,
                "length might be power of two but {}",
                m
            );
            if m >= n {
                return;
            }
            let q: usize = M::mod_into() - 1;
            self.cache.reserve_exact(n - m);
            self.icache.reserve_exact(n - m);
            if self.cache.is_empty() {
                self.cache.push(MInt::one());
                self.icache.push(MInt::one());
                m += 1;
            }
            while m < n {
                let p = M::primitive_root().pow(q / (m * 4));
                let pinv = p.inv();
                for i in 0..m {
                    self.cache.push(self.cache[i] * p);
                    self.icache.push(self.icache[i] * pinv);
                }
                m <<= 1;
            }
            assert_eq!(self.cache.len(), n);
        }
    }
    impl<M: NttModulus + MIntConvert<usize>> NumberTheoreticTransform<M> {
        fn convolve_inner(mut a: Vec<MInt<M>>, mut b: Vec<MInt<M>>) -> Vec<MInt<M>> {
            Self::ntt(&mut a);
            Self::ntt(&mut b);
            for (a, b) in a.iter_mut().zip(b.iter_mut()) {
                *a *= *b;
            }
            Self::intt(&mut a);
            a
        }
        #[allow(clippy::needless_range_loop)]
        fn ntt(a: &mut [MInt<M>]) {
            M::modify(|cache| {
                let n = a.len();
                cache.ensure(n / 2);
                let mut u = 1;
                let mut v = n / 2;
                for i in (1..=n.trailing_zeros()).rev() {
                    for jh in 0..u {
                        let wj = cache.cache[jh];
                        for j in jh << i..(jh << i) + v {
                            let ajv = wj * a[j + v];
                            a[j + v] = a[j] - ajv;
                            a[j] += ajv;
                        }
                    }
                    u <<= 1;
                    v >>= 1;
                }
            });
        }
        #[allow(clippy::needless_range_loop)]
        fn intt(a: &mut [MInt<M>]) {
            M::modify(|cache| {
                let n = a.len();
                cache.ensure(n / 2);
                let mut u = n / 2;
                let mut v = 1;
                for i in 1..=n.trailing_zeros() {
                    for jh in 0..u {
                        let wj = cache.icache[jh];
                        for j in jh << i..(jh << i) + v {
                            let ajv = a[j] - a[j + v];
                            a[j] += a[j + v];
                            a[j + v] = wj * ajv;
                        }
                    }
                    u >>= 1;
                    v <<= 1;
                }
            });
        }
        pub fn convert<T: Into<MInt<M>>, I: IntoIterator<Item = T>>(iter: I) -> Vec<MInt<M>> {
            iter.into_iter().map(|x| x.into()).collect()
        }
        pub fn convolve(mut a: Vec<MInt<M>>, mut b: Vec<MInt<M>>) -> Vec<MInt<M>> {
            let m = a.len() + b.len() - 1;
            let n = m.max(2).next_power_of_two();
            a.resize_with(n, Zero::zero);
            b.resize_with(n, Zero::zero);
            let mut c = Self::convolve_inner(a, b);
            c.truncate(m);
            let ninv = MInt::from(n).inv();
            for c in c.iter_mut() {
                *c *= ninv;
            }
            c
        }
        pub fn convolve_ref<T: Clone + Into<MInt<M>>>(a: &[T], b: &[T]) -> Vec<MInt<M>> {
            let m = a.len() + b.len() - 1;
            let n = m.max(2).next_power_of_two();
            let a = a
                .iter()
                .map(|a| a.clone().into())
                .chain(std::iter::repeat_with(Zero::zero))
                .take(n)
                .collect();
            let b = b
                .iter()
                .map(|b| b.clone().into())
                .chain(std::iter::repeat_with(Zero::zero))
                .take(n)
                .collect();
            let mut c = Self::convolve_inner(a, b);
            c.truncate(m);
            let ninv = MInt::from(n).inv();
            for c in c.iter_mut() {
                *c *= ninv;
            }
            c
        }
        pub fn convolve_it<T, I>(iter1: I, iter2: I) -> Vec<MInt<M>>
        where
            T: Into<MInt<M>>,
            I: IntoIterator<Item = T>,
        {
            Self::convolve(Self::convert(iter1), Self::convert(iter2))
        }
    }
}
pub type Ntt998244353 = NumberTheoreticTransform<mint_basic::Modulo998244353>;

/// max(a.len(), b.len()) * max(a) * max(b) < 3.64 * 10^18
pub fn convolve2<T>(a: &[T], b: &[T]) -> Vec<u64>
where
    T: Clone
        + Into<number_theoretic_transform_impls::MInt2013265921>
        + Into<number_theoretic_transform_impls::MInt1811939329>,
{
    type M1 = number_theoretic_transform_impls::Modulo2013265921;
    type M2 = number_theoretic_transform_impls::Modulo1811939329;
    let c1 = NumberTheoreticTransform::<M1>::convolve_ref(&a, &b);
    let c2 = NumberTheoreticTransform::<M2>::convolve_ref(&a, &b);
    let p1: u64 = M1::mod_into();
    let p1_inv = MInt::<M2>::new(M1::get_mod()).inv();
    c1.into_iter()
        .zip(c2.into_iter())
        .map(|(c1, c2)| {
            c1.inner() as u64 + p1 * ((c2 - MInt::<M2>::from(c1.inner())) * p1_inv).inner() as u64
        })
        .collect()
}

/// max(a.len(), b.len()) * max(a) * max(b) < 1.81 * 10^27
pub fn convolve_mint<M>(a: &[MInt<M>], b: &[MInt<M>]) -> Vec<MInt<M>>
where
    M: MIntConvert<u32>,
{
    type M1 = number_theoretic_transform_impls::Modulo2013265921;
    type M2 = number_theoretic_transform_impls::Modulo1811939329;
    type M3 = number_theoretic_transform_impls::Modulo2113929217;
    let cvt = |a: &MInt<M>| -> u32 { a.clone().into() };
    let c1 = NumberTheoreticTransform::<M1>::convolve_it(a.iter().map(cvt), b.iter().map(cvt));
    let c2 = NumberTheoreticTransform::<M2>::convolve_it(a.iter().map(cvt), b.iter().map(cvt));
    let c3 = NumberTheoreticTransform::<M3>::convolve_it(a.iter().map(cvt), b.iter().map(cvt));
    let t1 = MInt::<M2>::new(M1::get_mod()).inv();
    let m1 = MInt::<M>::from(M1::get_mod());
    let m13 = MInt::<M3>::new(M1::get_mod());
    let t2 = (MInt::<M3>::new(M1::get_mod()) * MInt::<M3>::new(M2::get_mod())).inv();
    let m2 = m1 * MInt::<M>::from(M2::get_mod());
    c1.into_iter()
        .zip(c2.into_iter())
        .zip(c3.into_iter())
        .map(|((c1, c2), c3)| {
            let x = MInt::<M3>::new(c1.inner())
                + MInt::<M3>::new(((c2 - MInt::<M2>::from(c1.inner())) * t1).inner()) * m13;
            MInt::<M>::from(c1.inner())
                + MInt::<M>::from(((c2 - MInt::<M2>::from(c1.inner())) * t1).inner()) * m1
                + MInt::<M>::from(((c3 - MInt::<M3>::from(x.inner())) * t2).inner()) * m2
        })
        .collect()
}

/// max(a.len(), b.len()) * max(a) * max(b) < 1.81 * 10^27
pub fn convolve3<T>(mut a: Vec<T>, mut b: Vec<T>) -> Vec<u128>
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::mint_basic::{MInt998244353, Modulo1000000009, Modulo998244353};
    use crate::{rand, tools::Xorshift};

    const N: usize = 3_000;

    #[test]
    fn test_ntt998244353() {
        let mut rng = Xorshift::time();
        pub type Ntt = NumberTheoreticTransform<Modulo998244353>;
        let a = Ntt::convert(rng.gen_iter(..MInt998244353::get_mod()).take(N));
        let b = Ntt::convert(rng.gen_iter(..MInt998244353::get_mod()).take(N));
        let mut c = vec![MInt998244353::zero(); N * 2 - 1];
        for i in 0..N {
            for j in 0..N {
                c[i + j] += a[i] * b[j];
            }
        }
        let d = Ntt::convolve(a, b);
        assert_eq!(c, d);
    }

    #[test]
    fn test_convolve2() {
        let mut rng = Xorshift::time();
        let m: u32 = ((std::u64::MAX / N as u64 / 100) as f64).sqrt() as _;
        rand!(rng, a: [..m; N], b: [..m; N]);
        let mut c = vec![0u64; N * 2 - 1];
        for i in 0..N {
            for j in 0..N {
                c[i + j] += a[i] as u64 * b[j] as u64;
            }
        }
        let d = convolve2(&a, &b);
        assert_eq!(c, d);
    }

    #[test]
    fn test_convolve3() {
        type M = MInt<Modulo1000000009>;
        let mut rng = Xorshift::time();
        let a: Vec<_> = rng
            .gen_iter(..M::get_mod())
            .map(M::new_unchecked)
            .take(N)
            .collect();
        let b: Vec<_> = rng
            .gen_iter(..M::get_mod())
            .map(M::new_unchecked)
            .take(N)
            .collect();
        let mut c = vec![M::zero(); N * 2 - 1];
        for i in 0..N {
            for j in 0..N {
                c[i + j] += a[i] * b[j];
            }
        }
        let d = convolve_mint::<Modulo1000000009>(&a, &b);
        assert_eq!(c, d);
    }

    #[test]
    fn test_convolve3_128() {
        let mut rng = Xorshift::time();
        const A: u64 = std::u32::MAX as _;
        rand!(rng, a: [..=A; N], b: [..=A; N]);
        let mut c = vec![0u128; N * 2 - 1];
        for i in 0..N {
            for j in 0..N {
                c[i + j] += (a[i] * b[j]) as u128;
            }
        }
        let d = convolve3(a, b);
        assert_eq!(c, d);
    }

    // #[test]
    #[allow(dead_code)]
    fn find_proth() {
        use crate::math::{divisors, prime_factors_rho};
        use crate::num::mint_basic::{DynMIntU32, DynModuloU32};
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
                    DynModuloU32::set_mod(p as u32);
                    for g in (3..).step_by(2) {
                        let g = DynMIntU32::new(g);
                        if divisors(p as u64 - 1)
                            .into_iter()
                            .filter(|&d| d != p as u64 - 1)
                            .all(|d| g.pow(d as usize) != DynMIntU32::one())
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
}
