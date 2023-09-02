use super::{montgomery::*, ConvolveSteps, MInt, MIntBase, MIntConvert, One, Zero};
use std::marker::PhantomData;

pub struct Convolve<M>(PhantomData<fn() -> M>);
pub type Convolve998244353 = Convolve<Modulo998244353>;
pub type MIntConvolve<M> = Convolve<(M, (Modulo2013265921, Modulo1811939329, Modulo2113929217))>;

macro_rules! impl_ntt_modulus {
    ($([$name:ident, $g:expr]),*) => {
        $(
            impl Montgomery32NttModulus for $name {}
        )*
    };
}
impl_ntt_modulus!(
    [Modulo998244353, 3],
    [Modulo2113929217, 5],
    [Modulo1811939329, 13],
    [Modulo2013265921, 31]
);

const fn reduce(z: u64, p: u32, r: u32) -> u32 {
    let mut z = ((z + r.wrapping_mul(z as u32) as u64 * p as u64) >> 32) as u32;
    if z >= p {
        z -= p;
    }
    z
}
const fn mod_mul(x: u32, y: u32, p: u32, r: u32) -> u32 {
    reduce(x as u64 * y as u64, p, r)
}
const fn mod_pow(mut x: u32, mut y: u32, p: u32, r: u32, mut z: u32) -> u32 {
    while y > 0 {
        if y & 1 == 1 {
            z = mod_mul(z, x, p, r);
        }
        x = mod_mul(x, x, p, r);
        y >>= 1;
    }
    z
}

pub trait Montgomery32NttModulus: Sized + MontgomeryReduction32 {
    const PRIMITIVE_ROOT: u32 = {
        let mut g = 3u32;
        loop {
            let mut ok = true;
            let mut d = 1u32;
            while d * d < Self::MOD {
                if (Self::MOD - 1) % d == 0 {
                    let ds = [d, (Self::MOD - 1) / d];
                    let mut i = 0;
                    while i < 2 {
                        ok &= ds[i] == Self::MOD - 1
                            || mod_pow(
                                reduce(g as u64 * Self::N2 as u64, Self::MOD, Self::R),
                                ds[i],
                                Self::MOD,
                                Self::R,
                                Self::N1,
                            ) != Self::N1;
                        i += 1;
                    }
                }
                d += 1;
            }
            if ok {
                break;
            }
            g += 2;
        }
        g
    };
    const RANK: u32 = (Self::MOD - 1).trailing_zeros();
    const INFO: NttInfo = NttInfo::new::<Self>();
}

pub struct NttInfo {
    root: [u32; 32],
    inv_root: [u32; 32],
    rate2: [u32; 32],
    inv_rate2: [u32; 32],
    rate3: [u32; 32],
    inv_rate3: [u32; 32],
}
impl NttInfo {
    const fn new<M>() -> Self
    where
        M: Montgomery32NttModulus,
    {
        let mut root = [0; 32];
        let mut inv_root = [0; 32];
        let mut rate2 = [0; 32];
        let mut inv_rate2 = [0; 32];
        let mut rate3 = [0; 32];
        let mut inv_rate3 = [0; 32];
        let rank = M::RANK as usize;

        let g = reduce(M::PRIMITIVE_ROOT as u64 * M::N2 as u64, M::MOD, M::R);
        root[rank] = mod_pow(g, (M::MOD - 1) >> rank, M::MOD, M::R, M::N1);
        inv_root[rank] = mod_pow(root[rank], M::MOD - 2, M::MOD, M::R, M::N1);
        let mut i = rank - 1;
        loop {
            root[i] = mod_mul(root[i + 1], root[i + 1], M::MOD, M::R);
            inv_root[i] = mod_mul(inv_root[i + 1], inv_root[i + 1], M::MOD, M::R);
            if i == 0 {
                break;
            }
            i -= 1;
        }

        let (mut i, mut prod, mut inv_prod) = (0, M::N1, M::N1);
        while i < rank - 1 {
            rate2[i] = mod_mul(root[i + 2], prod, M::MOD, M::R);
            inv_rate2[i] = mod_mul(inv_root[i + 2], inv_prod, M::MOD, M::R);
            prod = mod_mul(prod, inv_root[i + 2], M::MOD, M::R);
            inv_prod = mod_mul(inv_prod, root[i + 2], M::MOD, M::R);
            i += 1;
        }

        let (mut i, mut prod, mut inv_prod) = (0, M::N1, M::N1);
        while i < rank - 2 {
            rate3[i] = mod_mul(root[i + 3], prod, M::MOD, M::R);
            inv_rate3[i] = mod_mul(inv_root[i + 3], inv_prod, M::MOD, M::R);
            prod = mod_mul(prod, inv_root[i + 3], M::MOD, M::R);
            inv_prod = mod_mul(inv_prod, root[i + 3], M::MOD, M::R);
            i += 1;
        }

        NttInfo {
            root,
            inv_root,
            rate2,
            inv_rate2,
            rate3,
            inv_rate3,
        }
    }
}

macro_rules! impl_ntt {
    (@ntt $a:ident) => {
        let n = $a.len();
        let mut v = n / 2;
        let imag = MInt::<M>::new_unchecked(M::INFO.root[2]);
        while v > 1 {
            let mut w1 = MInt::<M>::one();
            for (s, a) in $a.chunks_exact_mut(v << 1).enumerate() {
                let (l, r) = a.split_at_mut(v);
                let (ll, lr) = l.split_at_mut(v >> 1);
                let (rl, rr) = r.split_at_mut(v >> 1);
                let w2 = w1 * w1;
                let w3 = w1 * w2;
                for (((x0, x1), x2), x3) in ll.iter_mut().zip(lr).zip(rl).zip(rr) {
                    let a0 = *x0;
                    let a1 = *x1 * w1;
                    let a2 = *x2 * w2;
                    let a3 = *x3 * w3;
                    let a0pa2 = a0 + a2;
                    let a0na2 = a0 - a2;
                    let a1pa3 = a1 + a3;
                    let a1na3imag = (a1 - a3) * imag;
                    *x0 = a0pa2 + a1pa3;
                    *x1 = a0pa2 - a1pa3;
                    *x2 = a0na2 + a1na3imag;
                    *x3 = a0na2 - a1na3imag;
                }
                w1 *= MInt::<M>::new_unchecked(M::INFO.rate3[s.trailing_ones() as usize]);
            }
            v >>= 2;
        }
        if v == 1 {
            let mut w1 = MInt::<M>::one();
            for (s, a) in $a.chunks_exact_mut(2).enumerate() {
                unsafe {
                    let (l, r) = a.split_at_mut(1);
                    let x0 = l.get_unchecked_mut(0);
                    let x1 = r.get_unchecked_mut(0);
                    let a0 = *x0;
                    let a1 = *x1 * w1;
                    *x0 = a0 + a1;
                    *x1 = a0 - a1;
                }
                w1 *= MInt::<M>::new_unchecked(M::INFO.rate2[s.trailing_ones() as usize]);
            }
        }
    };
    (@intt $a:ident) => {
        let n = $a.len();
        let mut v = 1;
        if n.trailing_zeros() & 1 == 1 {
            let mut w1 = MInt::<M>::one();
            for (s, a) in $a.chunks_exact_mut(2).enumerate() {
                unsafe {
                    let (l, r) = a.split_at_mut(1);
                    let x0 = l.get_unchecked_mut(0);
                    let x1 = r.get_unchecked_mut(0);
                    let a0 = *x0;
                    let a1 = *x1;
                    *x0 = a0 + a1;
                    *x1 = (a0 - a1) * w1;
                }
                w1 *= MInt::<M>::new_unchecked(M::INFO.inv_rate2[s.trailing_ones() as usize]);
            }
            v <<= 1;
        }
        let iimag = MInt::<M>::new_unchecked(M::INFO.inv_root[2]);
        while v < n {
            let mut w1 = MInt::<M>::one();
            for (s, a) in $a.chunks_exact_mut(v << 2).enumerate() {
                let (l, r) = a.split_at_mut(v << 1);
                let (ll, lr) = l.split_at_mut(v);
                let (rl, rr) = r.split_at_mut(v);
                let w2 = w1 * w1;
                let w3 = w1 * w2;
                for (((x0, x1), x2), x3) in ll.iter_mut().zip(lr).zip(rl).zip(rr) {
                    let a0 = *x0;
                    let a1 = *x1;
                    let a2 = *x2;
                    let a3 = *x3;
                    let a0pa1 = a0 + a1;
                    let a0na1 = a0 - a1;
                    let a2pa3 = a2 + a3;
                    let a2na3iimag = (a2 - a3) * iimag;
                    *x0 = a0pa1 + a2pa3;
                    *x1 = (a0na1 + a2na3iimag) * w1;
                    *x2 = (a0pa1 - a2pa3) * w2;
                    *x3 = (a0na1 - a2na3iimag) * w3;
                }
                w1 *= MInt::<M>::new_unchecked(M::INFO.inv_rate3[s.trailing_ones() as usize]);
            }
            v <<= 2;
        }
    };
}

fn ntt<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    // if is_x86_feature_detected!("avx512f")
    //     && is_x86_feature_detected!("avx512dq")
    //     && is_x86_feature_detected!("avx512cd")
    //     && is_x86_feature_detected!("avx512bw")
    //     && is_x86_feature_detected!("avx512vl")
    // {
    //     unsafe { ntt_inner_avx512(a) };
    // } else
    if is_x86_feature_detected!("avx2") {
        unsafe { ntt_inner_avx2(a) };
    } else {
        ntt_inner(a);
    }
}
// #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
// unsafe fn ntt_inner_avx512<M>(a: &mut [MInt<M>])
// where
//     M: Montgomery32NttModulus,
// {
//     impl_ntt!(@ntt a);
// }
#[target_feature(enable = "avx2")]
unsafe fn ntt_inner_avx2<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    impl_ntt!(@ntt a);
}
fn ntt_inner<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    impl_ntt!(@ntt a);
}
fn intt<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    // if is_x86_feature_detected!("avx512f")
    //     && is_x86_feature_detected!("avx512dq")
    //     && is_x86_feature_detected!("avx512cd")
    //     && is_x86_feature_detected!("avx512bw")
    //     && is_x86_feature_detected!("avx512vl")
    // {
    //     unsafe { intt_inner_avx512(a) };
    // } else
    if is_x86_feature_detected!("avx2") {
        unsafe { intt_inner_avx2(a) };
    } else {
        intt_inner(a);
    }
}
// #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
// unsafe fn intt_inner_avx512<M>(a: &mut [MInt<M>])
// where
//     M: Montgomery32NttModulus,
// {
//     impl_ntt!(@intt a);
// }
#[target_feature(enable = "avx2")]
unsafe fn intt_inner_avx2<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    impl_ntt!(@intt a);
}
fn intt_inner<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    impl_ntt!(@intt a);
}

fn convolve_naive<M>(a: &[MInt<M>], b: &[MInt<M>]) -> Vec<MInt<M>>
where
    M: MIntBase,
{
    if a.is_empty() && b.is_empty() {
        return Vec::new();
    }
    let len = a.len() + b.len() - 1;
    let mut c = vec![MInt::<M>::zero(); len];
    if a.len() < b.len() {
        for (i, &b) in b.iter().enumerate() {
            for (a, c) in a.iter().zip(&mut c[i..]) {
                *c += *a * b;
            }
        }
    } else {
        for (i, &a) in a.iter().enumerate() {
            for (b, c) in b.iter().zip(&mut c[i..]) {
                *c += *b * a;
            }
        }
    }
    c
}
impl<M> ConvolveSteps for Convolve<M>
where
    M: Montgomery32NttModulus,
{
    type T = Vec<MInt<M>>;
    type F = Vec<MInt<M>>;
    fn length(t: &Self::T) -> usize {
        t.len()
    }
    fn transform(mut t: Self::T, len: usize) -> Self::F {
        t.resize_with(len.max(2).next_power_of_two(), Zero::zero);
        ntt(&mut t);
        t
    }
    fn inverse_transform(mut f: Self::F, len: usize) -> Self::T {
        intt(&mut f);
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
    fn convolve(mut a: Self::T, mut b: Self::T) -> Self::T {
        if Self::length(&a).min(Self::length(&b)) <= 60 {
            return convolve_naive(&a, &b);
        }
        let len = (Self::length(&a) + Self::length(&b)).saturating_sub(1);
        let size = len.max(2).next_power_of_two();
        if len <= size / 2 + 2 {
            let xa = a.pop().unwrap();
            let xb = b.pop().unwrap();
            let mut c = vec![MInt::<M>::zero(); len];
            *c.last_mut().unwrap() = xa * xb;
            for (a, c) in a.iter().zip(&mut c[b.len()..]) {
                *c += *a * xb;
            }
            for (b, c) in b.iter().zip(&mut c[a.len()..]) {
                *c += *b * xa;
            }
            let d = Self::convolve(a, b);
            for (d, c) in d.into_iter().zip(&mut c) {
                *c += d;
            }
            return c;
        }
        let same = a == b;
        let mut a = Self::transform(a, len);
        if same {
            for a in a.iter_mut() {
                *a *= *a;
            }
        } else {
            let b = Self::transform(b, len);
            Self::multiply(&mut a, &b);
        }
        Self::inverse_transform(a, len)
    }
}
type MVec<M> = Vec<MInt<M>>;
impl<M, N1, N2, N3> ConvolveSteps for Convolve<(M, (N1, N2, N3))>
where
    M: MIntConvert + MIntConvert<u32>,
    N1: Montgomery32NttModulus,
    N2: Montgomery32NttModulus,
    N3: Montgomery32NttModulus,
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
        ntt(&mut f.0);
        ntt(&mut f.1);
        ntt(&mut f.2);
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
    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        if Self::length(&a).min(Self::length(&b)) <= 60 {
            return convolve_naive(&a, &b);
        }
        let len = (Self::length(&a) + Self::length(&b)).saturating_sub(1);
        let mut a = Self::transform(a, len);
        let b = Self::transform(b, len);
        Self::multiply(&mut a, &b);
        Self::inverse_transform(a, len)
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

    const N: usize = 8;

    #[test]
    fn test_ntt998244353() {
        let mut rng = Xorshift::new();
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
        let mut rng = Xorshift::new();
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
                        if divisors(p - 1)
                            .into_iter()
                            .filter(|&d| d != p - 1)
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
