use super::*;
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
        let x = a[i];
        a[i] = if x >= M::MOD { x - M::MOD } else { x };
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

#[target_feature(enable = "avx2")]
pub unsafe fn ntt_avx2<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let n = a.len();
    if n <= 1 {
        return;
    }
    let ptr = a.as_mut_ptr() as *mut u32;
    let a = std::slice::from_raw_parts_mut(ptr, n);
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mod2_vec = _mm256_set1_epi32(M::MOD.wrapping_add(M::MOD) as i32);
    let r_vec = _mm256_set1_epi32(M::R as i32);
    let imag = M::INFO.root[2];
    let imag_vec = _mm256_set1_epi32(imag as i32);

    let mut v = n / 2;
    if n.trailing_zeros() & 1 == 1 {
        let mut i = 0;
        while i + 8 <= v {
            let x0 = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
            let x1 = _mm256_loadu_si256(a.as_ptr().add(v + i) as *const __m256i);
            let y0 = add_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            let y1 = sub_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            _mm256_storeu_si256(a.as_mut_ptr().add(i) as *mut __m256i, y0);
            _mm256_storeu_si256(a.as_mut_ptr().add(v + i) as *mut __m256i, y1);
            i += 8;
        }
        while i < v {
            let x0 = a[i];
            let x1 = a[v + i];
            a[i] = M::mod_add(x0, x1);
            a[v + i] = M::mod_sub(x0, x1);
            i += 1;
        }
        v >>= 1;
    }
    while v > 1 {
        let half = v >> 1;
        let mut w1 = M::N1;
        for (s, block) in a.chunks_exact_mut(v << 1).enumerate() {
            let base = block.as_mut_ptr();
            let ll = base;
            let lr = base.add(half);
            let rl = base.add(v);
            let rr = base.add(v + half);

            let w2 = M::mod_mul(w1, w1);
            let w3 = M::mod_mul(w2, w1);
            let w1v = _mm256_set1_epi32(w1 as i32);
            let w2v = _mm256_set1_epi32(w2 as i32);
            let w3v = _mm256_set1_epi32(w3 as i32);

            let mut i = 0;
            while i + 8 <= half {
                let x0 = _mm256_loadu_si256(ll.add(i) as *const __m256i);
                let x1 = _mm256_loadu_si256(lr.add(i) as *const __m256i);
                let x2 = _mm256_loadu_si256(rl.add(i) as *const __m256i);
                let x3 = _mm256_loadu_si256(rr.add(i) as *const __m256i);

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

                _mm256_storeu_si256(ll.add(i) as *mut __m256i, y0);
                _mm256_storeu_si256(lr.add(i) as *mut __m256i, y1);
                _mm256_storeu_si256(rl.add(i) as *mut __m256i, y2);
                _mm256_storeu_si256(rr.add(i) as *mut __m256i, y3);
                i += 8;
            }
            while i < half {
                let a0 = *ll.add(i);
                let a1 = M::mod_mul(*lr.add(i), w1);
                let a2 = M::mod_mul(*rl.add(i), w2);
                let a3 = M::mod_mul(*rr.add(i), w3);
                let a0pa2 = M::mod_add(a0, a2);
                let a0na2 = M::mod_sub(a0, a2);
                let a1pa3 = M::mod_add(a1, a3);
                let a1na3 = M::mod_sub(a1, a3);
                let a1na3imag = M::mod_mul(a1na3, imag);
                *ll.add(i) = M::mod_add(a0pa2, a1pa3);
                *lr.add(i) = M::mod_sub(a0pa2, a1pa3);
                *rl.add(i) = M::mod_add(a0na2, a1na3imag);
                *rr.add(i) = M::mod_sub(a0na2, a1na3imag);
                i += 1;
            }
            w1 = M::mod_mul(w1, M::INFO.rate3[s.trailing_ones() as usize]);
        }
        v >>= 2;
    }
    normalize_avx2::<M>(a);
}

#[target_feature(enable = "avx2")]
pub unsafe fn intt_avx2<M>(a: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let n = a.len();
    if n <= 1 {
        return;
    }
    let ptr = a.as_mut_ptr() as *mut u32;
    let a = std::slice::from_raw_parts_mut(ptr, n);
    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mod2_vec = _mm256_set1_epi32(M::MOD.wrapping_add(M::MOD) as i32);
    let r_vec = _mm256_set1_epi32(M::R as i32);
    let iimag = M::INFO.inv_root[2];
    let iimag_vec = _mm256_set1_epi32(iimag as i32);

    let mut v = 1;
    let limit = if n.trailing_zeros() & 1 == 1 {
        n / 2
    } else {
        n
    };
    while v < limit {
        let mut w1 = M::N1;
        for (s, block) in a.chunks_exact_mut(v << 2).enumerate() {
            let base = block.as_mut_ptr();
            let ll = base;
            let lr = base.add(v);
            let rl = base.add(v << 1);
            let rr = base.add(v * 3);

            let w2 = M::mod_mul(w1, w1);
            let w3 = M::mod_mul(w2, w1);
            let w1v = _mm256_set1_epi32(w1 as i32);
            let w2v = _mm256_set1_epi32(w2 as i32);
            let w3v = _mm256_set1_epi32(w3 as i32);

            let mut i = 0;
            while i + 8 <= v {
                let x0 = _mm256_loadu_si256(ll.add(i) as *const __m256i);
                let x1 = _mm256_loadu_si256(lr.add(i) as *const __m256i);
                let x2 = _mm256_loadu_si256(rl.add(i) as *const __m256i);
                let x3 = _mm256_loadu_si256(rr.add(i) as *const __m256i);

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

                _mm256_storeu_si256(ll.add(i) as *mut __m256i, y0);
                _mm256_storeu_si256(lr.add(i) as *mut __m256i, y1);
                _mm256_storeu_si256(rl.add(i) as *mut __m256i, y2);
                _mm256_storeu_si256(rr.add(i) as *mut __m256i, y3);
                i += 8;
            }
            while i < v {
                let a0 = *ll.add(i);
                let a1 = *lr.add(i);
                let a2 = *rl.add(i);
                let a3 = *rr.add(i);
                let a0pa1 = M::mod_add(a0, a1);
                let a0na1 = M::mod_sub(a0, a1);
                let a2pa3 = M::mod_add(a2, a3);
                let a2na3iimag = M::mod_mul(M::mod_sub(a2, a3), iimag);
                *ll.add(i) = M::mod_add(a0pa1, a2pa3);
                *lr.add(i) = M::mod_mul(M::mod_add(a0na1, a2na3iimag), w1);
                *rl.add(i) = M::mod_mul(M::mod_sub(a0pa1, a2pa3), w2);
                *rr.add(i) = M::mod_mul(M::mod_sub(a0na1, a2na3iimag), w3);
                i += 1;
            }
            w1 = M::mod_mul(w1, M::INFO.inv_rate3[s.trailing_ones() as usize]);
        }
        v <<= 2;
    }
    if n.trailing_zeros() & 1 == 1 {
        let half = n >> 1;
        let mut i = 0;
        while i + 8 <= half {
            let x0 = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
            let x1 = _mm256_loadu_si256(a.as_ptr().add(half + i) as *const __m256i);
            let y0 = add_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            let y1 = sub_vec_avx2::<M>(x0, x1, mod_vec, mod2_vec);
            _mm256_storeu_si256(a.as_mut_ptr().add(i) as *mut __m256i, y0);
            _mm256_storeu_si256(a.as_mut_ptr().add(half + i) as *mut __m256i, y1);
            i += 8;
        }
        while i < half {
            let x0 = a[i];
            let x1 = a[half + i];
            a[i] = M::mod_add(x0, x1);
            a[half + i] = M::mod_sub(x0, x1);
            i += 1;
        }
    }
    let inv = M::mod_inv(<M as MIntConvert<u32>>::from(n as u32));
    let inv_vec = _mm256_set1_epi32(inv as i32);
    let mut i = 0;
    while i + 8 <= n {
        let x = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
        let y = simd32::montgomery_mul_256_canon(x, inv_vec, r_vec, mod_vec);
        _mm256_storeu_si256(a.as_mut_ptr().add(i) as *mut __m256i, y);
        i += 8;
    }
    while i < n {
        a[i] = M::mod_mul(a[i], inv);
        i += 1;
    }
}
