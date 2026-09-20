use super::{MInt, MIntBase, advise_huge_pages, avx512_enabled};
use std::arch::x86_64::*;

struct Kernel {
    modulus: u32,
    inverse: u32,
    avx512: bool,
}

#[target_feature(enable = "avx2")]
unsafe fn leaf_avx2(
    a: *const u32,
    b: *const u32,
    c: *mut u32,
    shape: (usize, usize, usize),
    modulus: u32,
    inverse: u32,
) {
    let (n, m, p) = shape;
    unsafe {
        // Eight products fit in u64 for modulus < 2^30. Subtracting 2*modulus*2^32
        // keeps each accumulator below that bound without changing its residue.
        let bound = _mm256_set1_epi64x(((2 * modulus as u64) << 32) as i64);
        let mv = _mm256_set1_epi32(modulus as i32);
        let rv = _mm256_set1_epi32(inverse as i32);
        for i in (0..n).step_by(4) {
            for j in (0..p).step_by(8) {
                let mut lo = [_mm256_setzero_si256(); 4];
                let mut hi = lo;
                for first in (0..m).step_by(8) {
                    for k in first..first + 8 {
                        let b0 = _mm256_loadu_si256(b.add(k * p + j).cast());
                        let b1 = _mm256_srli_epi64::<32>(b0);
                        for t in 0..4 {
                            let factor = _mm256_set1_epi32(*a.add((i + t) * m + k) as i32);
                            lo[t] = _mm256_add_epi64(lo[t], _mm256_mul_epu32(factor, b0));
                            hi[t] = _mm256_add_epi64(hi[t], _mm256_mul_epu32(factor, b1));
                        }
                    }
                    for t in 0..4 {
                        lo[t] = _mm256_min_epu32(lo[t], _mm256_sub_epi32(lo[t], bound));
                        hi[t] = _mm256_min_epu32(hi[t], _mm256_sub_epi32(hi[t], bound));
                    }
                }
                for t in 0..4 {
                    let x =
                        _mm256_add_epi64(lo[t], _mm256_mul_epu32(_mm256_mul_epu32(lo[t], rv), mv));
                    let y =
                        _mm256_add_epi64(hi[t], _mm256_mul_epu32(_mm256_mul_epu32(hi[t], rv), mv));
                    let x = _mm256_or_si256(_mm256_srli_epi64::<32>(x), y);
                    let x = _mm256_min_epu32(x, _mm256_sub_epi32(x, mv));
                    let x = _mm256_min_epu32(x, _mm256_sub_epi32(x, mv));
                    _mm256_storeu_si256(c.add((i + t) * p + j).cast(), x);
                }
            }
        }
    }
}

#[target_feature(enable = "avx512f")]
unsafe fn leaf_avx512(
    a: *const u32,
    b: *const u32,
    c: *mut u32,
    shape: (usize, usize, usize),
    modulus: u32,
    inverse: u32,
) {
    let (n, m, p) = shape;
    unsafe {
        let bound = _mm512_set1_epi64(((2 * modulus as u64) << 32) as i64);
        let mv = _mm512_set1_epi32(modulus as i32);
        let rv = _mm512_set1_epi32(inverse as i32);
        for i in (0..n).step_by(8) {
            for j in (0..p).step_by(16) {
                let mask = if j + 16 <= p { 0xffff } else { 0xff };
                let mut lo = [_mm512_setzero_si512(); 8];
                let mut hi = lo;
                for first in (0..m).step_by(8) {
                    for k in first..first + 8 {
                        let b0 = _mm512_maskz_loadu_epi32(mask, b.add(k * p + j).cast());
                        let b1 = _mm512_srli_epi64::<32>(b0);
                        for t in 0..8 {
                            let factor = _mm512_set1_epi32(*a.add((i + t) * m + k) as i32);
                            lo[t] = _mm512_add_epi64(lo[t], _mm512_mul_epu32(factor, b0));
                            hi[t] = _mm512_add_epi64(hi[t], _mm512_mul_epu32(factor, b1));
                        }
                    }
                    for t in 0..8 {
                        lo[t] = _mm512_min_epu32(lo[t], _mm512_sub_epi32(lo[t], bound));
                        hi[t] = _mm512_min_epu32(hi[t], _mm512_sub_epi32(hi[t], bound));
                    }
                }
                for t in 0..8 {
                    let x =
                        _mm512_add_epi64(lo[t], _mm512_mul_epu32(_mm512_mul_epu32(lo[t], rv), mv));
                    let y =
                        _mm512_add_epi64(hi[t], _mm512_mul_epu32(_mm512_mul_epu32(hi[t], rv), mv));
                    let x = _mm512_or_si512(_mm512_srli_epi64::<32>(x), y);
                    let x = _mm512_min_epu32(x, _mm512_sub_epi32(x, mv));
                    let x = _mm512_min_epu32(x, _mm512_sub_epi32(x, mv));
                    _mm512_mask_storeu_epi32(c.add((i + t) * p + j).cast(), mask, x);
                }
            }
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn combine<const SUB: bool>(
    a: *const u32,
    b: *const u32,
    c: *mut u32,
    len: usize,
    modulus: u32,
) {
    unsafe {
        let modulus = _mm256_set1_epi32(modulus as i32);
        for i in (0..len).step_by(8) {
            let a = _mm256_loadu_si256(a.add(i).cast());
            let b = _mm256_loadu_si256(b.add(i).cast());
            let v = if SUB {
                _mm256_sub_epi32(_mm256_add_epi32(a, modulus), b)
            } else {
                _mm256_add_epi32(a, b)
            };
            let v = _mm256_min_epu32(v, _mm256_sub_epi32(v, modulus));
            _mm256_storeu_si256(c.add(i).cast(), v);
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn multiply(
    a: *const u32,
    b: *const u32,
    c: *mut u32,
    shape: (usize, usize, usize),
    work: *mut u32,
    kernel: &Kernel,
) {
    unsafe {
        let (n, m, p) = shape;
        let (modulus, inverse) = (kernel.modulus, kernel.inverse);
        if n.min(m).min(p) <= 64 || n % 16 != 0 || m % 16 != 0 || p % 16 != 0 {
            if kernel.avx512 {
                leaf_avx512(a, b, c, shape, modulus, inverse);
            } else {
                leaf_avx2(a, b, c, shape, modulus, inverse);
            }
            return;
        }
        let (n, m, p) = (n / 2, m / 2, p / 2);
        let (s, t, q) = (work, work.add(n * m), work.add(n * m + m * p));
        let work = q.add(n * p);
        let (a00, a01, a10, a11) = (a, a.add(n * m), a.add(2 * n * m), a.add(3 * n * m));
        let (b00, b01, b10, b11) = (b, b.add(m * p), b.add(2 * m * p), b.add(3 * m * p));
        let (c00, c01, c10, c11) = (c, c.add(n * p), c.add(2 * n * p), c.add(3 * n * p));
        multiply(a00, b00, c11, (n, m, p), work, kernel);
        multiply(a01, b10, c00, (n, m, p), work, kernel);
        combine::<false>(c00, c11, c00, n * p, modulus);
        combine::<false>(a10, a11, s, n * m, modulus);
        combine::<true>(b01, b00, t, m * p, modulus);
        multiply(s, t, c01, (n, m, p), work, kernel);
        combine::<true>(s, a00, s, n * m, modulus);
        combine::<true>(b11, t, t, m * p, modulus);
        multiply(s, t, c10, (n, m, p), work, kernel);
        combine::<false>(c11, c10, c10, n * p, modulus);
        combine::<true>(a01, s, s, n * m, modulus);
        multiply(s, b11, q, (n, m, p), work, kernel);
        combine::<false>(c10, c01, c11, n * p, modulus);
        combine::<false>(c11, q, c01, n * p, modulus);
        combine::<true>(t, b10, t, m * p, modulus);
        multiply(a11, t, q, (n, m, p), work, kernel);
        combine::<true>(c10, q, c10, n * p, modulus);
        combine::<true>(a00, a10, s, n * m, modulus);
        combine::<true>(b11, b01, t, m * p, modulus);
        multiply(s, t, q, (n, m, p), work, kernel);
        combine::<false>(c10, q, c10, n * p, modulus);
        combine::<false>(c11, q, c11, n * p, modulus);
    }
}

fn blocks(rows: usize, cols: usize, depth: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    (0..1usize << (2 * depth)).map(move |block| {
        let (mut row, mut col) = (0, 0);
        for bit in 0..depth {
            row |= (block >> (2 * bit + 1) & 1) << bit;
            col |= (block >> (2 * bit) & 1) << bit;
        }
        (
            block * (rows >> depth) * (cols >> depth),
            row * (rows >> depth),
            col * (cols >> depth),
        )
    })
}

impl<M> MInt<M>
where
    M: MIntBase<Inner = u32>,
{
    /// # Safety
    /// AVX2 must be available. The modulus must be odd, greater than one and below 2^30.
    /// `scale` must be below the modulus; entries must have canonical raw representations.
    #[target_feature(enable = "avx2")]
    pub unsafe fn matrix_product_avx2(
        a: &[Vec<Self>],
        b: &[Vec<Self>],
        scale: u32,
    ) -> Vec<Vec<Self>> {
        let (n, m, p) = (a.len(), b.len(), b.first().map_or(0, Vec::len));
        assert!(a.iter().all(|row| row.len() == m));
        assert!(b.iter().all(|row| row.len() == p));
        let modulus = M::get_mod();
        let alignment = if n.min(m).min(p) <= 64 { 8 } else { 32 };
        let (nn, mm, pp) = (
            n.div_ceil(alignment) * alignment,
            m.div_ceil(alignment) * alignment,
            p.div_ceil(alignment) * alignment,
        );
        let mut depth = 0;
        let (mut x, mut y, mut z) = (nn, mm, pp);
        while x.min(y).min(z) > 64 && x % 16 == 0 && y % 16 == 0 && z % 16 == 0 {
            depth += 1;
            x /= 2;
            y /= 2;
            z /= 2;
        }
        let entries = nn * mm + mm * pp + nn * pp;
        // A recursive level uses one quarter of its parent's storage; siblings reuse it.
        let mut data = if entries + entries / 3 >= 1 << 20 {
            let mut data = Vec::with_capacity(entries + entries / 3);
            advise_huge_pages(&mut data);
            data.resize(entries + entries / 3, 0u32);
            data
        } else {
            vec![0u32; entries + entries / 3]
        };
        let quotient = (((scale as u64) << 32) / modulus as u64) as u32;
        let mut inverse = 1u32;
        for _ in 0..5 {
            inverse = inverse.wrapping_mul(2u32.wrapping_sub(modulus.wrapping_mul(inverse)));
        }
        let inverse = inverse.wrapping_neg();
        for (offset, row, col) in blocks(nn, mm, depth) {
            let (nr, nc) = (nn >> depth, mm >> depth);
            for i in row..(row + nr).min(n) {
                // SAFETY: MInt is transparent over u32. Reading raw words preserves Montgomery encoding.
                let values: &[u32] = unsafe { std::slice::from_raw_parts(a[i].as_ptr().cast(), m) };
                for j in col..(col + nc).min(m) {
                    let x = values[j];
                    let q = ((x as u64 * quotient as u64) >> 32) as u32;
                    let x = x.wrapping_mul(scale).wrapping_sub(q.wrapping_mul(modulus));
                    data[offset + (i - row) * nc + j - col] = x.min(x.wrapping_sub(modulus));
                }
            }
        }
        for (offset, row, col) in blocks(mm, pp, depth) {
            let (nr, nc) = (mm >> depth, pp >> depth);
            for i in row..(row + nr).min(m) {
                // SAFETY: MInt is transparent over u32 and row lengths were checked above.
                let values: &[u32] = unsafe { std::slice::from_raw_parts(b[i].as_ptr().cast(), p) };
                for j in col..(col + nc).min(p) {
                    data[nn * mm + offset + (i - row) * nc + j - col] = values[j];
                }
            }
        }
        let kernel = Kernel {
            modulus,
            inverse,
            avx512: avx512_enabled() && is_x86_feature_detected!("avx512f"),
        };
        // SAFETY: padding keeps every leaf dimension divisible by eight. The three matrices
        // and the geometric scratch space are disjoint parts of the allocated buffer.
        unsafe {
            let ptr = data.as_mut_ptr();
            multiply(
                ptr,
                ptr.add(nn * mm),
                ptr.add(nn * mm + mm * pp),
                (nn, mm, pp),
                ptr.add(entries),
                &kernel,
            );
        }
        let mut result = vec![vec![MInt::new_unchecked(M::mod_zero()); p]; n];
        for (offset, row, col) in blocks(nn, pp, depth) {
            let (nr, nc) = (nn >> depth, pp >> depth);
            for i in row..(row + nr).min(n) {
                for j in col..(col + nc).min(p) {
                    result[i][j] = MInt::new_unchecked(
                        data[nn * mm + mm * pp + offset + (i - row) * nc + j - col],
                    );
                }
            }
        }
        result
    }
}
