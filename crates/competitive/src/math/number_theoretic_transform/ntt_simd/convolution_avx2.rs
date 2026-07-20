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
unsafe fn add_mod_avx2(x: __m256i, y: __m256i, modulus: __m256i) -> __m256i {
    shrink_avx2(_mm256_add_epi32(x, y), modulus)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn sub_mod_avx2(x: __m256i, y: __m256i, modulus: __m256i) -> __m256i {
    let d = _mm256_sub_epi32(x, y);
    _mm256_min_epu32(d, _mm256_add_epi32(d, modulus))
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn lazy_sub_avx2(x: __m256i, y: __m256i, modulus: __m256i) -> __m256i {
    _mm256_add_epi32(x, _mm256_sub_epi32(modulus, y))
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn montgomery_mul_even_avx2(
    x: __m256i,
    y: __m256i,
    r: __m256i,
    modulus: __m256i,
) -> __m256i {
    let x_odd = _mm256_bsrli_epi128::<4>(x);
    let t_even = _mm256_mul_epu32(x, y);
    let t_odd = _mm256_mul_epu32(x_odd, y);
    let m_even = _mm256_mul_epu32(t_even, r);
    let m_odd = _mm256_mul_epu32(t_odd, r);
    let u_even = _mm256_add_epi64(t_even, _mm256_mul_epu32(m_even, modulus));
    let u_odd = _mm256_add_epi64(t_odd, _mm256_mul_epu32(m_odd, modulus));
    _mm256_or_si256(_mm256_bsrli_epi128::<4>(u_even), u_odd)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn update_root_avx2(root: __m256i, rate: __m256i, modulus: __m256i) -> __m256i {
    let correction = _mm256_mul_epu32(root, rate);
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

#[target_feature(enable = "avx2")]
unsafe fn ntt_blocks_avx2<M>(a: *mut u32, n: usize)
where
    M: Montgomery32NttModulus,
{
    let modulus = _mm256_set1_epi32(M::MOD as i32);
    let modulus2 = _mm256_set1_epi32(M::MOD.wrapping_mul(2) as i32);
    let r = _mm256_set1_epi32(M::R as i32);
    let imag = M::INFO.root[2];
    let imag_r = _mm256_set1_epi32(imag.wrapping_mul(M::R) as i32);
    let imag = _mm256_set1_epi32(imag as i32);
    let root_indices = _mm256_setr_epi32(0, 2, 0, 4, 0, 2, 0, 4);
    let root3 = M::INFO.root[3];
    let root2 = M::INFO.root[2];
    let initial_root = _mm256_setr_epi32(
        root3 as i32,
        0,
        root2 as i32,
        0,
        (M::MOD - M::mod_mul(root2, root3)) as i32,
        0,
        0,
        0,
    );
    let log_n = n.trailing_zeros() as usize;
    let mut roots = [initial_root; 16];
    let nn = n >> (log_n & 1);
    let tile_len = n.min(64);

    if nn != n {
        let mut i = 0;
        while i < nn {
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, nn + i);
            store_block_avx2(a, i, add_mod_avx2(x0, x1, modulus2));
            store_block_avx2(a, nn + i, lazy_sub_avx2(x0, x1, modulus2));
            i += 1;
        }
    }

    let mut size = nn >> 2;
    while size > 0 {
        let final_stage = size == 1;
        let mut i = 0;
        while i < size {
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, size + i);
            let x2 = load_block_avx2(a, size * 2 + i);
            let x3 = load_block_avx2(a, size * 3 + i);
            let g3 = simd32::montgomery_mul_256_fixed(
                lazy_sub_avx2(x1, x3, modulus2),
                imag,
                imag_r,
                modulus,
            );
            let g1 = add_mod_avx2(x1, x3, modulus2);
            let g0 = add_mod_avx2(x0, x2, modulus2);
            let g2 = sub_mod_avx2(x0, x2, modulus2);
            let mut y0 = add_mod_avx2(g0, g1, modulus2);
            let mut y1 = lazy_sub_avx2(g0, g1, modulus2);
            let mut y2 = _mm256_add_epi32(g2, g3);
            let mut y3 = lazy_sub_avx2(g2, g3, modulus2);
            if final_stage {
                y0 = normalize_avx2(y0, modulus, modulus2);
                y1 = normalize_avx2(y1, modulus, modulus2);
                y2 = normalize_avx2(y2, modulus, modulus2);
                y3 = normalize_avx2(y3, modulus, modulus2);
            }
            store_block_avx2(a, i, y0);
            store_block_avx2(a, size + i, y1);
            store_block_avx2(a, size * 2 + i, y2);
            store_block_avx2(a, size * 3 + i, y3);
            i += 1;
        }
        size >>= 2;
    }

    let mut tile = 0;
    let mut stage_log = log_n.min(6) & !1;
    let mut root_slot = (stage_log - 2) >> 1;
    while tile < n {
        let base = a.add(tile << 3);
        let mut group_len = 1usize << stage_log;
        let mut quarter = group_len >> 2;
        while quarter > 1 {
            let mut root = roots[root_slot];
            let mut i = if tile == 0 { group_len } else { 0 };
            let mut group = (tile + i) >> stage_log;
            while i < tile_len {
                let r1 = _mm256_permutevar8x32_epi32(root, root_indices);
                let r1_r = _mm256_permutevar8x32_epi32(_mm256_mul_epu32(root, r), root_indices);
                root = update_root_avx2(
                    root,
                    packed_rate_avx2::<M>((!group).trailing_zeros() as usize, false),
                    modulus,
                );
                let r2 = _mm256_shuffle_epi32::<0x55>(r1);
                let nr3 = _mm256_shuffle_epi32::<0xff>(r1);
                let r2_r = _mm256_shuffle_epi32::<0x55>(r1_r);
                let nr3_r = _mm256_shuffle_epi32::<0xff>(r1_r);
                let mut j = 0;
                while j < quarter {
                    let p0 = (i + j) << 3;
                    let x0 = _mm256_loadu_si256(base.add(p0).cast());
                    let x1 = _mm256_loadu_si256(base.add(p0 + (quarter << 3)).cast());
                    let x2 = _mm256_loadu_si256(base.add(p0 + (quarter << 4)).cast());
                    let x3 = _mm256_loadu_si256(base.add(p0 + quarter * 24).cast());
                    let g1 = simd32::montgomery_mul_256_fixed(x1, r1, r1_r, modulus);
                    let ng3 = simd32::montgomery_mul_256_fixed(x3, nr3, nr3_r, modulus);
                    let g2 = simd32::montgomery_mul_256_fixed(x2, r2, r2_r, modulus);
                    let g0 = shrink_avx2(x0, modulus2);
                    let h3 = simd32::montgomery_mul_256_fixed(
                        _mm256_add_epi32(g1, ng3),
                        imag,
                        imag_r,
                        modulus,
                    );
                    let h1 = sub_mod_avx2(g1, ng3, modulus2);
                    let h0 = add_mod_avx2(g0, g2, modulus2);
                    let h2 = sub_mod_avx2(g0, g2, modulus2);
                    _mm256_storeu_si256(base.add(p0).cast(), _mm256_add_epi32(h0, h1));
                    _mm256_storeu_si256(
                        base.add(p0 + (quarter << 3)).cast(),
                        lazy_sub_avx2(h0, h1, modulus2),
                    );
                    _mm256_storeu_si256(
                        base.add(p0 + (quarter << 4)).cast(),
                        _mm256_add_epi32(h2, h3),
                    );
                    _mm256_storeu_si256(
                        base.add(p0 + quarter * 24).cast(),
                        lazy_sub_avx2(h2, h3, modulus2),
                    );
                    j += 1;
                }
                i += group_len;
                group += 1;
            }
            roots[root_slot] = root;
            group_len = quarter;
            quarter >>= 2;
            stage_log -= 2;
            root_slot -= 1;
        }

        let mut root = roots[0];
        let mut i = tile + if tile == 0 { 4 } else { 0 };
        while i < tile + tile_len {
            let r1 = _mm256_permutevar8x32_epi32(root, root_indices);
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
            let g1 = montgomery_mul_even_avx2(x1, r1, r, modulus);
            let ng3 = montgomery_mul_even_avx2(x3, nr3, r, modulus);
            let g2 = montgomery_mul_even_avx2(x2, r2, r, modulus);
            let g0 = shrink_avx2(x0, modulus2);
            let h3 =
                simd32::montgomery_mul_256_fixed(_mm256_add_epi32(g1, ng3), imag, imag_r, modulus);
            let h1 = sub_mod_avx2(g1, ng3, modulus2);
            let h0 = add_mod_avx2(g0, g2, modulus2);
            let h2 = sub_mod_avx2(g0, g2, modulus2);
            store_block_avx2(
                a,
                i,
                normalize_avx2(_mm256_add_epi32(h0, h1), modulus, modulus2),
            );
            store_block_avx2(
                a,
                i + 1,
                normalize_avx2(lazy_sub_avx2(h0, h1, modulus2), modulus, modulus2),
            );
            store_block_avx2(
                a,
                i + 2,
                normalize_avx2(_mm256_add_epi32(h2, h3), modulus, modulus2),
            );
            store_block_avx2(
                a,
                i + 3,
                normalize_avx2(lazy_sub_avx2(h2, h3, modulus2), modulus, modulus2),
            );
            i += 4;
        }
        roots[0] = root;

        tile += tile_len;
        if tile < n {
            stage_log = tile.trailing_zeros() as usize & !1;
            root_slot = (stage_log - 2) >> 1;
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
    let r = _mm256_set1_epi32(M::R as i32);
    let imag = _mm256_set1_epi32(M::INFO.root[2] as i32);
    let imag_r = _mm256_set1_epi32(M::INFO.root[2].wrapping_mul(M::R) as i32);
    let root_indices = _mm256_setr_epi32(0, 2, 0, 4, 0, 2, 0, 4);
    let root3 = M::INFO.inv_root[3];
    let root2 = M::INFO.inv_root[2];
    let initial_root = _mm256_setr_epi32(
        root3 as i32,
        0,
        root2 as i32,
        0,
        M::mod_mul(root2, root3) as i32,
        0,
        0,
        0,
    );
    let log_n = n.trailing_zeros() as usize;
    let mut roots = [initial_root; 16];
    let nn = n >> (log_n & 1);
    let tile_len = n.min(64);
    let inv_vec = _mm256_set1_epi32(inv as i32);
    let inv_r = _mm256_set1_epi32(inv.wrapping_mul(M::R) as i32);
    roots[0] = inv_vec;

    let mut tile = 0;
    while tile < n {
        let max_stage_log = (tile + tile_len).trailing_zeros() as usize;
        let mut stage_log = 4usize;
        let mut root_slot = 1usize;

        let mut root = roots[0];
        let mut i = tile;
        while i < tile + tile_len {
            let r1 = _mm256_permutevar8x32_epi32(root, root_indices);
            root = update_root_avx2(
                root,
                packed_rate_avx2::<M>((!(i >> 2)).trailing_zeros() as usize, true),
                modulus,
            );
            let r2 = _mm256_shuffle_epi32::<0x55>(r1);
            let r3 = _mm256_shuffle_epi32::<0xff>(r1);
            let x0 = load_block_avx2(a, i);
            let x1 = load_block_avx2(a, i + 1);
            let x2 = load_block_avx2(a, i + 2);
            let x3 = load_block_avx2(a, i + 3);
            let g3 = simd32::montgomery_mul_256_fixed(
                lazy_sub_avx2(x3, x2, modulus2),
                imag,
                imag_r,
                modulus,
            );
            let g2 = add_mod_avx2(x2, x3, modulus2);
            let g0 = add_mod_avx2(x0, x1, modulus2);
            let g1 = sub_mod_avx2(x0, x1, modulus2);
            let h2 = lazy_sub_avx2(g0, g2, modulus2);
            let h3 = lazy_sub_avx2(g1, g3, modulus2);
            let h0 = _mm256_add_epi32(g0, g2);
            let h1 = _mm256_add_epi32(g1, g3);
            if inv == M::N1 {
                store_block_avx2(a, i, shrink_avx2(h0, modulus2));
            } else {
                store_block_avx2(
                    a,
                    i,
                    simd32::montgomery_mul_256_fixed(h0, inv_vec, inv_r, modulus),
                );
            }
            store_block_avx2(a, i + 1, montgomery_mul_even_avx2(h1, r1, r, modulus));
            store_block_avx2(a, i + 2, montgomery_mul_even_avx2(h2, r2, r, modulus));
            store_block_avx2(a, i + 3, montgomery_mul_even_avx2(h3, r3, r, modulus));
            i += 4;
        }
        roots[0] = root;

        let mut group_len = 16usize;
        let mut quarter = 4usize;
        while stage_log <= max_stage_log {
            let offset = tile + tile_len - group_len.max(tile_len);
            let base = a.add(offset << 3);
            let mut i = 0;
            let mut root = roots[root_slot];
            if offset == 0 {
                let final_stage = group_len == n;
                while i < quarter {
                    let x0 = load_block_avx2(a, i);
                    let x1 = load_block_avx2(a, quarter + i);
                    let x2 = load_block_avx2(a, quarter * 2 + i);
                    let x3 = load_block_avx2(a, quarter * 3 + i);
                    let g3 = simd32::montgomery_mul_256_fixed(
                        lazy_sub_avx2(x3, x2, modulus2),
                        imag,
                        imag_r,
                        modulus,
                    );
                    let g2 = add_mod_avx2(x2, x3, modulus2);
                    let g0 = add_mod_avx2(x0, x1, modulus2);
                    let g1 = sub_mod_avx2(x0, x1, modulus2);
                    let mut y0 = _mm256_add_epi32(g0, g2);
                    let mut y1 = _mm256_add_epi32(g1, g3);
                    let mut y2 = sub_mod_avx2(g0, g2, modulus2);
                    let mut y3 = sub_mod_avx2(g1, g3, modulus2);
                    if final_stage {
                        y0 = shrink_avx2(y0, modulus);
                        y1 = shrink_avx2(y1, modulus);
                        y2 = shrink_avx2(y2, modulus);
                        y3 = shrink_avx2(y3, modulus);
                    } else {
                        y0 = shrink_avx2(y0, modulus2);
                        y1 = shrink_avx2(y1, modulus2);
                    }
                    store_block_avx2(a, i, y0);
                    store_block_avx2(a, quarter + i, y1);
                    store_block_avx2(a, quarter * 2 + i, y2);
                    store_block_avx2(a, quarter * 3 + i, y3);
                    i += 1;
                }
                i = group_len;
            }

            let mut group = (tile + i) >> stage_log;
            while i < tile_len {
                let r1 = _mm256_permutevar8x32_epi32(root, root_indices);
                let r1_r = _mm256_permutevar8x32_epi32(_mm256_mul_epu32(root, r), root_indices);
                root = update_root_avx2(
                    root,
                    packed_rate_avx2::<M>((!group).trailing_zeros() as usize, true),
                    modulus,
                );
                let r2 = _mm256_shuffle_epi32::<0x55>(r1);
                let r3 = _mm256_shuffle_epi32::<0xff>(r1);
                let r2_r = _mm256_shuffle_epi32::<0x55>(r1_r);
                let r3_r = _mm256_shuffle_epi32::<0xff>(r1_r);
                let mut j = 0;
                while j < quarter {
                    let p0 = (i + j) << 3;
                    let x0 = _mm256_loadu_si256(base.add(p0).cast());
                    let x1 = _mm256_loadu_si256(base.add(p0 + (quarter << 3)).cast());
                    let x2 = _mm256_loadu_si256(base.add(p0 + (quarter << 4)).cast());
                    let x3 = _mm256_loadu_si256(base.add(p0 + quarter * 24).cast());
                    let g3 = simd32::montgomery_mul_256_fixed(
                        lazy_sub_avx2(x3, x2, modulus2),
                        imag,
                        imag_r,
                        modulus,
                    );
                    let g2 = add_mod_avx2(x2, x3, modulus2);
                    let g0 = add_mod_avx2(x0, x1, modulus2);
                    let g1 = sub_mod_avx2(x0, x1, modulus2);
                    let h2 = lazy_sub_avx2(g0, g2, modulus2);
                    let h3 = lazy_sub_avx2(g1, g3, modulus2);
                    let h0 = _mm256_add_epi32(g0, g2);
                    let h1 = _mm256_add_epi32(g1, g3);
                    _mm256_storeu_si256(base.add(p0).cast(), shrink_avx2(h0, modulus2));
                    _mm256_storeu_si256(
                        base.add(p0 + (quarter << 3)).cast(),
                        simd32::montgomery_mul_256_fixed(h1, r1, r1_r, modulus),
                    );
                    _mm256_storeu_si256(
                        base.add(p0 + (quarter << 4)).cast(),
                        simd32::montgomery_mul_256_fixed(h2, r2, r2_r, modulus),
                    );
                    _mm256_storeu_si256(
                        base.add(p0 + quarter * 24).cast(),
                        simd32::montgomery_mul_256_fixed(h3, r3, r3_r, modulus),
                    );
                    j += 1;
                }
                i += group_len;
                group += 1;
            }
            roots[root_slot] = root;
            quarter = group_len;
            group_len <<= 2;
            stage_log += 2;
            root_slot += 1;
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
                shrink_avx2(
                    shrink_avx2(add_mod_avx2(x0, x1, modulus2), modulus),
                    modulus,
                ),
            );
            store_block_avx2(
                a,
                nn + i,
                shrink_avx2(
                    shrink_avx2(sub_mod_avx2(x0, x1, modulus2), modulus),
                    modulus,
                ),
            );
            i += 1;
        }
    } else {
        let mut i = 0;
        while i < n {
            store_block_avx2(
                a,
                i,
                shrink_avx2(shrink_avx2(load_block_avx2(a, i), modulus), modulus),
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
    #[repr(C, align(32))]
    struct AlignedWork([u32; 64]);

    let mod_vec = _mm256_set1_epi32(M::MOD as i32);
    let mod2_vec = _mm256_set1_epi32(M::MOD.wrapping_add(M::MOD) as i32);
    let r_vec = _mm256_set1_epi32(M::R as i32);
    let mut rr = M::N1;
    let mut i = 0;
    while i < n {
        let rr_i = M::mod_mul(rr, M::INFO.root[2]);
        let mut work = std::mem::MaybeUninit::<AlignedWork>::uninit();
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
            // fw < 5 * MOD / 4, so the eight-product sum still reduces below 7 * MOD / 2.
            let fw = shrink_avx2(
                simd32::montgomery_mul_256_fixed(
                    ff,
                    _mm256_set1_epi32(ww as i32),
                    _mm256_set1_epi32(ww.wrapping_mul(M::R) as i32),
                    mod_vec,
                ),
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
            store_block_avx2(f, i + j, _mm256_min_epu32(x, _mm256_sub_epi32(x, mod2_vec)));
            j += 1;
        }
        i += 4;
        rr = M::mod_mul(rr, M::INFO.rate3[(i >> 2).trailing_zeros() as usize]);
    }
}

#[target_feature(enable = "avx2")]
pub unsafe fn transform_blocks_avx2<M>(f: &mut [MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let n = f.len() >> 3;
    let f = f.as_mut_ptr() as *mut u32;
    ntt_blocks_avx2::<M>(f, n);
}

#[target_feature(enable = "avx2")]
pub unsafe fn multiply_blocks_avx2<M>(f: &mut [MInt<M>], g: &[MInt<M>])
where
    M: Montgomery32NttModulus,
{
    let n = f.len() >> 3;
    let f = f.as_mut_ptr() as *mut u32;
    let g = g.as_ptr() as *const u32;
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
pub unsafe fn convolve_blocks_avx2<M>(f: &mut [MInt<M>], g: &mut [MInt<M>], same: bool)
where
    M: Montgomery32NttModulus,
{
    let n = f.len() >> 3;
    let f = f.as_mut_ptr().cast();
    let g = g.as_mut_ptr().cast();
    ntt_blocks_avx2::<M>(f, n);
    if same {
        std::ptr::copy_nonoverlapping(f, g, n << 3);
    } else {
        ntt_blocks_avx2::<M>(g, n);
    }
    convolve_8_avx2::<M>(f, g, n);
    intt_blocks_avx2::<M>(f, n);
}
