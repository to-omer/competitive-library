use super::*;

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn load_ntt_avx2(p: *const u32, step: usize) -> __m256i {
    if step == 4 {
        _mm256_castsi128_si256(_mm_loadu_si128(p.cast()))
    } else {
        _mm256_loadu_si256(p.cast())
    }
}
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store_ntt_avx2(p: *mut u32, value: __m256i, step: usize) {
    if step == 4 {
        _mm_storeu_si128(p.cast(), _mm256_castsi256_si128(value));
    } else {
        _mm256_storeu_si256(p.cast(), value);
    }
}

#[target_feature(enable = "avx2")]
pub unsafe fn ntt_four_avx2<M, const INVERSE: bool>(a: &mut [u32])
where
    M: Montgomery32NttModulus,
{
    let roots = if INVERSE {
        &M::INFO.inv_root
    } else {
        &M::INFO.root
    };
    let rates = &const {
        let info = M::INFO;
        let (roots, rates) = if INVERSE {
            (info.inv_root, info.inv_rate3_packed)
        } else {
            (info.root, info.rate3_packed)
        };
        let mut result = [[0u32; 4]; 32];
        let mut i = 0;
        while i < 32 {
            let r = mod_mul(rates[i][1], roots[3], M::MOD, M::R);
            let r2 = mod_mul(r, r, M::MOD, M::R);
            result[i] = [M::N1, r, r2, mod_mul(r2, r, M::MOD, M::R)];
            i += 1;
        }
        result
    };
    let one = _mm256_set1_epi32(M::N1 as i32);
    let modulus = _mm256_set1_epi32(M::MOD as i32);
    let modulus2 = _mm256_set1_epi32((M::MOD * 2) as i32);
    let r = _mm256_set1_epi32(M::R as i32);
    let root3 = M::mod_mul(roots[2], roots[3]) as i32;
    let step = _mm256_setr_epi32(
        M::N1 as i32,
        roots[3] as i32,
        roots[2] as i32,
        root3,
        M::N1 as i32,
        roots[3] as i32,
        roots[2] as i32,
        root3,
    );
    let mut twiddle = _mm256_blend_epi32::<0xf0>(one, step);
    let imag = if INVERSE {
        _mm256_setr_epi32(
            M::N1 as i32,
            roots[2] as i32,
            M::N1 as i32,
            roots[2] as i32,
            M::N1 as i32,
            roots[2] as i32,
            M::N1 as i32,
            roots[2] as i32,
        )
    } else {
        _mm256_setr_epi32(
            M::N1 as i32,
            M::N1 as i32,
            roots[2] as i32,
            roots[2] as i32,
            M::N1 as i32,
            M::N1 as i32,
            roots[2] as i32,
            roots[2] as i32,
        )
    };
    for (s, a) in a.as_chunks_mut::<8>().0.iter_mut().enumerate() {
        let mut x = _mm256_loadu_si256(a.as_ptr().cast());
        if !INVERSE {
            x = simd32::montgomery_mul_256(x, twiddle, r, modulus);
        }
        let pair = if INVERSE {
            let y = _mm256_shuffle_epi32::<0xb1>(x);
            let sum = simd32::montgomery_add_256(x, y, modulus2);
            let diff = simd32::montgomery_sub_256(x, y, modulus2);
            let sum = _mm256_shuffle_epi32::<0x88>(sum);
            let diff = _mm256_shuffle_epi32::<0x88>(diff);
            let diff = simd32::montgomery_mul_256(diff, imag, r, modulus);
            _mm256_unpacklo_epi64(sum, diff)
        } else {
            let y = _mm256_shuffle_epi32::<0x4e>(x);
            let sum = simd32::montgomery_add_256(x, y, modulus2);
            let diff = simd32::montgomery_sub_256(x, y, modulus2);
            _mm256_unpacklo_epi64(sum, diff)
        };
        let left = _mm256_shuffle_epi32::<0xa0>(pair);
        let mut right = _mm256_shuffle_epi32::<0xf5>(pair);
        if !INVERSE {
            right = simd32::montgomery_mul_256(right, imag, r, modulus);
        }
        let sum = simd32::montgomery_add_256(left, right, modulus2);
        let diff = simd32::montgomery_sub_256(left, right, modulus2);
        let mut value = _mm256_blend_epi32::<0xaa>(sum, diff);
        if INVERSE {
            value = _mm256_shuffle_epi32::<0xd8>(value);
            value = simd32::montgomery_mul_256(value, twiddle, r, modulus);
        }
        _mm256_storeu_si256(a.as_mut_ptr().cast(), value);
        let rate = _mm256_broadcastsi128_si256(_mm_loadu_si128(
            rates[s.trailing_ones() as usize + 1].as_ptr().cast(),
        ));
        twiddle = simd32::montgomery_mul_256(twiddle, rate, r, modulus);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn normalize_avx2<M>(a: &mut [u32])
where
    M: Montgomery32NttModulus,
{
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mut i = 0;
    while i + 8 <= a.len() {
        let x = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
        let y = _mm256_min_epu32(x, _mm256_sub_epi32(x, mod_vec));
        _mm256_storeu_si256(a.as_mut_ptr().add(i) as *mut __m256i, y);
        i += 8;
    }
    while i < a.len() {
        a[i] = normalize_scalar::<M>(a[i]);
        i += 1;
    }
}

pub unsafe fn add_vec_avx2<M>(
    a: __m256i,
    b: __m256i,
    mod_vec: __m256i,
    mod2_vec: __m256i,
) -> __m256i
where
    M: Montgomery32NttModulus,
{
    if M::MOD < LAZY_THRESHOLD {
        simd32::montgomery_add_256(a, b, mod2_vec)
    } else {
        simd32::add_mod_256(a, b, mod_vec)
    }
}

pub unsafe fn sub_vec_avx2<M>(
    a: __m256i,
    b: __m256i,
    mod_vec: __m256i,
    mod2_vec: __m256i,
) -> __m256i
where
    M: Montgomery32NttModulus,
{
    if M::MOD < LAZY_THRESHOLD {
        simd32::montgomery_sub_256(a, b, mod2_vec)
    } else {
        simd32::sub_mod_256(a, b, mod_vec)
    }
}

unsafe fn mul_vec_avx2<M>(a: __m256i, b: __m256i, r_vec: __m256i, mod_vec: __m256i) -> __m256i
where
    M: Montgomery32NttModulus,
{
    if M::MOD < LAZY_THRESHOLD {
        simd32::montgomery_mul_256(a, b, r_vec, mod_vec)
    } else {
        simd32::montgomery_mul_256_canon(a, b, r_vec, mod_vec)
    }
}

#[target_feature(enable = "avx2")]
pub unsafe fn pointwise_multiply_avx2<M>(f: &mut [MInt<M>], g: &[MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let r_vec = _mm256_set1_epi32(M::R as i32);
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mut i = 0;
    while i + 8 <= f.len() {
        let a = _mm256_loadu_si256(f.as_ptr().add(i) as *const __m256i);
        let b = _mm256_loadu_si256(g.as_ptr().add(i) as *const __m256i);
        let x = simd32::montgomery_mul_256_canon(a, b, r_vec, mod_vec);
        _mm256_storeu_si256(f.as_mut_ptr().add(i) as *mut __m256i, x);
        i += 8;
    }
    while i < f.len() {
        f[i] *= g[i];
        i += 1;
    }
}

#[target_feature(enable = "avx2")]
pub unsafe fn pointwise_multiply_add_avx2<M>(sum: &mut [MInt<M>], f: &[MInt<M>], g: &[MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let r_vec = _mm256_set1_epi32(M::R as i32);
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mut i = 0;
    while i + 8 <= sum.len() {
        let s = _mm256_loadu_si256(sum.as_ptr().add(i).cast());
        let f = _mm256_loadu_si256(f.as_ptr().add(i).cast());
        let g = _mm256_loadu_si256(g.as_ptr().add(i).cast());
        let product = simd32::montgomery_mul_256_canon(f, g, r_vec, mod_vec);
        _mm256_storeu_si256(
            sum.as_mut_ptr().add(i).cast(),
            simd32::add_mod_256(s, product, mod_vec),
        );
        i += 8;
    }
    while i < sum.len() {
        sum[i] += f[i] * g[i];
        i += 1;
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn ntt_batch_avx2<M, const PARTIAL: bool>(a: &mut [MInt<M>], width: usize)
where
    M: Montgomery32NttModulus,
{
    let n = a.len() / width;
    if n <= 1 {
        return;
    }
    let ptr = a.as_mut_ptr() as *mut u32;
    let a = std::slice::from_raw_parts_mut(ptr, a.len());
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mod2_vec = _mm256_set1_epi32(M::MOD.wrapping_add(M::MOD) as i32);
    let r_vec = _mm256_set1_epi32(M::R as i32);
    let imag = M::INFO.root[2];
    let imag_vec = _mm256_set1_epi32(imag as i32);

    let mut v = n / 2;
    if n.trailing_zeros() & 1 == 1 {
        let half = v * width;
        let step = if PARTIAL && half == 4 { 4 } else { 8 };
        let mut i = 0;
        while i + step <= half {
            let x0 = load_ntt_avx2(a.as_ptr().add(i), step);
            let x1 = load_ntt_avx2(a.as_ptr().add(half + i), step);
            let y0 = add_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            let y1 = sub_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            store_ntt_avx2(a.as_mut_ptr().add(i), y0, step);
            store_ntt_avx2(a.as_mut_ptr().add(half + i), y1, step);
            i += step;
        }
        while i < half {
            let x0 = a[i];
            let x1 = a[half + i];
            a[i] = add_scalar::<M>(x0, x1);
            a[half + i] = sub_scalar::<M>(x0, x1);
            i += 1;
        }
        v >>= 1;
    }
    while v > 1 {
        if width == 1 && v == 2 && a.len() >= 8 && M::MOD < LAZY_THRESHOLD {
            ntt_four_avx2::<M, false>(a);
            break;
        }
        let half = (v >> 1) * width;
        let step = if PARTIAL && half == 4 { 4 } else { 8 };
        let mut w1 = M::N1;
        let mut w2 = w1;
        let mut w3 = w1;
        for (s, block) in a.chunks_exact_mut((v << 1) * width).enumerate() {
            let base = block.as_mut_ptr();
            let ll = base;
            let lr = base.add(half);
            let rl = base.add(v * width);
            let rr = base.add(v * width + half);

            let w1v = _mm256_set1_epi32(w1 as i32);
            let w2v = _mm256_set1_epi32(w2 as i32);
            let w3v = _mm256_set1_epi32(w3 as i32);

            let mut i = 0;
            while i + step <= half {
                let x0 = load_ntt_avx2(ll.add(i), step);
                let x1 = load_ntt_avx2(lr.add(i), step);
                let x2 = load_ntt_avx2(rl.add(i), step);
                let x3 = load_ntt_avx2(rr.add(i), step);

                let (a1, a2, a3) = if s == 0 {
                    (x1, x2, x3)
                } else {
                    (
                        mul_vec_avx2::<M>(x1, w1v, r_vec, mod_vec),
                        mul_vec_avx2::<M>(x2, w2v, r_vec, mod_vec),
                        mul_vec_avx2::<M>(x3, w3v, r_vec, mod_vec),
                    )
                };

                let a0pa2 = add_vec_avx2::<M>(x0, a2, mod_vec, mod2_vec);
                let a0na2 = sub_vec_avx2::<M>(x0, a2, mod_vec, mod2_vec);
                let a1pa3 = add_vec_avx2::<M>(a1, a3, mod_vec, mod2_vec);
                let a1na3 = sub_vec_avx2::<M>(a1, a3, mod_vec, mod2_vec);
                let a1na3imag = mul_vec_avx2::<M>(a1na3, imag_vec, r_vec, mod_vec);

                let y0 = add_vec_avx2::<M>(a0pa2, a1pa3, mod_vec, mod2_vec);
                let y1 = sub_vec_avx2::<M>(a0pa2, a1pa3, mod_vec, mod2_vec);
                let y2 = add_vec_avx2::<M>(a0na2, a1na3imag, mod_vec, mod2_vec);
                let y3 = sub_vec_avx2::<M>(a0na2, a1na3imag, mod_vec, mod2_vec);

                store_ntt_avx2(ll.add(i), y0, step);
                store_ntt_avx2(lr.add(i), y1, step);
                store_ntt_avx2(rl.add(i), y2, step);
                store_ntt_avx2(rr.add(i), y3, step);
                i += step;
            }
            while i < half {
                let a0 = *ll.add(i);
                let a1 = mul_scalar::<M>(*lr.add(i), w1);
                let a2 = mul_scalar::<M>(*rl.add(i), w2);
                let a3 = mul_scalar::<M>(*rr.add(i), w3);
                let a0pa2 = add_scalar::<M>(a0, a2);
                let a0na2 = sub_scalar::<M>(a0, a2);
                let a1pa3 = add_scalar::<M>(a1, a3);
                let a1na3 = sub_scalar::<M>(a1, a3);
                let a1na3imag = mul_scalar::<M>(a1na3, imag);
                *ll.add(i) = add_scalar::<M>(a0pa2, a1pa3);
                *lr.add(i) = sub_scalar::<M>(a0pa2, a1pa3);
                *rl.add(i) = add_scalar::<M>(a0na2, a1na3imag);
                *rr.add(i) = sub_scalar::<M>(a0na2, a1na3imag);
                i += 1;
            }
            let rate = &M::INFO.rate3_packed[s.trailing_ones() as usize];
            w1 = M::mod_mul(w1, rate[1]);
            w2 = M::mod_mul(w2, rate[3]);
            w3 = M::mod_mul(w3, rate[5]);
        }
        v >>= 2;
    }
    normalize_avx2::<M>(a);
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn intt_batch_avx2<M, const PARTIAL: bool>(a: &mut [MInt<M>], width: usize)
where
    M: Montgomery32NttModulus,
{
    let n = a.len() / width;
    if n <= 1 {
        return;
    }
    let ptr = a.as_mut_ptr() as *mut u32;
    let a = std::slice::from_raw_parts_mut(ptr, a.len());
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mod2_vec = _mm256_set1_epi32(M::MOD.wrapping_add(M::MOD) as i32);
    let r_vec = _mm256_set1_epi32(M::R as i32);
    let iimag = M::INFO.inv_root[2];
    let iimag_vec = _mm256_set1_epi32(iimag as i32);

    let mut v = 1;
    if width == 1 && a.len() >= 8 && M::MOD < LAZY_THRESHOLD {
        ntt_four_avx2::<M, true>(a);
        v = 4;
    }
    let limit = if n.trailing_zeros() & 1 == 1 {
        n / 2
    } else {
        n
    };
    while v < limit {
        let quarter = v * width;
        let step = if PARTIAL && quarter == 4 { 4 } else { 8 };
        let mut w1 = M::N1;
        let mut w2 = w1;
        let mut w3 = w1;
        for (s, block) in a.chunks_exact_mut((v << 2) * width).enumerate() {
            let base = block.as_mut_ptr();
            let ll = base;
            let lr = base.add(quarter);
            let rl = base.add(quarter * 2);
            let rr = base.add(quarter * 3);

            let w1v = _mm256_set1_epi32(w1 as i32);
            let w2v = _mm256_set1_epi32(w2 as i32);
            let w3v = _mm256_set1_epi32(w3 as i32);

            let mut i = 0;
            while i + step <= quarter {
                let x0 = load_ntt_avx2(ll.add(i), step);
                let x1 = load_ntt_avx2(lr.add(i), step);
                let x2 = load_ntt_avx2(rl.add(i), step);
                let x3 = load_ntt_avx2(rr.add(i), step);

                let a0pa1 = add_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
                let a0na1 = sub_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
                let a2pa3 = add_vec_avx2::<M>(x2, x3, mod_vec, mod2_vec);
                let a2na3 = sub_vec_avx2::<M>(x2, x3, mod_vec, mod2_vec);
                let a2na3iimag = mul_vec_avx2::<M>(a2na3, iimag_vec, r_vec, mod_vec);

                let y0 = add_vec_avx2::<M>(a0pa1, a2pa3, mod_vec, mod2_vec);
                let y1 = add_vec_avx2::<M>(a0na1, a2na3iimag, mod_vec, mod2_vec);
                let y2 = sub_vec_avx2::<M>(a0pa1, a2pa3, mod_vec, mod2_vec);
                let y3 = sub_vec_avx2::<M>(a0na1, a2na3iimag, mod_vec, mod2_vec);

                let (y1, y2, y3) = if s == 0 {
                    (y1, y2, y3)
                } else {
                    (
                        mul_vec_avx2::<M>(y1, w1v, r_vec, mod_vec),
                        mul_vec_avx2::<M>(y2, w2v, r_vec, mod_vec),
                        mul_vec_avx2::<M>(y3, w3v, r_vec, mod_vec),
                    )
                };

                store_ntt_avx2(ll.add(i), y0, step);
                store_ntt_avx2(lr.add(i), y1, step);
                store_ntt_avx2(rl.add(i), y2, step);
                store_ntt_avx2(rr.add(i), y3, step);
                i += step;
            }
            while i < quarter {
                let a0 = *ll.add(i);
                let a1 = *lr.add(i);
                let a2 = *rl.add(i);
                let a3 = *rr.add(i);
                let a0pa1 = add_scalar::<M>(a0, a1);
                let a0na1 = sub_scalar::<M>(a0, a1);
                let a2pa3 = add_scalar::<M>(a2, a3);
                let a2na3iimag = mul_scalar::<M>(sub_scalar::<M>(a2, a3), iimag);
                *ll.add(i) = add_scalar::<M>(a0pa1, a2pa3);
                *lr.add(i) = mul_scalar::<M>(add_scalar::<M>(a0na1, a2na3iimag), w1);
                *rl.add(i) = mul_scalar::<M>(sub_scalar::<M>(a0pa1, a2pa3), w2);
                *rr.add(i) = mul_scalar::<M>(sub_scalar::<M>(a0na1, a2na3iimag), w3);
                i += 1;
            }
            let rate = &M::INFO.inv_rate3_packed[s.trailing_ones() as usize];
            w1 = M::mod_mul(w1, rate[1]);
            w2 = M::mod_mul(w2, rate[3]);
            w3 = M::mod_mul(w3, rate[5]);
        }
        v <<= 2;
    }
    if n.trailing_zeros() & 1 == 1 {
        let half = (n >> 1) * width;
        let step = if PARTIAL && half == 4 { 4 } else { 8 };
        let mut i = 0;
        while i + step <= half {
            let x0 = load_ntt_avx2(a.as_ptr().add(i), step);
            let x1 = load_ntt_avx2(a.as_ptr().add(half + i), step);
            let y0 = add_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            let y1 = sub_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            store_ntt_avx2(a.as_mut_ptr().add(i), y0, step);
            store_ntt_avx2(a.as_mut_ptr().add(half + i), y1, step);
            i += step;
        }
        while i < half {
            let x0 = a[i];
            let x1 = a[half + i];
            a[i] = add_scalar::<M>(x0, x1);
            a[half + i] = sub_scalar::<M>(x0, x1);
            i += 1;
        }
    }
    let inv = M::mod_inv(<M as MIntConvert<u32>>::from(n as u32));
    let inv_vec = _mm256_set1_epi32(inv as i32);
    let mut i = 0;
    while i + 8 <= a.len() {
        let x = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
        let y = simd32::montgomery_mul_256_canon(x, inv_vec, r_vec, mod_vec);
        _mm256_storeu_si256(a.as_mut_ptr().add(i) as *mut __m256i, y);
        i += 8;
    }
    while i < a.len() {
        a[i] = M::mod_mul(a[i], inv);
        i += 1;
    }
}
