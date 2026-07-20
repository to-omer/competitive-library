use super::{
    ConvolveSteps, MInt, MIntBase, MIntConvert, One, Xorshift, Zero,
    fast_fourier_transform::ConvolveRealFft, montgomery::*,
};
#[cfg(target_arch = "x86_64")]
use super::{SimdBackend, avx512_supported, simd_backend};
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{AddAssign, Mul, SubAssign},
};

pub struct Convolve<M>(PhantomData<fn() -> M>);
pub type Convolve998244353 = Convolve<Modulo998244353>;
pub type MIntConvolve<M> = Convolve<(M, (Modulo167772161, Modulo469762049, Modulo754974721))>;
pub type U64Convolve = Convolve<(u64, (Modulo167772161, Modulo469762049, Modulo754974721))>;

macro_rules! impl_ntt_modulus {
    ($([$name:ident, $g:expr]),*) => {
        $(
            impl Montgomery32NttModulus for $name {}
        )*
    };
}
impl_ntt_modulus!(
    [Modulo167772161, 3],
    [Modulo469762049, 3],
    [Modulo754974721, 11],
    [Modulo998244353, 3]
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
    rate3: [u32; 32],
    inv_rate3: [u32; 32],
    rate3_packed: [[u32; 8]; 32],
    inv_rate3_packed: [[u32; 8]; 32],
}
impl NttInfo {
    const fn new<M>() -> Self
    where
        M: Montgomery32NttModulus,
    {
        let mut root = [0; 32];
        let mut inv_root = [0; 32];
        let mut rate3 = [0; 32];
        let mut inv_rate3 = [0; 32];
        let mut rate3_packed = [[0; 8]; 32];
        let mut inv_rate3_packed = [[0; 8]; 32];
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
        while i < rank - 2 {
            rate3[i] = mod_mul(root[i + 3], prod, M::MOD, M::R);
            let rate3_2 = mod_mul(rate3[i], rate3[i], M::MOD, M::R);
            let rate3_3 = mod_mul(rate3_2, rate3[i], M::MOD, M::R);
            inv_rate3[i] = mod_mul(inv_root[i + 3], inv_prod, M::MOD, M::R);
            let inv_rate3_2 = mod_mul(inv_rate3[i], inv_rate3[i], M::MOD, M::R);
            let inv_rate3_3 = mod_mul(inv_rate3_2, inv_rate3[i], M::MOD, M::R);
            rate3_packed[i] = [
                rate3[i].wrapping_mul(M::R),
                rate3[i],
                rate3_2.wrapping_mul(M::R),
                rate3_2,
                rate3_3.wrapping_mul(M::R),
                rate3_3,
                0,
                0,
            ];
            inv_rate3_packed[i] = [
                inv_rate3[i].wrapping_mul(M::R),
                inv_rate3[i],
                inv_rate3_2.wrapping_mul(M::R),
                inv_rate3_2,
                inv_rate3_3.wrapping_mul(M::R),
                inv_rate3_3,
                0,
                0,
            ];
            prod = mod_mul(prod, inv_root[i + 3], M::MOD, M::R);
            inv_prod = mod_mul(inv_prod, root[i + 3], M::MOD, M::R);
            i += 1;
        }

        NttInfo {
            root,
            inv_root,
            rate3,
            inv_rate3,
            rate3_packed,
            inv_rate3_packed,
        }
    }
}

fn ntt_scalar<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let n = a.len();
    let mut v = n / 2;
    if n.trailing_zeros() & 1 == 1 {
        let (l, r) = a.split_at_mut(v);
        for (x0, x1) in l.iter_mut().zip(r) {
            let a0 = *x0;
            let a1 = *x1;
            *x0 = a0 + a1;
            *x1 = a0 - a1;
        }
        v >>= 1;
    }
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
}

fn intt_scalar<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let n = a.len();
    let mut v = 1;
    let limit = if n.trailing_zeros() & 1 == 1 {
        n / 2
    } else {
        n
    };
    let iimag = MInt::<M>::new_unchecked(M::INFO.inv_root[2]);
    while v < limit {
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
    if n.trailing_zeros() & 1 == 1 {
        let (l, r) = a.split_at_mut(n / 2);
        for (x0, x1) in l.iter_mut().zip(r) {
            let a0 = *x0;
            let a1 = *x1;
            *x0 = a0 + a1;
            *x1 = a0 - a1;
        }
    }
    let inv = MInt::<M>::from(n as u32).inv();
    for a in a {
        *a *= inv;
    }
}

fn ntt<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    #[cfg(target_arch = "x86_64")]
    match simd_backend() {
        SimdBackend::Avx512 => unsafe { ntt_simd::ntt_avx512(a) },
        SimdBackend::Avx2 => unsafe { ntt_simd::ntt_avx2(a) },
        SimdBackend::Scalar => ntt_scalar(a),
    }
    #[cfg(not(target_arch = "x86_64"))]
    ntt_scalar(a);
}

fn intt<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    #[cfg(target_arch = "x86_64")]
    match simd_backend() {
        SimdBackend::Avx512 => unsafe { ntt_simd::intt_avx512(a) },
        SimdBackend::Avx2 => unsafe { ntt_simd::intt_avx2(a) },
        SimdBackend::Scalar => intt_scalar(a),
    }
    #[cfg(not(target_arch = "x86_64"))]
    intt_scalar(a);
}

#[cfg(target_arch = "x86_64")]
fn use_block_ntt<M>(len: usize) -> bool
where
    M: Montgomery32NttModulus,
{
    len >= 64 && M::MOD < 1 << 30 && is_x86_feature_detected!("avx2") && !avx512_supported()
}

fn pointwise_multiply<M>(f: &mut [MInt<M>], g: &[MInt<M>])
where
    M: Montgomery32NttModulus,
{
    assert!(f.len() <= g.len());
    crate::avx_helper!(
        @dispatch simd_backend, SimdBackend;
        unsafe { ntt_simd::pointwise_multiply_avx512(f, g) },
        unsafe { ntt_simd::pointwise_multiply_avx2(f, g) },
        {
            for (f, g) in f.iter_mut().zip(g.iter()) {
                *f *= *g;
            }
        }
    )
}

fn pointwise_multiply_add<M>(sum: &mut [MInt<M>], f: &[MInt<M>], g: &[MInt<M>])
where
    M: Montgomery32NttModulus,
{
    crate::avx_helper!(
        @dispatch simd_backend, SimdBackend;
        unsafe { ntt_simd::pointwise_multiply_add_avx512(sum, f, g) },
        unsafe { ntt_simd::pointwise_multiply_add_avx2(sum, f, g) },
        {
            for ((sum, f), g) in sum.iter_mut().zip(f.iter()).zip(g.iter()) {
                *sum += *f * *g;
            }
        }
    )
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)] // SIMD intrinsics and raw pointers are confined here
mod ntt_simd;

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

#[cold]
fn convolve_large_ntt<M>(a: Vec<MInt<M>>, b: Vec<MInt<M>>) -> Vec<MInt<M>>
where
    M: Montgomery32NttModulus,
{
    let len = a.len() + b.len() - 1;
    let ntt_len = 1usize << M::RANK;
    let block_len = ntt_len / 2;
    let same = a == b;
    let fa: Vec<_> = a
        .chunks(block_len)
        .map(|a| Convolve::<M>::transform_ntt(a.to_vec(), ntt_len))
        .collect();
    let fb: Option<Vec<_>> = if same {
        None
    } else {
        Some(
            b.chunks(block_len)
                .map(|b| Convolve::<M>::transform_ntt(b.to_vec(), ntt_len))
                .collect(),
        )
    };
    let b_blocks = fb.as_ref().map_or(fa.len(), Vec::len);
    let mut result = vec![MInt::<M>::zero(); len];
    for diagonal in 0..fa.len() + b_blocks - 1 {
        let mut spectrum = vec![MInt::<M>::zero(); ntt_len];
        let start = diagonal.saturating_sub(b_blocks - 1);
        for i in start..=diagonal.min(fa.len() - 1) {
            let j = diagonal - i;
            let g = if let Some(fb) = &fb { &fb[j] } else { &fa[j] };
            pointwise_multiply_add(&mut spectrum, &fa[i], g);
        }
        spectrum = Convolve::<M>::inverse_transform_ntt(spectrum, ntt_len);
        let offset = diagonal * block_len;
        for (result, value) in result[offset..].iter_mut().zip(spectrum) {
            *result += value;
        }
    }
    result
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
        #[cfg(target_arch = "x86_64")]
        if use_block_ntt::<M>(t.len()) {
            unsafe { ntt_simd::transform_blocks_avx2(&mut t) };
            return t;
        }
        ntt(&mut t);
        t
    }
    fn inverse_transform(mut f: Self::F, len: usize) -> Self::T {
        #[cfg(target_arch = "x86_64")]
        if use_block_ntt::<M>(f.len()) {
            unsafe { ntt_simd::inverse_transform_blocks_avx2(&mut f) };
            f.truncate(len);
            return f;
        }
        intt(&mut f);
        f.truncate(len);
        f
    }
    fn multiply(f: &mut Self::F, g: &Self::F) {
        assert_eq!(f.len(), g.len());
        #[cfg(target_arch = "x86_64")]
        if use_block_ntt::<M>(f.len()) {
            unsafe { ntt_simd::multiply_blocks_avx2(f, g) };
            return;
        }
        pointwise_multiply(f, g);
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
        if size > 1usize << M::RANK {
            return convolve_large_ntt(a, b);
        }
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
        #[cfg(target_arch = "x86_64")]
        if use_block_ntt::<M>(size) {
            a.resize_with(size, Zero::zero);
            b.resize_with(size, Zero::zero);
            unsafe { ntt_simd::convolve_blocks_avx2(&mut a, &mut b, same) };
            a.truncate(len);
            return a;
        }
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

fn convert_crt_input<M, N1, N2, N3>(t: MVec<M>, capacity: usize) -> (MVec<N1>, MVec<N2>, MVec<N3>)
where
    M: MIntConvert<u32>,
    N1: Montgomery32NttModulus,
    N2: Montgomery32NttModulus,
    N3: Montgomery32NttModulus,
{
    let mut f = (
        MVec::<N1>::with_capacity(capacity),
        MVec::<N2>::with_capacity(capacity),
        MVec::<N3>::with_capacity(capacity),
    );
    for t in t {
        let t = <M as MIntConvert<u32>>::into(t.inner());
        f.0.push(t.into());
        f.1.push(t.into());
        f.2.push(t.into());
    }
    f
}

fn reconstruct_mint_crt<M, N1, N2, N3>(f: (MVec<N1>, MVec<N2>, MVec<N3>)) -> MVec<M>
where
    M: MIntConvert + MIntConvert<u32>,
    N1: Montgomery32NttModulus,
    N2: Montgomery32NttModulus,
    N3: Montgomery32NttModulus,
{
    let t1 = MInt::<N2>::new(N1::get_mod()).inv();
    let m1_3 = MInt::<N3>::new(N1::get_mod());
    let t2 = (m1_3 * MInt::<N3>::new(N2::get_mod())).inv();
    let modulus = <M as MIntConvert<u32>>::mod_into() as u64;
    let m1 = N1::get_mod() as u64;
    let m2 = m1 * N2::get_mod() as u64 % modulus;
    let fits_u64 = (N1::get_mod() - 1) as u128
        + (N2::get_mod() - 1) as u128 * m1 as u128
        + (N3::get_mod() - 1) as u128 * m2 as u128
        <= u64::MAX as u128;
    f.0.into_iter()
        .zip(f.1)
        .zip(f.2)
        .map(|((c1, c2), c3)| {
            let d1 = c1.inner();
            let d2 = ((c2 - MInt::<N2>::from(d1)) * t1).inner();
            let x = MInt::<N3>::new(d1) + MInt::<N3>::new(d2) * m1_3;
            let d3 = ((c3 - x) * t2).inner();
            let value = if fits_u64 {
                (d1 as u64 + d2 as u64 * m1 + d3 as u64 * m2) % modulus
            } else {
                ((d1 as u128 + d2 as u128 * m1 as u128 + d3 as u128 * m2 as u128) % modulus as u128)
                    as u64
            };
            MInt::<M>::from(value as u32)
        })
        .collect()
}

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
        let f = convert_crt_input(t, npot);
        (
            Convolve::<N1>::transform(f.0, npot),
            Convolve::<N2>::transform(f.1, npot),
            Convolve::<N3>::transform(f.2, npot),
        )
    }
    fn inverse_transform(f: Self::F, len: usize) -> Self::T {
        reconstruct_mint_crt((
            Convolve::<N1>::inverse_transform(f.0, len),
            Convolve::<N2>::inverse_transform(f.1, len),
            Convolve::<N3>::inverse_transform(f.2, len),
        ))
    }
    fn multiply(f: &mut Self::F, g: &Self::F) {
        Convolve::<N1>::multiply(&mut f.0, &g.0);
        Convolve::<N2>::multiply(&mut f.1, &g.1);
        Convolve::<N3>::multiply(&mut f.2, &g.2);
    }
    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        if Self::length(&a).max(Self::length(&b)) <= 300 {
            return convolve_karatsuba(&a, &b);
        }
        if Self::length(&a).min(Self::length(&b)) <= 60 {
            return convolve_naive(&a, &b);
        }
        if (a.len() + b.len() - 1).next_power_of_two() <= 1 << 20 {
            crate::avx_helper!(@dispatch_avx2_fma return unsafe {
                super::mint_fft_convolve::convolve_mint_avx2(a, b)
            }, ());
        }
        let a_len = a.len();
        let b_len = b.len();
        let a = convert_crt_input(a, a_len);
        let b = convert_crt_input(b, b_len);
        reconstruct_mint_crt((
            Convolve::<N1>::convolve(a.0, b.0),
            Convolve::<N2>::convolve(a.1, b.1),
            Convolve::<N3>::convolve(a.2, b.2),
        ))
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
        (
            Convolve::<N1>::transform(f.0, npot),
            Convolve::<N2>::transform(f.1, npot),
            Convolve::<N3>::transform(f.2, npot),
        )
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
        Convolve::<N1>::multiply(&mut f.0, &g.0);
        Convolve::<N2>::multiply(&mut f.1, &g.1);
        Convolve::<N3>::multiply(&mut f.2, &g.2);
    }

    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        if Self::length(&a).max(Self::length(&b)) <= 300 {
            return convolve_karatsuba(&a, &b);
        }
        if Self::length(&a).min(Self::length(&b)) <= 60 {
            return convolve_naive(&a, &b);
        }
        let len = (Self::length(&a) + Self::length(&b)).saturating_sub(1);
        if len.next_power_of_two() <= 1 << 21 {
            let factor = Xorshift::new().rand64() | 1;
            let mut inverse = factor;
            for _ in 0..5 {
                inverse = inverse.wrapping_mul(2u64.wrapping_sub(factor.wrapping_mul(inverse)));
            }
            crate::avx_helper!(@dispatch_avx2_fma return unsafe {
                super::mint_fft_convolve::convolve_u64_avx2(a, b, factor, inverse)
            }, ());
            return convolve_u64_fft_scalar(a, b, factor, inverse);
        }
        let mut a = Self::transform(a, len);
        let b = Self::transform(b, len);
        Self::multiply(&mut a, &b);
        Self::inverse_transform(a, len)
    }
}

fn convolve_u64_fft_scalar(a: Vec<u64>, b: Vec<u64>, factor: u64, inverse: u64) -> Vec<u64> {
    fn split(values: &[u64], factor: u64) -> [Vec<i64>; 4] {
        let mut result = std::array::from_fn(|_| Vec::with_capacity(values.len()));
        let mut multiplier = 1u64;
        for &value in values {
            let mut value = value.wrapping_mul(multiplier);
            for part in &mut result {
                let digit = value as i16;
                part.push(digit as i64);
                value = (value >> 16).wrapping_add(u64::from(digit < 0));
            }
            multiplier = multiplier.wrapping_mul(factor);
        }
        result
    }

    let len = a.len() + b.len() - 1;
    let fa = split(&a, factor).map(|a| ConvolveRealFft::transform(a, len));
    drop(a);
    let fb = split(&b, factor).map(|b| ConvolveRealFft::transform(b, len));
    drop(b);
    let values: [Vec<i64>; 4] = std::array::from_fn(|part| {
        let mut sum = fa[0].clone();
        ConvolveRealFft::multiply(&mut sum, &fb[part]);
        for left in 1..=part {
            let mut product = fa[left].clone();
            ConvolveRealFft::multiply(&mut product, &fb[part - left]);
            for (sum, product) in sum.iter_mut().zip(product) {
                *sum += product;
            }
        }
        ConvolveRealFft::inverse_transform(sum, len)
    });
    let mut multiplier = 1u64;
    (0..len)
        .map(|i| {
            let value = (values[0][i] as u64)
                .wrapping_add((values[1][i] as u64) << 16)
                .wrapping_add((values[2][i] as u64) << 32)
                .wrapping_add((values[3][i] as u64) << 48)
                .wrapping_mul(multiplier);
            multiplier = multiplier.wrapping_mul(inverse);
            value
        })
        .collect()
}

pub trait NttReuse: ConvolveSteps {
    const MULTIPLE: bool = true;

    /// Transforms coefficients into the usual NTT frequency order.
    fn transform_ntt(t: Self::T, len: usize) -> Self::F {
        Self::transform(t, len)
    }

    /// Inverts a value produced by `transform_ntt`.
    fn inverse_transform_ntt(f: Self::F, len: usize) -> Self::T {
        Self::inverse_transform(f, len)
    }

    /// Extends a value produced by `transform_ntt` to twice its length.
    fn ntt_doubling(f: Self::F) -> Self::F;

    /// Extracts the even coefficients of `a(x) * b(-x)` in the usual NTT frequency order.
    fn even_mul_normal_neg(f: &Self::F, g: &Self::F) -> Self::F;

    /// Extracts the odd coefficients of `a(x) * b(-x)` in the usual NTT frequency order.
    fn odd_mul_normal_neg(f: &Self::F, g: &Self::F) -> Self::F;

    /// Multiplies a usual NTT transform by the corresponding prefix of another one.
    fn multiply_prefix(f: &mut Self::F, g: &Self::F);

    /// Adds the pointwise product of two usual NTT transforms to `sum`.
    fn multiply_add(sum: &mut Self::F, f: &Self::F, g: &Self::F);
}

thread_local!(
    static BIT_REVERSE: UnsafeCell<Vec<Vec<usize>>> = const { UnsafeCell::new(vec![]) };
);

impl<M> NttReuse for Convolve<M>
where
    M: Montgomery32NttModulus,
{
    const MULTIPLE: bool = false;

    fn transform_ntt(mut t: Self::T, len: usize) -> Self::F {
        t.resize_with(len.max(1).next_power_of_two(), Zero::zero);
        ntt(&mut t);
        t
    }

    fn inverse_transform_ntt(mut f: Self::F, len: usize) -> Self::T {
        intt(&mut f);
        f.truncate(len);
        f
    }

    fn ntt_doubling(mut f: Self::F) -> Self::F {
        let n = f.len();
        let k = n.trailing_zeros() as usize;
        let mut a = Self::inverse_transform_ntt(f.clone(), n);
        let mut rot = MInt::<M>::one();
        let zeta = MInt::<M>::new_unchecked(M::INFO.root[k + 1]);
        for a in a.iter_mut() {
            *a *= rot;
            rot *= zeta;
        }
        f.extend(Self::transform_ntt(a, n));
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
                    v[i] = (v[i >> 1] >> 1) | ((i & 1) << k.saturating_sub(1));
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

    fn multiply_prefix(f: &mut Self::F, g: &Self::F) {
        pointwise_multiply(f, g);
    }

    fn multiply_add(sum: &mut Self::F, f: &Self::F, g: &Self::F) {
        assert!(sum.len() == f.len() && sum.len() == g.len());
        pointwise_multiply_add(sum, f, g);
    }
}

impl<M, N1, N2, N3> NttReuse for Convolve<(M, (N1, N2, N3))>
where
    M: MIntConvert + MIntConvert<u32>,
    N1: Montgomery32NttModulus,
    N2: Montgomery32NttModulus,
    N3: Montgomery32NttModulus,
{
    fn transform_ntt(t: Self::T, len: usize) -> Self::F {
        let npot = len.max(1).next_power_of_two();
        let f = convert_crt_input(t, npot);
        (
            Convolve::<N1>::transform_ntt(f.0, npot),
            Convolve::<N2>::transform_ntt(f.1, npot),
            Convolve::<N3>::transform_ntt(f.2, npot),
        )
    }

    fn inverse_transform_ntt(f: Self::F, len: usize) -> Self::T {
        reconstruct_mint_crt((
            Convolve::<N1>::inverse_transform_ntt(f.0, len),
            Convolve::<N2>::inverse_transform_ntt(f.1, len),
            Convolve::<N3>::inverse_transform_ntt(f.2, len),
        ))
    }

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
                        v[i] = (v[i >> 1] >> 1) | ((i & 1) << k.saturating_sub(1));
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

    fn multiply_prefix(f: &mut Self::F, g: &Self::F) {
        Convolve::<N1>::multiply_prefix(&mut f.0, &g.0);
        Convolve::<N2>::multiply_prefix(&mut f.1, &g.1);
        Convolve::<N3>::multiply_prefix(&mut f.2, &g.2);
    }

    fn multiply_add(sum: &mut Self::F, f: &Self::F, g: &Self::F) {
        Convolve::<N1>::multiply_add(&mut sum.0, &f.0, &g.0);
        Convolve::<N2>::multiply_add(&mut sum.1, &f.1, &g.1);
        Convolve::<N3>::multiply_add(&mut sum.2, &f.2, &g.2);
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
        for _ in 0..1000 {
            let (n, m) = if rng.random(0..100) == 0 {
                let w = rng.random(6..=8);
                ((1usize << w) + 1usize, (1usize << w) + 1usize)
            } else {
                let n = rng.random(0..=5);
                let m = rng.random(0..=5);
                (
                    if n == 5 { rng.random(70..=120) } else { n },
                    if m == 5 { rng.random(70..=120) } else { m },
                )
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
    fn test_convolve_large_ntt() {
        enum Modulo17 {}
        impl MontgomeryReduction32 for Modulo17 {
            const MOD: u32 = 17;
        }
        impl Montgomery32NttModulus for Modulo17 {}

        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.random(101..=300);
            let m = rng.random(101..=300);
            let a: Vec<_> = rng
                .random_iter(0u32..17)
                .take(n)
                .map(MInt::<Modulo17>::from)
                .collect();
            let b = if rng.random(0..2) == 0 {
                a.clone()
            } else {
                rng.random_iter(0u32..17)
                    .take(m)
                    .map(MInt::<Modulo17>::from)
                    .collect()
            };
            assert_eq!(convolve_naive(&a, &b), Convolve::<Modulo17>::convolve(a, b));
        }
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
            let wide = rng.random(0..100) == 0;
            let (n, m) = if wide {
                (rng.random(301..=400), rng.random(301..=400))
            } else {
                let n = rng.random(0..=5);
                let m = rng.random(0..=5);
                (
                    if n == 5 { rng.random(70..=400) } else { n },
                    if m == 5 { rng.random(70..=400) } else { m },
                )
            };
            let a: Vec<u64> = if wide {
                rng.random_iter(..).take(n).collect()
            } else {
                rng.random_iter(0u64..1 << 24).take(n).collect()
            };
            let b: Vec<u64> = if wide {
                rng.random_iter(..).take(m).collect()
            } else {
                rng.random_iter(0u64..1 << 24).take(m).collect()
            };
            let mut c = vec![0u64; (n + m).saturating_sub(1)];
            for i in 0..n {
                for j in 0..m {
                    c[i + j] = c[i + j].wrapping_add(a[i].wrapping_mul(b[j]));
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
            let f = Convolve998244353::transform_ntt(a.clone(), n);

            // doubling
            {
                let f_double = Convolve998244353::ntt_doubling(f.clone());
                let mut a = a.clone();
                a.resize_with(n * 2, Zero::zero);
                assert_eq!(f_double, Convolve998244353::transform_ntt(a, n * 2));
            }

            let f = Convolve998244353::transform_ntt(a.clone(), n * 2);
            let b: Vec<MInt998244353> = rng.random_iter(..).take(n).collect();
            let g = Convolve998244353::transform_ntt(b.clone(), n * 2);
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
                assert_eq!(fg_neg, Convolve998244353::transform_ntt(ab_neg_even, n));
            }

            // odd_mul_normal_neg
            {
                let fg_neg = Convolve998244353::odd_mul_normal_neg(&f, &g);
                let ab_neg_odd: Vec<_> = Convolve998244353::convolve(a.clone(), b_neg.clone())
                    .into_iter()
                    .skip(1)
                    .step_by(2)
                    .collect();
                assert_eq!(fg_neg, Convolve998244353::transform_ntt(ab_neg_odd, n));
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
            let f = MIntConvolve::<Modulo1000000009>::transform_ntt(a.clone(), n);

            // doubling
            {
                let f_double = MIntConvolve::<Modulo1000000009>::ntt_doubling(f.clone());
                let mut a = a.clone();
                a.resize_with(n * 2, Zero::zero);
                assert_eq!(
                    f_double,
                    MIntConvolve::<Modulo1000000009>::transform_ntt(a, n * 2)
                );
            }

            let f = MIntConvolve::<Modulo1000000009>::transform_ntt(a.clone(), n * 2);
            let b: Vec<M> = rng.random_iter(..).take(n).collect();
            let g = MIntConvolve::<Modulo1000000009>::transform_ntt(b.clone(), n * 2);
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
                    MIntConvolve::<Modulo1000000009>::inverse_transform_ntt(fg_neg, n),
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
                    MIntConvolve::<Modulo1000000009>::inverse_transform_ntt(fg_neg, n),
                    ab_neg_odd
                );
            }
        }
    }
}
