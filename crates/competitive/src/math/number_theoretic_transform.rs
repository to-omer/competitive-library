use super::{montgomery::*, ConvolveSteps, MInt, MIntBase, MIntConvert, One, Zero};
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{AddAssign, Mul, SubAssign},
};

pub struct Convolve<M>(PhantomData<fn() -> M>);
pub type Convolve998244353 = Convolve<Modulo998244353>;
pub type MIntConvolve<M> = Convolve<(M, (Modulo2013265921, Modulo1811939329, Modulo2113929217))>;
pub type U64Convolve = Convolve<(u64, (Modulo2013265921, Modulo1811939329, Modulo2113929217))>;

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

#[derive(Debug, PartialEq)]
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

crate::avx_helper!(
    @avx2 fn ntt<M>(a: &mut [MInt<M>])
    where
        [M: Montgomery32NttModulus]
    {
        let n = a.len();
        let mut v = n / 2;
        let imag = MInt::<M>::new_unchecked(M::INFO.root[2]);
        while v > 1 {
            let mut w1 = MInt::<M>::one();
            for (s, a) in a.chunks_exact_mut(v << 1).enumerate() {
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
            for (s, a) in a.chunks_exact_mut(2).enumerate() {
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
    }
);
crate::avx_helper!(
    @avx2 fn intt<M>(a: &mut [MInt<M>])
    where
        [M: Montgomery32NttModulus]
    {
        let n = a.len();
        let mut v = 1;
        if n.trailing_zeros() & 1 == 1 {
            let mut w1 = MInt::<M>::one();
            for (s, a) in a.chunks_exact_mut(2).enumerate() {
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
            for (s, a) in a.chunks_exact_mut(v << 2).enumerate() {
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
    }
);

fn convolve_naive<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Copy + Zero + AddAssign<T> + Mul<Output = T>,
{
    if a.is_empty() && b.is_empty() {
        return Vec::new();
    }
    let len = a.len() + b.len() - 1;
    let mut c = vec![T::zero(); len];
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

fn convolve_karatsuba<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Copy + Zero + AddAssign<T> + SubAssign<T> + Mul<Output = T>,
{
    if a.len().min(b.len()) <= 30 {
        return convolve_naive(a, b);
    }
    let m = a.len().max(b.len()).div_ceil(2);
    let (a0, a1) = if a.len() <= m {
        (a, &[][..])
    } else {
        a.split_at(m)
    };
    let (b0, b1) = if b.len() <= m {
        (b, &[][..])
    } else {
        b.split_at(m)
    };
    let f00 = convolve_karatsuba(a0, b0);
    let f11 = convolve_karatsuba(a1, b1);
    let mut a0a1 = a0.to_vec();
    for (a0a1, &a1) in a0a1.iter_mut().zip(a1) {
        *a0a1 += a1;
    }
    let mut b0b1 = b0.to_vec();
    for (b0b1, &b1) in b0b1.iter_mut().zip(b1) {
        *b0b1 += b1;
    }
    let mut f01 = convolve_karatsuba(&a0a1, &b0b1);
    for (f01, &f00) in f01.iter_mut().zip(&f00) {
        *f01 -= f00;
    }
    for (f01, &f11) in f01.iter_mut().zip(&f11) {
        *f01 -= f11;
    }
    let mut c = vec![T::zero(); a.len() + b.len() - 1];
    for (c, &f00) in c.iter_mut().zip(&f00) {
        *c += f00;
    }
    for (c, &f01) in c[m..].iter_mut().zip(&f01) {
        *c += f01;
    }
    for (c, &f11) in c[m << 1..].iter_mut().zip(&f11) {
        *c += f11;
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
        t.resize_with(len.max(1).next_power_of_two(), Zero::zero);
        ntt(&mut t);
        t
    }
    fn inverse_transform(mut f: Self::F, len: usize) -> Self::T {
        intt(&mut f);
        f.truncate(len);
        let inv = MInt::from(len.max(1).next_power_of_two() as u32).inv();
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
        if Self::length(&a).max(Self::length(&b)) <= 100 {
            return convolve_karatsuba(&a, &b);
        }
        if Self::length(&a).min(Self::length(&b)) <= 60 {
            return convolve_naive(&a, &b);
        }
        let len = (Self::length(&a) + Self::length(&b)).saturating_sub(1);
        let size = len.max(1).next_power_of_two();
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
        let npot = len.max(1).next_power_of_two();
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
        if Self::length(&a).max(Self::length(&b)) <= 300 {
            return convolve_karatsuba(&a, &b);
        }
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

impl<N1, N2, N3> ConvolveSteps for Convolve<(u64, (N1, N2, N3))>
where
    N1: Montgomery32NttModulus,
    N2: Montgomery32NttModulus,
    N3: Montgomery32NttModulus,
{
    type T = Vec<u64>;
    type F = (MVec<N1>, MVec<N2>, MVec<N3>);

    fn length(t: &Self::T) -> usize {
        t.len()
    }

    fn transform(t: Self::T, len: usize) -> Self::F {
        let npot = len.max(1).next_power_of_two();
        let mut f = (
            MVec::<N1>::with_capacity(npot),
            MVec::<N2>::with_capacity(npot),
            MVec::<N3>::with_capacity(npot),
        );
        for t in t {
            f.0.push(t.into());
            f.1.push(t.into());
            f.2.push(t.into());
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
        let m1 = N1::get_mod() as u64;
        let m1_3 = MInt::<N3>::new(N1::get_mod());
        let t2 = (m1_3 * MInt::<N3>::new(N2::get_mod())).inv();
        let m2 = m1 * N2::get_mod() as u64;
        Convolve::<N1>::inverse_transform(f.0, len)
            .into_iter()
            .zip(Convolve::<N2>::inverse_transform(f.1, len))
            .zip(Convolve::<N3>::inverse_transform(f.2, len))
            .map(|((c1, c2), c3)| {
                let d1 = c1.inner();
                let d2 = ((c2 - MInt::<N2>::from(d1)) * t1).inner();
                let x = MInt::<N3>::new(d1) + MInt::<N3>::new(d2) * m1_3;
                let d3 = ((c3 - x) * t2).inner();
                d1 as u64 + d2 as u64 * m1 + d3 as u64 * m2
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
        if Self::length(&a).max(Self::length(&b)) <= 300 {
            return convolve_karatsuba(&a, &b);
        }
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

pub trait NttReuse: ConvolveSteps {
    const MULTIPLE: bool = true;

    /// F(a) → F(a + [0] * a.len())
    fn ntt_doubling(f: Self::F) -> Self::F;

    /// F(a(x)), F(b(x)) → even(F(a(x) * b(-x)))
    fn even_mul_normal_neg(f: &Self::F, g: &Self::F) -> Self::F;

    /// F(a(x)), F(b(x)) → odd(F(a(x) * b(-x)))
    fn odd_mul_normal_neg(f: &Self::F, g: &Self::F) -> Self::F;
}

thread_local!(
    static BIT_REVERSE: UnsafeCell<Vec<Vec<usize>>> = const { UnsafeCell::new(vec![]) };
);

impl<M> NttReuse for Convolve<M>
where
    M: Montgomery32NttModulus,
{
    const MULTIPLE: bool = false;

    fn ntt_doubling(mut f: Self::F) -> Self::F {
        let n = f.len();
        let k = n.trailing_zeros() as usize;
        let mut a = Self::inverse_transform(f.clone(), n);
        let mut rot = MInt::<M>::one();
        let zeta = MInt::<M>::new_unchecked(M::INFO.root[k + 1]);
        for a in a.iter_mut() {
            *a *= rot;
            rot *= zeta;
        }
        f.extend(Self::transform(a, n));
        f
    }

    fn even_mul_normal_neg(f: &Self::F, g: &Self::F) -> Self::F {
        assert_eq!(f.len(), g.len());
        assert!(f.len().is_power_of_two());
        assert!(f.len() >= 2);
        let inv2 = MInt::<M>::from(2).inv();
        let n = f.len() / 2;
        (0..n)
            .map(|i| (f[i << 1] * g[i << 1 | 1] + f[i << 1 | 1] * g[i << 1]) * inv2)
            .collect()
    }

    fn odd_mul_normal_neg(f: &Self::F, g: &Self::F) -> Self::F {
        assert_eq!(f.len(), g.len());
        assert!(f.len().is_power_of_two());
        assert!(f.len() >= 2);
        let mut inv2 = MInt::<M>::from(2).inv();
        let n = f.len() / 2;
        let k = f.len().trailing_zeros() as usize;
        let mut h = vec![MInt::<M>::zero(); n];
        let w = MInt::<M>::new_unchecked(M::INFO.inv_root[k]);
        BIT_REVERSE.with(|br| {
            let br = unsafe { &mut *br.get() };
            if br.len() < k {
                br.resize_with(k, Default::default);
            }
            let k = k - 1;
            if br[k].is_empty() {
                let mut v = vec![0; 1 << k];
                for i in 0..1 << k {
                    v[i] = (v[i >> 1] >> 1) | ((i & 1) << (k.saturating_sub(1)));
                }
                br[k] = v;
            }
            for &i in &br[k] {
                h[i] = (f[i << 1] * g[i << 1 | 1] - f[i << 1 | 1] * g[i << 1]) * inv2;
                inv2 *= w;
            }
        });
        h
    }
}

impl<M, N1, N2, N3> NttReuse for Convolve<(M, (N1, N2, N3))>
where
    M: MIntConvert + MIntConvert<u32>,
    N1: Montgomery32NttModulus,
    N2: Montgomery32NttModulus,
    N3: Montgomery32NttModulus,
{
    fn ntt_doubling(f: Self::F) -> Self::F {
        (
            Convolve::<N1>::ntt_doubling(f.0),
            Convolve::<N2>::ntt_doubling(f.1),
            Convolve::<N3>::ntt_doubling(f.2),
        )
    }

    fn even_mul_normal_neg(f: &Self::F, g: &Self::F) -> Self::F {
        fn even_mul_normal_neg_corrected<M>(f: &[MInt<M>], g: &[MInt<M>], m: u32) -> Vec<MInt<M>>
        where
            M: Montgomery32NttModulus,
        {
            let n = f.len();
            assert_eq!(f.len(), g.len());
            assert!(f.len().is_power_of_two());
            assert!(f.len() >= 2);
            let inv2 = MInt::<M>::from(2).inv();
            let u = MInt::<M>::new(m) * MInt::<M>::from(n as u32);
            let n = f.len() / 2;
            (0..n)
                .map(|i| {
                    (f[i << 1]
                        * if i == 0 {
                            g[i << 1 | 1] + u
                        } else {
                            g[i << 1 | 1]
                        }
                        + f[i << 1 | 1] * g[i << 1])
                        * inv2
                })
                .collect()
        }

        let m = M::mod_into();
        (
            even_mul_normal_neg_corrected(&f.0, &g.0, m),
            even_mul_normal_neg_corrected(&f.1, &g.1, m),
            even_mul_normal_neg_corrected(&f.2, &g.2, m),
        )
    }

    fn odd_mul_normal_neg(f: &Self::F, g: &Self::F) -> Self::F {
        fn odd_mul_normal_neg_corrected<M>(f: &[MInt<M>], g: &[MInt<M>], m: u32) -> Vec<MInt<M>>
        where
            M: Montgomery32NttModulus,
        {
            assert_eq!(f.len(), g.len());
            assert!(f.len().is_power_of_two());
            assert!(f.len() >= 2);
            let mut inv2 = MInt::<M>::from(2).inv();
            let u = MInt::<M>::new(m) * MInt::<M>::from(f.len() as u32);
            let n = f.len() / 2;
            let k = f.len().trailing_zeros() as usize;
            let mut h = vec![MInt::<M>::zero(); n];
            let w = MInt::<M>::new_unchecked(M::INFO.inv_root[k]);
            BIT_REVERSE.with(|br| {
                let br = unsafe { &mut *br.get() };
                if br.len() < k {
                    br.resize_with(k, Default::default);
                }
                let k = k - 1;
                if br[k].is_empty() {
                    let mut v = vec![0; 1 << k];
                    for i in 0..1 << k {
                        v[i] = (v[i >> 1] >> 1) | ((i & 1) << (k.saturating_sub(1)));
                    }
                    br[k] = v;
                }
                for &i in &br[k] {
                    h[i] = (f[i << 1]
                        * if i == 0 {
                            g[i << 1 | 1] + u
                        } else {
                            g[i << 1 | 1]
                        }
                        - f[i << 1 | 1] * g[i << 1])
                        * inv2;
                    inv2 *= w;
                }
            });
            h
        }

        let m = M::mod_into();
        (
            odd_mul_normal_neg_corrected(&f.0, &g.0, m),
            odd_mul_normal_neg_corrected(&f.1, &g.1, m),
            odd_mul_normal_neg_corrected(&f.2, &g.2, m),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::{mint_basic::Modulo1000000009, montgomery::MInt998244353};
    use crate::tools::Xorshift;

    #[test]
    fn test_convolve_naive() {
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let n = rng.random(0..=60);
            let m = rng.random(0..=60);
            let a: Vec<u32> = rng.random_iter(0u32..1000).take(n).collect();
            let b: Vec<u32> = rng.random_iter(0u32..1000).take(m).collect();
            let mut c = vec![0u32; (n + m).saturating_sub(1)];
            for i in 0..n {
                for j in 0..m {
                    c[i + j] += a[i] * b[j];
                }
            }
            let d = convolve_naive(&a, &b);
            assert_eq!(c, d);
        }
    }

    #[test]
    fn test_convolve_karatsuba() {
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let n = rng.random(0..=200);
            let m = rng.random(0..=200);
            let a: Vec<u32> = rng.random_iter(0u32..1000).take(n).collect();
            let b: Vec<u32> = rng.random_iter(0u32..1000).take(m).collect();
            let mut c = vec![0u32; (n + m).saturating_sub(1)];
            for i in 0..n {
                for j in 0..m {
                    c[i + j] += a[i] * b[j];
                }
            }
            let d = convolve_karatsuba(&a, &b);
            assert_eq!(c, d);
        }
    }

    #[test]
    fn test_ntt998244353() {
        let mut rng = Xorshift::default();
        for t in 0..1000 {
            let n: usize = rng.random(0..=5);
            let n = if n == 5 { rng.random(70..=120) } else { n };
            let m: usize = rng.random(0..=5);
            let m = if m == 5 { rng.random(70..=120) } else { m };
            let (n, m) = if t % 100 != 0 {
                (n, m)
            } else {
                let w = rng.random(6..=8);
                ((1usize << w) + 1usize, (1usize << w) + 1usize)
            };
            let a: Vec<MInt998244353> = rng.random_iter(..).take(n).collect();
            let mut b: Vec<MInt998244353> = rng.random_iter(..).take(m).collect();
            if n == m && rng.random(0..2) == 0 {
                b = a.clone();
            }

            let mut c = vec![MInt998244353::zero(); (n + m).saturating_sub(1)];
            for i in 0..n {
                for j in 0..m {
                    c[i + j] += a[i] * b[j];
                }
            }
            let d = Convolve998244353::convolve(a, b);
            assert_eq!(c, d);
        }
        assert_eq!(NttInfo::new::<Modulo998244353>(), Modulo998244353::INFO);
    }

    #[test]
    fn test_convolve3() {
        type M = MInt<Modulo1000000009>;
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let n = rng.random(0..=5);
            let n = if n == 5 { rng.random(70..=400) } else { n };
            let m = rng.random(0..=5);
            let m = if m == 5 { rng.random(70..=400) } else { m };
            let a: Vec<M> = rng.random_iter(..).take(n).collect();
            let b: Vec<M> = rng.random_iter(..).take(m).collect();
            let mut c = vec![M::zero(); (n + m).saturating_sub(1)];
            for i in 0..n {
                for j in 0..m {
                    c[i + j] += a[i] * b[j];
                }
            }
            let d = MIntConvolve::<Modulo1000000009>::convolve(a, b);
            assert_eq!(c, d);
        }
    }

    #[test]
    fn test_convolve_u64() {
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let n = rng.random(0..=5);
            let n = if n == 5 { rng.random(70..=400) } else { n };
            let m = rng.random(0..=5);
            let m = if m == 5 { rng.random(70..=400) } else { m };
            let a: Vec<u64> = rng.random_iter(0u64..1 << 24).take(n).collect();
            let b: Vec<u64> = rng.random_iter(0u64..1 << 24).take(m).collect();
            let mut c = vec![0; (n + m).saturating_sub(1)];
            for i in 0..n {
                for j in 0..m {
                    c[i + j] += a[i] * b[j];
                }
            }
            let d = U64Convolve::convolve(a, b);
            assert_eq!(c, d);
        }
    }

    #[test]
    fn test_ntt_reuse_998244353() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n: usize = if rng.gen_bool(0.5) {
                rng.random(1..=20)
            } else {
                rng.random(1..=1000)
            };
            let a: Vec<MInt998244353> = rng.random_iter(..).take(n).collect();
            let f = Convolve998244353::transform(a.clone(), n);

            // doubling
            {
                let f_double = Convolve998244353::ntt_doubling(f.clone());
                let mut a = a.clone();
                a.resize_with(n * 2, Zero::zero);
                let f2 = Convolve998244353::transform(a, n * 2);
                assert_eq!(f_double, f2);
            }

            let f = Convolve998244353::transform(a.clone(), n * 2);
            let b: Vec<MInt998244353> = rng.random_iter(..).take(n).collect();
            let g = Convolve998244353::transform(b.clone(), n * 2);
            let mut b_neg = b.clone();
            for b in b_neg.iter_mut().skip(1).step_by(2) {
                *b = -*b;
            }

            // even_mul_normal_neg
            {
                let fg_neg = Convolve998244353::even_mul_normal_neg(&f, &g);
                let ab_neg_even: Vec<_> = Convolve998244353::convolve(a.clone(), b_neg.clone())
                    .into_iter()
                    .step_by(2)
                    .collect();
                let fg = Convolve998244353::transform(ab_neg_even, n);
                assert_eq!(fg_neg, fg);
            }

            // odd_mul_normal_neg
            {
                let fg_neg = Convolve998244353::odd_mul_normal_neg(&f, &g);
                let ab_neg_odd: Vec<_> = Convolve998244353::convolve(a.clone(), b_neg.clone())
                    .into_iter()
                    .skip(1)
                    .step_by(2)
                    .collect();
                let fg = Convolve998244353::transform(ab_neg_odd, n);
                assert_eq!(fg_neg, fg);
            }
        }
    }

    #[test]
    fn test_ntt_reuse_triple() {
        type M = MInt<Modulo1000000009>;
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n: usize = if rng.gen_bool(0.5) {
                rng.random(1..=20)
            } else {
                rng.random(1..=1000)
            };
            let a: Vec<M> = rng.random_iter(..).take(n).collect();
            let f = MIntConvolve::<Modulo1000000009>::transform(a.clone(), n);

            // doubling
            {
                let f_double = MIntConvolve::<Modulo1000000009>::ntt_doubling(f.clone());
                let mut a = a.clone();
                a.resize_with(n * 2, Zero::zero);
                let f2 = MIntConvolve::<Modulo1000000009>::transform(a, n * 2);
                assert_eq!(f_double, f2);
            }

            let f = MIntConvolve::<Modulo1000000009>::transform(a.clone(), n * 2);
            let b: Vec<M> = rng.random_iter(..).take(n).collect();
            let g = MIntConvolve::<Modulo1000000009>::transform(b.clone(), n * 2);
            let mut b_neg = b.clone();
            for b in b_neg.iter_mut().skip(1).step_by(2) {
                *b = -*b;
            }

            // even_mul_normal_neg
            {
                let fg_neg = MIntConvolve::<Modulo1000000009>::even_mul_normal_neg(&f, &g);
                let ab_neg_even: Vec<_> =
                    MIntConvolve::<Modulo1000000009>::convolve(a.clone(), b_neg.clone())
                        .into_iter()
                        .step_by(2)
                        .collect();
                assert_eq!(
                    MIntConvolve::<Modulo1000000009>::inverse_transform(fg_neg.clone(), n),
                    ab_neg_even
                );
            }

            // odd_mul_normal_neg
            {
                let fg_neg = MIntConvolve::<Modulo1000000009>::odd_mul_normal_neg(&f, &g);
                let ab_neg_odd: Vec<_> =
                    MIntConvolve::<Modulo1000000009>::convolve(a.clone(), b_neg.clone())
                        .into_iter()
                        .skip(1)
                        .step_by(2)
                        .chain([M::zero()])
                        .collect();
                assert_eq!(
                    MIntConvolve::<Modulo1000000009>::inverse_transform(fg_neg.clone(), n),
                    ab_neg_odd
                );
            }
        }
    }
}
