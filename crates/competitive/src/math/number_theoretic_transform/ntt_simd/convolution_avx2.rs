use super::ntt_avx2::{add_vec_avx2, sub_vec_avx2};
use super::*;

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn load_block_avx2(a: *const u32, i: usize) -> __m256i {
    _mm256_loadu_si256(a.add(i << 3) as *const __m256i)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store_block_avx2(a: *mut u32, i: usize, x: __m256i) {
    _mm256_storeu_si256(a.add(i << 3) as *mut __m256i, x);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn ntt_block_stage_avx2<M>(
    a: *mut u32,
    n: usize,
    v: usize,
    mut w: [u32; 3],
    mod_vec: __m256i,
    mod2_vec: __m256i,
    imag_vec: __m256i,
) where
    M: Montgomery32NttModulus,
{
    let half = v >> 1;
    let mut base = 0;
    let mut s = 0usize;
    while base < n {
        let w1v = _mm256_set1_epi32(w[0] as i32);
        let w2v = _mm256_set1_epi32(w[1] as i32);
        let w3v = _mm256_set1_epi32(w[2] as i32);
        let w1r = _mm256_set1_epi32(w[0].wrapping_mul(M::R) as i32);
        let w2r = _mm256_set1_epi32(w[1].wrapping_mul(M::R) as i32);
        let w3r = _mm256_set1_epi32(w[2].wrapping_mul(M::R) as i32);
        let mut i = 0;
        while i < half {
            let x0 = load_block_avx2(a, base + i);
            let x1 = load_block_avx2(a, base + half + i);
            let x2 = load_block_avx2(a, base + v + i);
            let x3 = load_block_avx2(a, base + v + half + i);
            let (a1, a2, a3) = if s == 0 {
                (x1, x2, x3)
            } else {
                (
                    simd32::montgomery_mul_256_fixed(x1, w1v, w1r, mod_vec),
                    simd32::montgomery_mul_256_fixed(x2, w2v, w2r, mod_vec),
                    simd32::montgomery_mul_256_fixed(x3, w3v, w3r, mod_vec),
                )
            };
            let a0pa2 = add_vec_avx2::<M>(x0, a2, mod_vec, mod2_vec);
            let a0na2 = sub_vec_avx2::<M>(x0, a2, mod_vec, mod2_vec);
            let a1pa3 = add_vec_avx2::<M>(a1, a3, mod_vec, mod2_vec);
            let a1na3 = sub_vec_avx2::<M>(a1, a3, mod_vec, mod2_vec);
            let a1na3imag = simd32::montgomery_mul_256_fixed(
                a1na3,
                imag_vec,
                _mm256_set1_epi32(M::INFO.root[2].wrapping_mul(M::R) as i32),
                mod_vec,
            );
            store_block_avx2(
                a,
                base + i,
                add_vec_avx2::<M>(a0pa2, a1pa3, mod_vec, mod2_vec),
            );
            store_block_avx2(
                a,
                base + half + i,
                sub_vec_avx2::<M>(a0pa2, a1pa3, mod_vec, mod2_vec),
            );
            store_block_avx2(
                a,
                base + v + i,
                add_vec_avx2::<M>(a0na2, a1na3imag, mod_vec, mod2_vec),
            );
            store_block_avx2(
                a,
                base + v + half + i,
                sub_vec_avx2::<M>(a0na2, a1na3imag, mod_vec, mod2_vec),
            );
            i += 1;
        }
        let k = s.trailing_ones() as usize;
        w[0] = M::mod_mul(w[0], M::INFO.rate3[k]);
        w[1] = M::mod_mul(w[1], M::INFO.rate3_2[k]);
        w[2] = M::mod_mul(w[2], M::INFO.rate3_3[k]);
        s += 1;
        base += v << 1;
    }
}

#[target_feature(enable = "avx2")]
unsafe fn ntt_blocks_avx2<M>(a: *mut u32, n: usize)
where
    M: Montgomery32NttModulus,
{
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mod2_vec = _mm256_set1_epi32(M::MOD.wrapping_add(M::MOD) as i32);
    let imag_vec = _mm256_set1_epi32(M::INFO.root[2] as i32);

    let mut v = n >> 1;
    if n.trailing_zeros() & 1 == 1 {
        let mut i = 0;
        while i < v {
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, v + i);
            store_block_avx2(a, i, add_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec));
            store_block_avx2(a, v + i, sub_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec));
            i += 1;
        }
        v >>= 1;
    }
    while v > 1 {
        ntt_block_stage_avx2::<M>(a, n, v, [M::N1; 3], mod_vec, mod2_vec, imag_vec);
        v >>= 2;
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn intt_block_stage_avx2<M>(
    a: *mut u32,
    n: usize,
    v: usize,
    mut w: [u32; 3],
    mod_vec: __m256i,
    mod2_vec: __m256i,
    iimag_vec: __m256i,
    scale: u32,
) where
    M: Montgomery32NttModulus,
{
    let mut base = 0;
    let mut s = 0usize;
    while base < n {
        let w1v = _mm256_set1_epi32(w[0] as i32);
        let w2v = _mm256_set1_epi32(w[1] as i32);
        let w3v = _mm256_set1_epi32(w[2] as i32);
        let w1r = _mm256_set1_epi32(w[0].wrapping_mul(M::R) as i32);
        let w2r = _mm256_set1_epi32(w[1].wrapping_mul(M::R) as i32);
        let w3r = _mm256_set1_epi32(w[2].wrapping_mul(M::R) as i32);
        let mut i = 0;
        while i < v {
            let x0 = load_block_avx2(a, base + i);
            let x1 = load_block_avx2(a, base + v + i);
            let x2 = load_block_avx2(a, base + (v << 1) + i);
            let x3 = load_block_avx2(a, base + v * 3 + i);
            let a0pa1 = add_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            let a0na1 = sub_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            let a2pa3 = add_vec_avx2::<M>(x2, x3, mod_vec, mod2_vec);
            let a2na3 = sub_vec_avx2::<M>(x2, x3, mod_vec, mod2_vec);
            let a2na3iimag = simd32::montgomery_mul_256_fixed(
                a2na3,
                iimag_vec,
                _mm256_set1_epi32(M::INFO.inv_root[2].wrapping_mul(M::R) as i32),
                mod_vec,
            );
            let y0 = add_vec_avx2::<M>(a0pa1, a2pa3, mod_vec, mod2_vec);
            let y1 = add_vec_avx2::<M>(a0na1, a2na3iimag, mod_vec, mod2_vec);
            let y2 = sub_vec_avx2::<M>(a0pa1, a2pa3, mod_vec, mod2_vec);
            let y3 = sub_vec_avx2::<M>(a0na1, a2na3iimag, mod_vec, mod2_vec);
            let (y0, y1, y2, y3) = if v == 1 {
                (
                    simd32::montgomery_mul_256_fixed(
                        y0,
                        _mm256_set1_epi32(scale as i32),
                        _mm256_set1_epi32(scale.wrapping_mul(M::R) as i32),
                        mod_vec,
                    ),
                    simd32::montgomery_mul_256_fixed(y1, w1v, w1r, mod_vec),
                    simd32::montgomery_mul_256_fixed(y2, w2v, w2r, mod_vec),
                    simd32::montgomery_mul_256_fixed(y3, w3v, w3r, mod_vec),
                )
            } else if s == 0 {
                (y0, y1, y2, y3)
            } else {
                (
                    y0,
                    simd32::montgomery_mul_256_fixed(y1, w1v, w1r, mod_vec),
                    simd32::montgomery_mul_256_fixed(y2, w2v, w2r, mod_vec),
                    simd32::montgomery_mul_256_fixed(y3, w3v, w3r, mod_vec),
                )
            };
            store_block_avx2(a, base + i, y0);
            store_block_avx2(a, base + v + i, y1);
            store_block_avx2(a, base + (v << 1) + i, y2);
            store_block_avx2(a, base + v * 3 + i, y3);
            i += 1;
        }
        let k = s.trailing_ones() as usize;
        w[0] = M::mod_mul(w[0], M::INFO.inv_rate3[k]);
        w[1] = M::mod_mul(w[1], M::INFO.inv_rate3_2[k]);
        w[2] = M::mod_mul(w[2], M::INFO.inv_rate3_3[k]);
        s += 1;
        base += v << 2;
    }
}

#[target_feature(enable = "avx2")]
unsafe fn intt_blocks_avx2<M>(a: *mut u32, n: usize)
where
    M: Montgomery32NttModulus,
{
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mod2_vec = _mm256_set1_epi32(M::MOD.wrapping_add(M::MOD) as i32);
    let iimag_vec = _mm256_set1_epi32(M::INFO.inv_root[2] as i32);
    let limit = if n.trailing_zeros() & 1 == 1 {
        n >> 1
    } else {
        n
    };
    let inv = M::mod_inv(<M as MIntConvert<u32>>::from(n as u32));

    let mut v = 1;
    while v < limit {
        intt_block_stage_avx2::<M>(
            a,
            n,
            v,
            if v == 1 { [inv; 3] } else { [M::N1; 3] },
            mod_vec,
            mod2_vec,
            iimag_vec,
            inv,
        );
        v <<= 2;
    }
    if n.trailing_zeros() & 1 == 1 {
        let half = n >> 1;
        let mut i = 0;
        while i < half {
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, half + i);
            let y0 = add_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            let y1 = sub_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            store_block_avx2(a, i, y0);
            store_block_avx2(a, half + i, y1);
            i += 1;
        }
    }
    let mut i = 0;
    while i < n {
        let x = load_block_avx2(a, i);
        store_block_avx2(a, i, _mm256_min_epu32(x, _mm256_sub_epi32(x, mod_vec)));
        i += 1;
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn reduce_sum_avx2(
    even: __m256i,
    odd: __m256i,
    r_vec: __m256i,
    mod_vec: __m256i,
) -> __m256i {
    let even_m = _mm256_mul_epu32(even, r_vec);
    let odd_m = _mm256_mul_epu32(odd, r_vec);
    let even = _mm256_add_epi64(even, _mm256_mul_epu32(even_m, mod_vec));
    let odd = _mm256_add_epi64(odd, _mm256_mul_epu32(odd_m, mod_vec));
    _mm256_or_si256(_mm256_bsrli_epi128::<4>(even), odd)
}

#[target_feature(enable = "avx2")]
unsafe fn convolve_8_avx2<M>(f: *mut u32, g: *mut u32, n: usize)
where
    M: Montgomery32NttModulus,
{
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mod2_vec = _mm256_set1_epi32(M::MOD.wrapping_add(M::MOD) as i32);
    let r_vec = _mm256_set1_epi32(M::R as i32);
    let mut rr = M::N1;
    let mut i = 0;
    while i < n {
        let rr_i = M::mod_mul(rr, M::INFO.root[2]);
        let mut a = [[0u32; 16]; 4];
        let mut b = [[0u32; 8]; 4];
        for (j, ww) in [rr, M::MOD - rr, rr_i, M::MOD - rr_i]
            .into_iter()
            .enumerate()
        {
            let k = i + j;
            let ff = load_block_avx2(f, k);
            let gg = load_block_avx2(g, k);
            let ff = _mm256_min_epu32(ff, _mm256_sub_epi32(ff, mod_vec));
            let gg = _mm256_min_epu32(gg, _mm256_sub_epi32(gg, mod_vec));
            // fw < 5 * MOD / 4, so the eight-product sum still reduces below 7 * MOD / 2.
            let fw = simd32::montgomery_mul_256_fixed(
                ff,
                _mm256_set1_epi32(ww as i32),
                _mm256_set1_epi32(ww.wrapping_mul(M::R) as i32),
                mod_vec,
            );
            _mm256_storeu_si256(a[j].as_mut_ptr() as *mut __m256i, fw);
            _mm256_storeu_si256(a[j].as_mut_ptr().add(8) as *mut __m256i, ff);
            _mm256_storeu_si256(b[j].as_mut_ptr() as *mut __m256i, gg);
        }
        let mut even = [_mm256_setzero_si256(); 4];
        let mut odd = [_mm256_setzero_si256(); 4];
        let mut l = 0;
        while l < 8 {
            let mut j = 0;
            while j < 4 {
                let x = _mm256_loadu_si256(a[j].as_ptr().add(8 - l) as *const __m256i);
                let y = _mm256_set1_epi32(b[j][l] as i32);
                even[j] = _mm256_add_epi64(even[j], _mm256_mul_epu32(x, y));
                odd[j] = _mm256_add_epi64(odd[j], _mm256_mul_epu32(_mm256_bsrli_epi128::<4>(x), y));
                j += 1;
            }
            l += 1;
        }
        let mut j = 0;
        while j < 4 {
            let x = reduce_sum_avx2(even[j], odd[j], r_vec, mod_vec);
            store_block_avx2(f, i + j, _mm256_min_epu32(x, _mm256_sub_epi32(x, mod2_vec)));
            j += 1;
        }
        i += 4;
        rr = M::mod_mul(rr, M::INFO.rate3[(i >> 2).trailing_zeros() as usize]);
    }
}

#[target_feature(enable = "avx2")]
pub(in super::super) unsafe fn convolve_blocks_avx2<M>(f: &mut [MInt<M>], g: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let n = f.len() >> 3;
    let f = f.as_mut_ptr() as *mut u32;
    let g = g.as_mut_ptr() as *mut u32;
    ntt_blocks_avx2::<M>(f, n);
    ntt_blocks_avx2::<M>(g, n);
    convolve_8_avx2::<M>(f, g, n);
    intt_blocks_avx2::<M>(f, n);
}
