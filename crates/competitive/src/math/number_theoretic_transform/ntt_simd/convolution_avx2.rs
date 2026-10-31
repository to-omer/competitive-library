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
unsafe fn shrink_avx2(x: __m256i, modulus: __m256i) -> __m256i {
    _mm256_min_epu32(x, _mm256_sub_epi32(x, modulus))
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn normalize_avx2(x: __m256i, modulus: __m256i, modulus2: __m256i) -> __m256i {
    shrink_avx2(shrink_avx2(x, modulus2), modulus)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn lazy_sub_avx2(x: __m256i, y: __m256i, modulus: __m256i) -> __m256i {
    _mm256_add_epi32(x, _mm256_sub_epi32(modulus, y))
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn update_root_avx2(root: __m256i, rate: __m256i, modulus: __m256i) -> __m256i {
    let correction = montgomery_simd::mul_epu32_256(root, rate);
    let product = _mm256_mul_epu32(root, _mm256_srli_epi64::<32>(rate));
    let correction = _mm256_mul_epu32(correction, modulus);
    shrink_avx2(
        _mm256_srli_epi64::<32>(_mm256_add_epi64(product, correction)),
        modulus,
    )
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn packed_rate_avx2<M>(index: usize, inverse: bool) -> __m256i
where
    M: Montgomery32NttModulus,
{
    let rate = if inverse {
        &M::INFO.inv_rate3_packed[index]
    } else {
        &M::INFO.rate3_packed[index]
    };
    _mm256_loadu_si256(rate.as_ptr().cast())
}

macro_rules! block_butterfly {
    ($forward:ident,$inverse:ident,$features:literal,$step:literal,$broadcast:expr,
    $load:ident,$store:ident,$add:ident,$sub:ident,$min:ident,$mul:path,$add_mod:path,$sub_mod:path) => {
        #[inline]
        #[target_feature(enable=$features)]
        unsafe fn $forward<M, const FIRST: bool>(base: *mut u32, quarter: usize, root: __m256i)
        where
            M: Montgomery32NttModulus,
        {
            let modulus = ($broadcast)(_mm256_set1_epi32(M::MOD as i32));
            let modulus2 = ($broadcast)(_mm256_set1_epi32((M::MOD * 2) as i32));
            let imag = ($broadcast)(_mm256_set1_epi32(M::INFO.root[2] as i32));
            let imag_r = ($broadcast)(_mm256_set1_epi32(M::INFO.root[2].wrapping_mul(M::R) as i32));
            let indices = _mm256_setr_epi32(0, 2, 0, 4, 0, 2, 0, 4);
            let r1 = _mm256_permutevar8x32_epi32(root, indices);
            let r1_r = _mm256_permutevar8x32_epi32(
                montgomery_simd::mul_epu32_256(root, _mm256_set1_epi32(M::R as i32)),
                indices,
            );
            let r2 = ($broadcast)(_mm256_shuffle_epi32::<0x55>(r1));
            let nr3 = ($broadcast)(_mm256_shuffle_epi32::<0xff>(r1));
            let r2_r = ($broadcast)(_mm256_shuffle_epi32::<0x55>(r1_r));
            let nr3_r = ($broadcast)(_mm256_shuffle_epi32::<0xff>(r1_r));
            let r1 = ($broadcast)(r1);
            let r1_r = ($broadcast)(r1_r);
            let shrink = |x, m| $min(x, $sub(x, m));
            let lazy_sub = |x, y, m| $add(x, $sub(m, y));

            let mut j = 0;
            while j < quarter {
                let p0 = j << 3;
                let x0 = $load(base.add(p0).cast());
                let x1 = $load(base.add(p0 + (quarter << 3)).cast());
                let x2 = $load(base.add(p0 + (quarter << 4)).cast());
                let x3 = $load(base.add(p0 + quarter * 24).cast());
                let (h0, h1, h2, h3) = if FIRST {
                    let h0 = $add_mod(x0, x2, modulus2);
                    let h1 = $add_mod(x1, x3, modulus2);
                    let h2 = $sub_mod(x0, x2, modulus2);
                    let h3 = $mul(lazy_sub(x1, x3, modulus2), imag, imag_r, modulus);
                    (h0, h1, h2, h3)
                } else {
                    let g1 = $mul(x1, r1, r1_r, modulus);
                    let ng3 = $mul(x3, nr3, nr3_r, modulus);
                    let g2 = $mul(x2, r2, r2_r, modulus);
                    let g0 = shrink(x0, modulus2);
                    let h3 = $mul($add(g1, ng3), imag, imag_r, modulus);
                    let h1 = $sub_mod(g1, ng3, modulus2);
                    let h0 = $add_mod(g0, g2, modulus2);
                    let h2 = $sub_mod(g0, g2, modulus2);
                    (h0, h1, h2, h3)
                };
                // Only y0 feeds the next first group; keep its inputs below 2*MOD.
                let mut y0 = if FIRST {
                    $add_mod(h0, h1, modulus2)
                } else {
                    $add(h0, h1)
                };
                let mut y1 = lazy_sub(h0, h1, modulus2);
                let mut y2 = $add(h2, h3);
                let mut y3 = lazy_sub(h2, h3, modulus2);
                if FIRST && quarter == 1 {
                    y0 = shrink(shrink(y0, modulus2), modulus);
                    y1 = shrink(shrink(y1, modulus2), modulus);
                    y2 = shrink(shrink(y2, modulus2), modulus);
                    y3 = shrink(shrink(y3, modulus2), modulus);
                }
                $store(base.add(p0).cast(), y0);
                $store(base.add(p0 + (quarter << 3)).cast(), y1);
                $store(base.add(p0 + (quarter << 4)).cast(), y2);
                $store(base.add(p0 + quarter * 24).cast(), y3);
                j += $step;
            }
        }
        #[inline]
        #[target_feature(enable=$features)]
        unsafe fn $inverse<M>(
            base: *mut u32,
            quarter: usize,
            root: __m256i,
            inv: u32,
            group: usize,
            last: bool,
        ) where
            M: Montgomery32NttModulus,
        {
            let modulus = ($broadcast)(_mm256_set1_epi32(M::MOD as i32));
            let modulus2 = ($broadcast)(_mm256_set1_epi32((M::MOD * 2) as i32));
            let imag = ($broadcast)(_mm256_set1_epi32(M::INFO.root[2] as i32));
            let imag_r = ($broadcast)(_mm256_set1_epi32(M::INFO.root[2].wrapping_mul(M::R) as i32));
            let indices = _mm256_setr_epi32(0, 2, 0, 4, 0, 2, 0, 4);
            let r1 = _mm256_permutevar8x32_epi32(root, indices);
            let r1_r = _mm256_permutevar8x32_epi32(
                montgomery_simd::mul_epu32_256(root, _mm256_set1_epi32(M::R as i32)),
                indices,
            );
            let r2 = ($broadcast)(_mm256_shuffle_epi32::<0x55>(r1));
            let r3 = ($broadcast)(_mm256_shuffle_epi32::<0xff>(r1));
            let r2_r = ($broadcast)(_mm256_shuffle_epi32::<0x55>(r1_r));
            let r3_r = ($broadcast)(_mm256_shuffle_epi32::<0xff>(r1_r));
            let r1 = ($broadcast)(r1);
            let r1_r = ($broadcast)(r1_r);
            let shrink = |x, m| $min(x, $sub(x, m));
            let lazy_sub = |x, y, m| $add(x, $sub(m, y));
            let inv_vec = ($broadcast)(_mm256_set1_epi32(inv as i32));
            let inv_r = ($broadcast)(_mm256_set1_epi32(inv.wrapping_mul(M::R) as i32));
            let mut j = 0;
            while j < quarter {
                let p0 = j << 3;
                let x0 = $load(base.add(p0).cast());
                let x1 = $load(base.add(p0 + (quarter << 3)).cast());
                let x2 = $load(base.add(p0 + (quarter << 4)).cast());
                let x3 = $load(base.add(p0 + quarter * 24).cast());
                let g3 = $mul(lazy_sub(x3, x2, modulus2), imag, imag_r, modulus);
                let g2 = $add_mod(x2, x3, modulus2);
                let g0 = $add_mod(x0, x1, modulus2);
                let g1 = $sub_mod(x0, x1, modulus2);
                let h2 = lazy_sub(g0, g2, modulus2);
                let h3 = lazy_sub(g1, g3, modulus2);
                let h0 = $add(g0, g2);
                let h1 = $add(g1, g3);
                let mut y0 = if quarter == 1 {
                    $mul(h0, inv_vec, inv_r, modulus)
                } else {
                    shrink(h0, modulus2)
                };
                let (mut y1, mut y2, mut y3) = if group == 0 && quarter > 1 {
                    (
                        shrink(h1, modulus2),
                        $sub_mod(g0, g2, modulus2),
                        $sub_mod(g1, g3, modulus2),
                    )
                } else {
                    (
                        $mul(h1, r1, r1_r, modulus),
                        $mul(h2, r2, r2_r, modulus),
                        $mul(h3, r3, r3_r, modulus),
                    )
                };
                if last {
                    y0 = shrink(y0, modulus);
                    y1 = shrink(y1, modulus);
                    y2 = shrink(y2, modulus);
                    y3 = shrink(y3, modulus);
                }
                $store(base.add(p0).cast(), y0);
                $store(base.add(p0 + (quarter << 3)).cast(), y1);
                $store(base.add(p0 + (quarter << 4)).cast(), y2);
                $store(base.add(p0 + quarter * 24).cast(), y3);
                j += $step;
            }
        }
    };
}
block_butterfly!(
    ntt_butterfly_avx2,
    intt_butterfly_avx2,
    "avx2",
    1,
    |x| x,
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_add_epi32,
    _mm256_sub_epi32,
    _mm256_min_epu32,
    montgomery_simd::montgomery_mul_256_fixed,
    montgomery_simd::add_mod_256,
    montgomery_simd::sub_mod_256
);
block_butterfly!(
    ntt_butterfly_avx512,
    intt_butterfly_avx512,
    "avx2,avx512f,avx512dq,avx512bw,avx512vl,avx512cd",
    2,
    _mm512_broadcast_i64x4,
    _mm512_loadu_si512,
    _mm512_storeu_si512,
    _mm512_add_epi32,
    _mm512_sub_epi32,
    _mm512_min_epu32,
    montgomery_simd::montgomery_mul_512_fixed,
    montgomery_simd::add_mod_512,
    montgomery_simd::sub_mod_512
);
#[target_feature(enable = "avx2")]
unsafe fn ntt_blocks_avx2<M>(a: *mut u32, n: usize, input_len: usize)
where
    M: Montgomery32NttModulus,
{
    let modulus = _mm256_set1_epi32(M::MOD as i32);
    let modulus2 = _mm256_set1_epi32(M::MOD.wrapping_mul(2) as i32);
    let wide = matches!(simd_backend(), SimdBackend::Avx512);
    let imag = M::INFO.root[2];
    let imag_r = _mm256_set1_epi32(imag.wrapping_mul(M::R) as i32);
    let imag = _mm256_set1_epi32(imag as i32);
    let initial_root = _mm256_setr_epi32(
        M::INFO.root[3] as i32,
        0,
        M::INFO.root[2] as i32,
        0,
        (M::MOD - M::mod_mul(M::INFO.root[2], M::INFO.root[3])) as i32,
        0,
        0,
        0,
    );
    let log_n = n.trailing_zeros() as usize;
    let mut roots = [initial_root; 16];
    let nn = n >> (log_n & 1);
    let tile_len = n.min(64);
    let mut stage_log = log_n & !1;
    let sparse = nn != n && input_len <= nn << 3;
    if nn != n && !sparse {
        let mut i = 0;
        while i < nn {
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, nn + i);
            store_block_avx2(a, i, montgomery_simd::add_mod_256(x0, x1, modulus2));
            store_block_avx2(a, nn + i, lazy_sub_avx2(x0, x1, modulus2));
            i += 1;
        }
    }
    if sparse {
        let quarter = nn >> 2;
        let w1 = M::INFO.root[3];
        let w3 = M::MOD - M::mod_mul(w1, M::INFO.root[2]);
        let r1 = _mm256_set1_epi32(w1 as i32);
        let r1_r = _mm256_set1_epi32(w1.wrapping_mul(M::R) as i32);
        let nr3 = _mm256_set1_epi32(w3 as i32);
        let nr3_r = _mm256_set1_epi32(w3.wrapping_mul(M::R) as i32);
        let store = |offset, values: [__m256i; 4]| {
            for (k, mut y) in values.into_iter().enumerate() {
                if quarter == 1 {
                    y = normalize_avx2(y, modulus, modulus2);
                }
                store_block_avx2(a, offset + quarter * k, y);
            }
        };
        for j in 0..quarter {
            let x0 = load_block_avx2(a, j);
            let x1 = load_block_avx2(a, j + quarter);
            let x2 = load_block_avx2(a, j + quarter * 2);
            let x3 = load_block_avx2(a, j + quarter * 3);
            let h0 = _mm256_add_epi32(x0, x2);
            let h1 = _mm256_add_epi32(x1, x3);
            let h2 = lazy_sub_avx2(x0, x2, modulus);
            let h3 = montgomery_simd::montgomery_mul_256_fixed(
                lazy_sub_avx2(x1, x3, modulus),
                imag,
                imag_r,
                modulus,
            );
            store(
                j,
                [
                    montgomery_simd::add_mod_256(h0, h1, modulus2),
                    lazy_sub_avx2(h0, h1, modulus2),
                    _mm256_add_epi32(h2, h3),
                    lazy_sub_avx2(h2, h3, modulus2),
                ],
            );
            let g1 = montgomery_simd::montgomery_mul_256_fixed(x1, r1, r1_r, modulus);
            let ng3 = montgomery_simd::montgomery_mul_256_fixed(x3, nr3, nr3_r, modulus);
            let g2 = montgomery_simd::montgomery_mul_256_fixed(x2, imag, imag_r, modulus);
            let h3 = montgomery_simd::montgomery_mul_256_fixed(
                _mm256_add_epi32(g1, ng3),
                imag,
                imag_r,
                modulus,
            );
            let h1 = montgomery_simd::sub_mod_256(g1, ng3, modulus2);
            let h0 = montgomery_simd::add_mod_256(x0, g2, modulus2);
            let h2 = montgomery_simd::sub_mod_256(x0, g2, modulus2);
            store(
                nn + j,
                [
                    _mm256_add_epi32(h0, h1),
                    lazy_sub_avx2(h0, h1, modulus2),
                    _mm256_add_epi32(h2, h3),
                    lazy_sub_avx2(h2, h3, modulus2),
                ],
            );
        }
        stage_log -= 2;
    }
    if nn == n && input_len <= n << 2 {
        let quarter = n >> 2;
        let mut i = 0;
        while i < quarter {
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, quarter + i);
            let x3 = montgomery_simd::montgomery_mul_256_fixed(x1, imag, imag_r, modulus);
            store_block_avx2(a, i, _mm256_add_epi32(x0, x1));
            store_block_avx2(a, quarter + i, lazy_sub_avx2(x0, x1, modulus2));
            store_block_avx2(a, 2 * quarter + i, _mm256_add_epi32(x0, x3));
            store_block_avx2(a, 3 * quarter + i, lazy_sub_avx2(x0, x3, modulus2));
            i += 1;
        }
        stage_log -= 2;
    }
    if stage_log == 0 {
        return;
    }
    let max_stage_log = stage_log;
    while stage_log >= 2 {
        let quarter = 1 << (stage_log - 2);
        if wide && quarter > 1 {
            ntt_butterfly_avx512::<M, true>(a, quarter, initial_root);
        } else {
            ntt_butterfly_avx2::<M, true>(a, quarter, initial_root);
        }
        stage_log -= 2;
    }
    stage_log = max_stage_log.min(6);
    let mut tile = 0;
    while tile < n {
        let base = a.add(tile << 3);
        let mut group_len = 1usize << stage_log;
        let mut quarter = group_len >> 2;
        while quarter > 1 {
            let root_slot = (stage_log - 2) >> 1;
            let mut root = roots[root_slot];
            let mut i = if tile == 0 { group_len } else { 0 };
            let mut group = (tile + i) >> stage_log;
            while i < tile_len {
                let current = root;
                root = update_root_avx2(
                    root,
                    packed_rate_avx2::<M>((!group).trailing_zeros() as usize, false),
                    modulus,
                );
                let base = base.add(i << 3);
                if wide {
                    ntt_butterfly_avx512::<M, false>(base, quarter, current);
                } else {
                    ntt_butterfly_avx2::<M, false>(base, quarter, current);
                }
                i += group_len;
                group += 1;
            }
            roots[root_slot] = root;
            group_len = quarter;
            quarter >>= 2;
            stage_log -= 2;
        }
        // Final-stage roots are used once; skip precomputing root*R.
        let r = _mm256_set1_epi32(M::R as i32);
        let store = |i, y| store_block_avx2(a, i, normalize_avx2(y, modulus, modulus2));
        let mut root = roots[0];
        let mut i = tile + if tile == 0 { 4 } else { 0 };
        while i < tile + tile_len {
            let r1 = _mm256_permutevar8x32_epi32(root, _mm256_setr_epi32(0, 2, 0, 4, 0, 2, 0, 4));
            root = update_root_avx2(
                root,
                packed_rate_avx2::<M>((!(i >> 2)).trailing_zeros() as usize, false),
                modulus,
            );
            let r2 = _mm256_shuffle_epi32::<0x55>(r1);
            let nr3 = _mm256_shuffle_epi32::<0xff>(r1);
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, i + 1);
            let x2 = load_block_avx2(a, i + 2);
            let x3 = load_block_avx2(a, i + 3);
            let g1 = montgomery_simd::montgomery_mul_256(
                x1,
                _mm256_shuffle_epi32::<0xa0>(r1),
                r,
                modulus,
            );
            let ng3 = montgomery_simd::montgomery_mul_256(x3, nr3, r, modulus);
            let g2 = montgomery_simd::montgomery_mul_256(x2, r2, r, modulus);
            let g0 = shrink_avx2(x0, modulus2);
            let h3 = montgomery_simd::montgomery_mul_256_fixed(
                _mm256_add_epi32(g1, ng3),
                imag,
                imag_r,
                modulus,
            );
            let h1 = montgomery_simd::sub_mod_256(g1, ng3, modulus2);
            let h0 = montgomery_simd::add_mod_256(g0, g2, modulus2);
            let h2 = montgomery_simd::sub_mod_256(g0, g2, modulus2);
            store(i, _mm256_add_epi32(h0, h1));
            store(i + 1, lazy_sub_avx2(h0, h1, modulus2));
            store(i + 2, _mm256_add_epi32(h2, h3));
            store(i + 3, lazy_sub_avx2(h2, h3, modulus2));
            i += 4;
        }
        roots[0] = root;

        tile += tile_len;
        if tile < n {
            stage_log = (tile.trailing_zeros() as usize & !1).min(max_stage_log);
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn intt_blocks_avx2<M>(a: *mut u32, n: usize)
where
    M: Montgomery32NttModulus,
{
    let inv = M::mod_inv(<M as MIntConvert<u32>>::from(n as u32));
    let modulus = _mm256_set1_epi32(M::MOD as i32);
    let modulus2 = _mm256_set1_epi32(M::MOD.wrapping_mul(2) as i32);
    let wide = matches!(simd_backend(), SimdBackend::Avx512);

    let initial_root = _mm256_setr_epi32(M::N1 as i32, 0, M::N1 as i32, 0, M::N1 as i32, 0, 0, 0);
    let log_n = n.trailing_zeros() as usize;
    let mut roots = [initial_root; 16];
    let nn = n >> (log_n & 1);
    let tile_len = n.min(64);
    let inv_vec = _mm256_set1_epi32(inv as i32);

    roots[0] = inv_vec;
    let mut tile = 0;
    while tile < n {
        let max_stage_log = (tile + tile_len).trailing_zeros() as usize;
        let mut stage_log = 2usize;
        let mut group_len = 4usize;
        let mut quarter = 1usize;
        while stage_log <= max_stage_log {
            let offset = tile + tile_len - group_len.max(tile_len);
            let base = a.add(offset << 3);
            let mut i = 0;
            let root_slot = (stage_log - 2) >> 1;
            let mut root = roots[root_slot];
            let mut group = (tile + i) >> stage_log;
            while i < tile_len {
                let current = root;
                root = update_root_avx2(
                    root,
                    packed_rate_avx2::<M>((!group).trailing_zeros() as usize, true),
                    modulus,
                );
                let base = base.add(i << 3);
                let last = group_len == n;
                if wide && quarter > 1 {
                    intt_butterfly_avx512::<M>(base, quarter, current, inv, group, last);
                } else {
                    intt_butterfly_avx2::<M>(base, quarter, current, inv, group, last);
                }
                i += group_len;
                group += 1;
            }
            roots[root_slot] = root;
            quarter = group_len;
            group_len <<= 2;
            stage_log += 2;
        }
        tile += tile_len;
    }
    if nn != n {
        let mut i = 0;
        while i < nn {
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, nn + i);
            store_block_avx2(
                a,
                i,
                shrink_avx2(montgomery_simd::add_mod_256(x0, x1, modulus2), modulus),
            );
            store_block_avx2(
                a,
                nn + i,
                shrink_avx2(montgomery_simd::sub_mod_256(x0, x1, modulus2), modulus),
            );
            i += 1;
        }
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
unsafe fn convolve_8_avx2<M>(f: *mut u32, g: *const u32, n: usize)
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
        let mut work = std::mem::MaybeUninit::<[__m256i; 8]>::uninit();
        let work = work.as_mut_ptr().cast::<u32>();
        for (j, ww) in [
            rr,
            M::MOD.wrapping_mul(2) - rr,
            rr_i,
            M::MOD.wrapping_mul(2) - rr_i,
        ]
        .into_iter()
        .enumerate()
        {
            let k = i + j;
            let ff = load_block_avx2(f, k);
            // ff, g < MOD and fw < 3 * MOD / 2, so the sum reduces below 4 * MOD.
            let fw = montgomery_simd::montgomery_mul_256_fixed(
                ff,
                _mm256_set1_epi32(ww as i32),
                _mm256_set1_epi32(ww.wrapping_mul(M::R) as i32),
                mod_vec,
            );
            _mm256_store_si256(work.add(j << 4).cast(), fw);
            _mm256_store_si256(work.add((j << 4) + 8).cast(), ff);
        }
        let mut even = [_mm256_setzero_si256(); 4];
        let mut odd = [_mm256_setzero_si256(); 4];
        let mut l = 0;
        while l < 8 {
            let mut j = 0;
            while j < 4 {
                let x = _mm256_loadu_si256(work.add((j << 4) + 8 - l).cast());
                let y = _mm256_set1_epi32(*g.add(((i + j) << 3) + l) as i32);
                even[j] = _mm256_add_epi64(even[j], _mm256_mul_epu32(x, y));
                odd[j] = _mm256_add_epi64(odd[j], _mm256_mul_epu32(_mm256_bsrli_epi128::<4>(x), y));
                j += 1;
            }
            l += 1;
        }
        let mut j = 0;
        while j < 4 {
            let x = reduce_sum_avx2(even[j], odd[j], r_vec, mod_vec);
            // Repeated products must preserve the canonical input range.
            store_block_avx2(f, i + j, normalize_avx2(x, mod_vec, mod2_vec));
            j += 1;
        }
        i += 4;
        let rate = M::INFO.rate3_packed[(i >> 2).trailing_zeros() as usize][1];
        rr = M::mod_mul(rr, rate);
    }
}

#[target_feature(enable = "avx2")]
pub unsafe fn transform_blocks_avx2<M>(f: &mut [MInt<M>], input_len: usize)
where
    M: Montgomery32NttModulus,
{
    let n = f.len() >> 3;
    let f = f.as_mut_ptr() as *mut u32;
    ntt_blocks_avx2::<M>(f, n, input_len);
}

#[target_feature(enable = "avx2")]
pub unsafe fn multiply_blocks_avx2<M>(f: &mut [MInt<M>], g: Option<&[MInt<M>]>)
where
    M: Montgomery32NttModulus,
{
    let n = f.len() >> 3;
    let f = f.as_mut_ptr() as *mut u32;
    let g = g.map_or(f.cast_const(), |g| g.as_ptr().cast());
    convolve_8_avx2::<M>(f, g, n);
}

#[target_feature(enable = "avx2")]
pub unsafe fn inverse_transform_blocks_avx2<M>(f: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let n = f.len() >> 3;
    let f = f.as_mut_ptr() as *mut u32;
    intt_blocks_avx2::<M>(f, n);
}

#[target_feature(enable = "avx2")]
pub unsafe fn convolve_blocks_avx2<M>(
    f: &mut Vec<MInt<M>>,
    g: &mut Vec<MInt<M>>,
    same: bool,
    size: usize,
    input_len: (usize, usize),
) where
    M: Montgomery32NttModulus,
{
    let n = size >> 3;
    let f_ptr = f.as_mut_ptr().cast();
    let g_ptr = if same { f_ptr } else { g.as_mut_ptr().cast() };
    ntt_blocks_avx2::<M>(f_ptr, n, input_len.0);
    f.set_len(size);
    if !same {
        ntt_blocks_avx2::<M>(g_ptr, n, input_len.1);
    }
    convolve_8_avx2::<M>(f_ptr, g_ptr, n);
    intt_blocks_avx2::<M>(f_ptr, n);
}
