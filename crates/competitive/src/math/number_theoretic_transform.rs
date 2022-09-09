use super::{
    montgomery::*, AssociatedValue, ConvolveSteps, MInt, MIntBase, MIntConvert, One, Zero,
};
use std::marker::PhantomData;

pub trait NttModulus:
    'static + Sized + MIntBase<Inner = u32> + MIntConvert<u32> + AssociatedValue<T = NttCache<Self>>
{
    fn primitive_root() -> MInt<Self>;
}

pub struct Convolve<M>(PhantomData<fn() -> M>);
pub type Convolve998244353 = Convolve<Modulo998244353>;
pub type MIntConvolve<M> = Convolve<(M, (Modulo2013265921, Modulo1811939329, Modulo2113929217))>;

macro_rules! impl_ntt_modulus {
    ($([$name:ident, $g:expr]),*) => {
        $(
            impl NttModulus for $name {
                fn primitive_root() -> MInt<Self> {
                    MInt::new_unchecked($g)
                }
            }
            crate::impl_assoc_value!($name, NttCache<$name>, NttCache::new());
        )*
    };
}
impl_ntt_modulus!(
    [Modulo998244353, 3],
    [Modulo2113929217, 5],
    [Modulo1811939329, 13],
    [Modulo2013265921, 31]
);
#[derive(Debug)]
pub struct NttCache<M>
where
    M: NttModulus,
{
    cache: Vec<MInt<M>>,
    icache: Vec<MInt<M>>,
}
impl<M> Clone for NttCache<M>
where
    M: NttModulus,
{
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            icache: self.icache.clone(),
        }
    }
}
impl<M> NttCache<M>
where
    M: NttModulus,
{
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
        let q: usize = M::mod_into() as usize - 1;
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
impl<M> NttCache<M>
where
    M: NttModulus,
{
    fn ntt(a: &mut [MInt<M>]) {
        M::modify(|cache| {
            let n = a.len();
            cache.ensure(n / 2);
            let mut v = n / 2;
            while v > 0 {
                for (a, wj) in a.chunks_exact_mut(v << 1).zip(&cache.cache) {
                    let (l, r) = a.split_at_mut(v);
                    for (x, y) in l.iter_mut().zip(r) {
                        let ajv = wj * *y;
                        *y = *x - ajv;
                        *x += ajv;
                    }
                }
                v >>= 1;
            }
        });
    }
    fn intt(a: &mut [MInt<M>]) {
        M::modify(|cache| {
            let n = a.len();
            cache.ensure(n / 2);
            let mut v = 1;
            while v < n {
                for (a, wj) in a.chunks_exact_mut(v << 1).zip(&cache.icache) {
                    let (l, r) = a.split_at_mut(v);
                    for (x, y) in l.iter_mut().zip(r) {
                        let ajv = *x - *y;
                        *x += *y;
                        *y = wj * ajv;
                    }
                }
                v <<= 1;
            }
        });
    }
}
impl<M> ConvolveSteps for Convolve<M>
where
    M: NttModulus,
{
    type T = Vec<MInt<M>>;
    type F = Vec<MInt<M>>;
    fn length(t: &Self::T) -> usize {
        t.len()
    }
    fn transform(mut t: Self::T, len: usize) -> Self::F {
        t.resize_with(len.max(2).next_power_of_two(), Zero::zero);
        NttCache::<M>::ntt(&mut t);
        t
    }
    fn inverse_transform(mut f: Self::F, len: usize) -> Self::T {
        NttCache::<M>::intt(&mut f);
        f.truncate(len);
        let inv = MInt::from(len.max(2).next_power_of_two() as u32).inv();
        for f in f.iter_mut() {
            *f *= inv;
        }
        f
    }
    fn multiply(f: &mut Self::F, g: &Self::F) {
        assert_eq!(f.len(), g.len());
        for (f, g) in f.iter_mut().zip(g.iter()) {
            *f *= *g;
        }
    }
}
type MVec<M> = Vec<MInt<M>>;
impl<M, N1, N2, N3> ConvolveSteps for Convolve<(M, (N1, N2, N3))>
where
    M: MIntConvert + MIntConvert<u32>,
    N1: NttModulus,
    N2: NttModulus,
    N3: NttModulus,
{
    type T = MVec<M>;
    type F = (MVec<N1>, MVec<N2>, MVec<N3>);
    fn length(t: &Self::T) -> usize {
        t.len()
    }
    fn transform(t: Self::T, len: usize) -> Self::F {
        let npot = len.max(2).next_power_of_two();
        let mut f = (
            MVec::<N1>::with_capacity(npot),
            MVec::<N2>::with_capacity(npot),
            MVec::<N3>::with_capacity(npot),
        );
        for t in t {
            f.0.push(<M as MIntConvert<u32>>::into(t.inner()).into());
            f.1.push(<M as MIntConvert<u32>>::into(t.inner()).into());
            f.2.push(<M as MIntConvert<u32>>::into(t.inner()).into());
        }
        f.0.resize_with(npot, Zero::zero);
        f.1.resize_with(npot, Zero::zero);
        f.2.resize_with(npot, Zero::zero);
        NttCache::<N1>::ntt(&mut f.0);
        NttCache::<N2>::ntt(&mut f.1);
        NttCache::<N3>::ntt(&mut f.2);
        f
    }
    fn inverse_transform(f: Self::F, len: usize) -> Self::T {
        let t1 = MInt::<N2>::new(N1::get_mod()).inv();
        let m1 = MInt::<M>::from(N1::get_mod());
        let m1_3 = MInt::<N3>::new(N1::get_mod());
        let t2 = (m1_3 * MInt::<N3>::new(N2::get_mod())).inv();
        let m2 = m1 * MInt::<M>::from(N2::get_mod());
        Convolve::<N1>::inverse_transform(f.0, len)
            .into_iter()
            .zip(Convolve::<N2>::inverse_transform(f.1, len))
            .zip(Convolve::<N3>::inverse_transform(f.2, len))
            .map(|((c1, c2), c3)| {
                let d1 = c1.inner();
                let d2 = ((c2 - MInt::<N2>::from(d1)) * t1).inner();
                let x = MInt::<N3>::new(d1) + MInt::<N3>::new(d2) * m1_3;
                let d3 = ((c3 - x) * t2).inner();
                MInt::<M>::from(d1) + MInt::<M>::from(d2) * m1 + MInt::<M>::from(d3) * m2
            })
            .collect()
    }
    fn multiply(f: &mut Self::F, g: &Self::F) {
        assert_eq!(f.0.len(), g.0.len());
        assert_eq!(f.1.len(), g.1.len());
        assert_eq!(f.2.len(), g.2.len());
        for (f, g) in f.0.iter_mut().zip(g.0.iter()) {
            *f *= *g;
        }
        for (f, g) in f.1.iter_mut().zip(g.1.iter()) {
            *f *= *g;
        }
        for (f, g) in f.2.iter_mut().zip(g.2.iter()) {
            *f *= *g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::{
        mint_basic::Modulo1000000009,
        montgomery::{MInt998244353, Modulo998244353},
    };
    use crate::tools::Xorshift;

    const N: usize = 3_000;

    #[test]
    fn test_ntt998244353() {
        let mut rng = Xorshift::time();
        let a: Vec<_> = rng
            .gen_iter(..MInt998244353::get_mod())
            .map(MInt998244353::new_unchecked)
            .take(N)
            .collect();
        let b: Vec<_> = rng
            .gen_iter(..MInt998244353::get_mod())
            .map(MInt998244353::new_unchecked)
            .take(N)
            .collect();
        let mut c = vec![MInt998244353::zero(); N * 2 - 1];
        for i in 0..N {
            for j in 0..N {
                c[i + j] += a[i] * b[j];
            }
        }
        let d = Convolve::<Modulo998244353>::convolve(a, b);
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
        let d = MIntConvolve::<Modulo1000000009>::convolve(a, b);
        assert_eq!(c, d);
    }

    // #[test]
    #[allow(dead_code)]
    fn find_proth() {
        use crate::math::{divisors, prime_factors_flatten};
        use crate::num::mint_basic::DynMIntU32;
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
                let f = prime_factors_flatten(p);
                if f.len() == 1 && f[0] == p {
                    DynMIntU32::set_mod(p as u32);
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
