#[cfg(target_arch = "x86_64")]
use super::{SimdBackend, simd_backend};
use super::{advise_huge_pages, partial_fft};

/// Matches bytes with `?` as a wildcard on either side.
/// Requires round-to-nearest binary64 arithmetic with gradual underflow and no fast-math.
pub fn wildcard_pattern_matching(p: &[u8], s: &[u8]) -> Vec<bool> {
    assert!(!p.is_empty());
    assert!(p.len() <= s.len());
    matching_with_limit(p, s, FFT_LIMIT)
}

// The score is sum(mask_s * mask_p * (s - p)^2), an integer >= 1 on a mismatch.
// For n,m <= 2^22, 65536 * gamma_3295 * sqrt(n*m) + 2^-900 < 0.100556.
// This includes fixed-root generation, partial transforms, leaf products and the
// final addition of exact square sums. Each tile owns its outputs; scores from
// different tiles or pattern pieces are never added in floating point.
const FFT_LIMIT: usize = 1 << 22;

fn matching_with_limit(p: &[u8], s: &[u8], limit: usize) -> Vec<bool> {
    if s.len() <= limit {
        return bounded(p, s, |value| value.abs() < 0.5);
    }
    let mut result = vec![true; s.len() - p.len() + 1];
    for offset in (0..p.len()).step_by(limit / 2) {
        let part = &p[offset..offset + (p.len() - offset).min(limit / 2)];
        let step = limit - part.len() + 1;
        for start in (0..result.len()).step_by(step) {
            let count = step.min(result.len() - start);
            let text = &s[start + offset..start + offset + count + part.len() - 1];
            let values = bounded(part, text, |value| value.abs() < 0.5);
            for (answer, value) in result[start..start + count].iter_mut().zip(values) {
                *answer &= value;
            }
        }
    }
    result
}

fn bounded<T>(p: &[u8], s: &[u8], convert: impl FnMut(f64) -> T) -> Vec<T> {
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        match simd_backend() {
            SimdBackend::Avx512 => return unsafe { avx512::run(p, s, convert) },
            SimdBackend::Avx2 => return unsafe { avx2::run(p, s, convert) },
            SimdBackend::Scalar => {}
        }
    }
    scalar::run(p, s, convert)
}

fn capacity<T>(len: usize) -> Vec<T> {
    let mut result = Vec::with_capacity(len);
    advise_huge_pages(&mut result);
    result
}

fn square(ch: u8) -> u64 {
    let value = i64::from(ch) - 128;
    if ch == b'?' {
        0
    } else {
        (value * value) as u64
    }
}

// Reuse pattern transforms only when the estimated transform work decreases.
fn transform_length(p: usize, s: usize, lanes: usize, terms: usize) -> usize {
    let full = s.next_power_of_two().max(2 * lanes);
    let mut length = full;
    let mut best = (2 * terms + 1) * full * full.ilog2() as usize;
    let mut option = (2 * p - 1).next_power_of_two().max(2 * lanes);
    while option < full {
        let blocks = (s - p + 1).div_ceil(option - p + 1);
        let cost = (terms + (terms + 1) * blocks) * option * option.ilog2() as usize;
        if cost < best {
            length = option;
            best = cost;
        }
        option *= 2;
    }
    length
}

macro_rules! matching_kernel {
    ([$($qualifier:tt)*] $(#[$attr:meta])*; $block:ident, $lanes:literal) => {
        $(#[$attr])*
        pub $($qualifier)* fn run<T>(p: &[u8], s: &[u8], convert: impl FnMut(f64) -> T) -> Vec<T> {
            match (!p.contains(&b'?'), !s.contains(&b'?')) {
                (true, true) => general::<_, 1, true, true>(p, s, convert),
                (true, false) => general::<_, 2, true, false>(p, s, convert),
                (false, true) => general::<_, 2, false, true>(p, s, convert),
                (false, false) => general::<_, 3, false, false>(p, s, convert),
            }
        }

        $(#[$attr])*
        $($qualifier)* fn general<T, const TERMS: usize, const PCLEAR: bool, const SCLEAR: bool>(
            p: &[u8],
            s: &[u8],
            mut convert: impl FnMut(f64) -> T,
        ) -> Vec<T> {
            let length = transform_length(p.len(), s.len(), $lanes, TERMS);
            let n = length / 2;
            let step = length - p.len() + 1;
            let mut text: [Vec<$block>; TERMS] = std::array::from_fn(|_| capacity(n / $lanes));
            let mut pattern: [Vec<$block>; TERMS] = std::array::from_fn(|_| capacity(n / $lanes));
            pack::<TERMS, true, PCLEAR>(p, n, &mut pattern);
            for part in &mut pattern {
                fft_soa(part);
            }
            let mut window = if PCLEAR {
                s[..p.len()].iter().map(|&ch| square(ch)).sum::<u64>()
            } else {
                0
            };
            let pattern_square = if SCLEAR {
                p.iter().map(|&ch| square(ch)).sum::<u64>()
            } else {
                0
            };
            let mut result = Vec::with_capacity(s.len() - p.len() + 1);
            for start in (0..s.len() - p.len() + 1).step_by(step) {
                let count = step.min(s.len() - p.len() + 1 - start);
                let bytes = &s[start..start + count + p.len() - 1];
                pack::<TERMS, false, PCLEAR>(bytes, n, &mut text);
                for part in &mut text {
                    fft_soa(part);
                }
                with_roots(n / 2, |cache| {
                    for block in 0..text[0].len() {
                        let mut br = [splat(0.); TERMS];
                        let mut bi = br;
                        for term in 0..TERMS {
                            (br[term], bi[term]) = load(&pattern[term][block]);
                        }
                        let mut rr = splat(0.);
                        let mut ri = rr;
                        let w = eval_twiddle(cache, 1, text[0].len(), block);
                        let wr = first(w.re, 1.);
                        let wi = first(w.im, 0.);
                        for lane in 0..$lanes {
                            for term in 0..TERMS {
                                let ar = splat(text[term][block].re[lane]);
                                let ai = splat(text[term][block].im[lane]);
                                multiply_accumulate(&mut rr, &mut ri, ar, ai, br[term], bi[term]);
                            }
                            if lane != $lanes - 1 {
                                for term in 0..TERMS {
                                    br[term] = rotate(br[term]);
                                    bi[term] = rotate(bi[term]);
                                    (br[term], bi[term]) = complex_mul(br[term], bi[term], wr, wi);
                                }
                            }
                        }
                        store(&mut text[0][block], rr, ri);
                    }
                });
                ifft_soa(&mut text[0]);
                for i in p.len() - 1..p.len() - 1 + count {
                    let mut value = if i < n {
                        text[0][i / $lanes].re[i % $lanes]
                    } else {
                        text[0][(i - n) / $lanes].im[i % $lanes]
                    };
                    if PCLEAR || SCLEAR {
                        value += (window + pattern_square) as f64;
                    }
                    if PCLEAR {
                        let position = start + i - (p.len() - 1);
                        if position + 1 < s.len() - p.len() + 1 {
                            window -= square(s[position]);
                            window += square(s[position + p.len()]);
                        }
                    }
                    result.push(convert(value));
                }
            }
            result
        }

        $(#[$attr])*
        #[inline]
        $($qualifier)* fn pack<const TERMS: usize, const PATTERN: bool, const PCLEAR: bool>(
            bytes: &[u8],
            n: usize,
            parts: &mut [Vec<$block>; TERMS],
        ) {
            for part in parts.iter_mut() {
                part.clear();
            }
            for block in 0..n / $lanes {
                let (xr, mr) = values(bytes, block * $lanes, PATTERN);
                let (xi, mi) = values(bytes, block * $lanes + n, PATTERN);
                for (term, part) in parts.iter_mut().enumerate() {
                    let (re, im) = match (PATTERN, term + usize::from(PCLEAR)) {
                        (true, 0) | (false, 2) => (mr, mi),
                        (true, 1) => (mul(xr, mr), mul(xi, mi)),
                        (false, 1) => (mul(mul(splat(-2.), xr), mr), mul(mul(splat(-2.), xi), mi)),
                        _ => (mul(mul(xr, xr), mr), mul(mul(xi, xi), mi)),
                    };
                    let mut value = $block::default();
                    store(&mut value, re, im);
                    part.push(value);
                }
            }
        }
    };
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn byte_block<const LANES: usize>(bytes: &[u8], start: usize, reverse: bool) -> [u8; LANES] {
    let count = bytes.len().saturating_sub(start).min(LANES);
    let mut block = [b'?'; LANES];
    if count == LANES {
        let at = if reverse {
            bytes.len() - start - LANES
        } else {
            start
        };
        block.copy_from_slice(&bytes[at..at + LANES]);
        if reverse {
            block.reverse();
        }
    } else {
        for (lane, value) in block.iter_mut().take(count).enumerate() {
            *value = bytes[if reverse {
                bytes.len() - start - lane - 1
            } else {
                start + lane
            }];
        }
    }
    block
}

mod scalar {
    use super::partial_fft::{eval_twiddle, scalar::*, with_roots};
    use super::{capacity, square, transform_length};
    fn values(bytes: &[u8], start: usize, reverse: bool) -> ([f64; 4], [f64; 4]) {
        let count = bytes.len().saturating_sub(start).min(4);
        let mut x = [0.; 4];
        let mut mask = [0.; 4];
        for lane in 0..count {
            let ch = bytes[if reverse {
                bytes.len() - start - lane - 1
            } else {
                start + lane
            }];
            x[lane] = f64::from(ch) - 128.;
            mask[lane] = f64::from(ch != b'?');
        }
        (x, mask)
    }
    matching_kernel!([]; Complex4, 4);
}

#[cfg(target_arch = "x86_64")]
mod avx2 {
    // The dispatcher checks the target features before calling this module.
    #![allow(unsafe_op_in_unsafe_fn)]
    use super::partial_fft::{avx2::*, eval_twiddle, with_roots};
    use super::{byte_block, capacity, square, transform_length};
    #[target_feature(enable = "avx2,fma")]
    #[inline]
    unsafe fn values(
        bytes: &[u8],
        start: usize,
        reverse: bool,
    ) -> (std::arch::x86_64::__m256d, std::arch::x86_64::__m256d) {
        use std::arch::x86_64::*;
        let raw = u32::from_le_bytes(byte_block(bytes, start, reverse));
        let raw = _mm_cvtsi32_si128(raw as i32);
        let ch = _mm256_cvtepi32_pd(_mm_cvtepu8_epi32(raw));
        let x = _mm256_sub_pd(ch, _mm256_set1_pd(128.));
        let mask = _mm256_andnot_pd(
            _mm256_cmp_pd::<_CMP_EQ_OQ>(ch, _mm256_set1_pd(63.)),
            _mm256_set1_pd(1.),
        );
        (x, mask)
    }

    matching_kernel!([unsafe] #[target_feature(enable = "avx2,fma")]; Complex4, 4);
}

#[cfg(target_arch = "x86_64")]
mod avx512 {
    // The dispatcher checks the target features before calling this module.
    #![allow(unsafe_op_in_unsafe_fn)]
    use super::partial_fft::{avx512::*, eval_twiddle, with_roots};
    use super::{byte_block, capacity, square, transform_length};
    #[target_feature(enable = "avx2,avx512f,fma")]
    #[inline]
    unsafe fn values(
        bytes: &[u8],
        start: usize,
        reverse: bool,
    ) -> (std::arch::x86_64::__m512d, std::arch::x86_64::__m512d) {
        use std::arch::x86_64::*;
        let raw = u64::from_le_bytes(byte_block(bytes, start, reverse));
        let raw = _mm_cvtsi64_si128(raw as i64);
        let ch = _mm512_cvtepi32_pd(_mm256_cvtepu8_epi32(raw));
        let x = _mm512_sub_pd(ch, _mm512_set1_pd(128.));
        let mask = _mm512_maskz_mov_pd(
            _mm512_cmp_pd_mask::<_CMP_NEQ_OQ>(ch, _mm512_set1_pd(63.)),
            _mm512_set1_pd(1.),
        );
        (x, mask)
    }

    matching_kernel!([unsafe] #[target_feature(enable = "avx2,avx512f,fma")]; Complex8, 8);
}

#[cfg(test)]
mod tests {
    use super::{matching_with_limit, scalar, wildcard_pattern_matching};
    use crate::tools::Xorshift;
    use crate::tools::testutil::{
        exhaustive_sequences, integer_cases, sample_usize, sequence_cases,
    };

    #[test]
    fn test_wildcard_pattern_matching() {
        let check = |p: &[u8], s: &[u8], actual: Vec<bool>| {
            let expected: Vec<_> = s
                .windows(p.len())
                .map(|window| {
                    p.iter()
                        .zip(window)
                        .all(|(&a, &b)| a == b || a == b'?' || b == b'?')
                })
                .collect();
            assert_eq!(actual, expected, "pattern={p:?}, text={s:?}");
        };
        let mut rng = Xorshift::default();
        for n in sample_usize(&mut rng, 16, 1..=1024, 0) {
            for c in [0, 1, b'?', 127, 128, 254, 255] {
                let p = vec![c; n];
                check(&p, &p, wildcard_pattern_matching(&p, &p));
                let wildcards = vec![b'?'; n];
                check(&wildcards, &p, wildcard_pattern_matching(&wildcards, &p));
            }
        }
        let bytes: Vec<_> = (0..=u8::MAX).collect();
        for c in 0..=u8::MAX {
            check(&[c], &bytes, wildcard_pattern_matching(&[c], &bytes));
        }
        let inputs: Vec<_> = exhaustive_sequences([0, b'?', u8::MAX], 1..=5).collect();
        let exhaustive = inputs.iter().flat_map(|p| {
            inputs
                .iter()
                .filter(|s| s.len() >= p.len())
                .map(move |s| (p.clone(), s.clone()))
        });
        let mut sampled_rng = Xorshift::default();
        let sampled = sequence_cases(&mut sampled_rng, b'a'..=b'd', 1..=128).map(|mut p| {
            let n = p.len();
            let m = n + rng.random(0usize..256);
            for ch in &mut p {
                if *ch == b'd' {
                    *ch = b'?';
                }
            }
            let mut s: Vec<_> = rng
                .random_iter(b'a'..=b'd')
                .take(m)
                .map(|ch| if ch == b'd' { b'?' } else { ch })
                .collect();
            match rng.random(0..4) {
                0 => p.copy_from_slice(&s[..n]),
                1 => p.fill(b'?'),
                2 => s.fill(b'?'),
                _ => {}
            }
            (p, s)
        });
        let mut input_rng = Xorshift::default();
        let different_bytes = sequence_cases(&mut input_rng, 0..=u8::MAX, 1..=1024).map(|p| {
            let s = p.iter().map(|&c| c ^ 0x80).collect();
            (p, s)
        });
        let swapped = std::iter::once((vec![89, 180], vec![180, 89]));
        for (p, s) in exhaustive
            .chain(sampled)
            .chain(different_bytes)
            .chain(swapped)
        {
            check(&p, &s, wildcard_pattern_matching(&p, &s));
        }
        let limit = 64;
        let mut block_rng = Xorshift::default();
        for m in integer_cases(1usize..=2 * limit + 1) {
            for n in [
                m,
                m + 1,
                m + limit - 1,
                m + limit,
                m + limit + 1,
                4 * limit + 7,
            ] {
                let input: Vec<_> = block_rng
                    .random_iter(0..=u8::MAX)
                    .take(m)
                    .map(|ch| if ch == b'?' { 0 } else { ch })
                    .collect();
                for mask in 0..4 {
                    let mut p = input.clone();
                    if mask & 1 != 0 {
                        for ch in p.iter_mut().skip(1).step_by(3) {
                            *ch = b'?';
                        }
                        p[m - 1] = b'?';
                    }
                    for position in [None, Some(m / 2), Some(m - 1), Some(n - 1)] {
                        let mut s: Vec<_> = input.iter().copied().cycle().take(n).collect();
                        if let Some(i) = position {
                            s[i] ^= 1;
                            if s[i] == b'?' {
                                s[i] ^= 2;
                            }
                        }
                        if mask & 2 != 0 {
                            for ch in s.iter_mut().skip(limit).step_by(5) {
                                *ch = b'?';
                            }
                            s[n - 1] = b'?';
                        }
                        check(&p, &s, matching_with_limit(&p, &s, limit));
                    }
                }
            }
        }
    }

    #[test]
    fn test_wildcard_pattern_matching_scores() {
        let mut rng = Xorshift::default();
        let boundaries = integer_cases(1usize..=129)
            .map(|m| (0..m).map(|i| [0, 1, 127, 128, 254, 255][i % 6]).collect());
        let structured = sequence_cases(&mut rng, 0..=u8::MAX, 1..=129);
        for mut input in boundaries.chain(structured) {
            for ch in &mut input {
                if *ch == b'?' {
                    *ch = 0;
                }
            }
            let m = input.len();
            for n in [m, 2 * m + 3, 8 * m + 17, 1025] {
                for mask in 0..4 {
                    let mut p = input.clone();
                    let mut s: Vec<_> = input.iter().copied().cycle().take(n).collect();
                    s[n / 2] ^= 1;
                    if s[n / 2] == b'?' {
                        s[n / 2] ^= 2;
                    }
                    if mask & 1 != 0 {
                        for ch in p.iter_mut().step_by(5) {
                            *ch = b'?';
                        }
                    }
                    if mask & 2 != 0 {
                        for ch in s.iter_mut().step_by(7) {
                            *ch = b'?';
                        }
                    }
                    let expected: Vec<_> = s
                        .windows(m)
                        .map(|window| {
                            p.iter()
                                .zip(window)
                                .map(|(&a, &b)| {
                                    if a == b'?' || b == b'?' {
                                        0
                                    } else {
                                        let difference = i64::from(a) - i64::from(b);
                                        difference * difference
                                    }
                                })
                                .sum::<i64>() as f64
                        })
                        .collect();
                    let check = |actual: Vec<f64>| {
                        assert_eq!(actual.len(), expected.len());
                        for (actual, expected) in actual.into_iter().zip(&expected) {
                            assert!(
                                (actual - expected).abs() < 0.5,
                                "m={m}, n={n}, mask={mask}, actual={actual}, expected={expected}"
                            );
                        }
                    };
                    check(scalar::run(&p, &s, |x| x));
                    #[cfg(target_arch = "x86_64")]
                    unsafe {
                        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
                            check(super::avx2::run(&p, &s, |x| x));
                            if is_x86_feature_detected!("avx512f") {
                                check(super::avx512::run(&p, &s, |x| x));
                            }
                        }
                    }
                }
            }
        }
    }
}
