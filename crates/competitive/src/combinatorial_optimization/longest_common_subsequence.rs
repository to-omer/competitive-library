#[cfg(target_arch = "x86_64")]
use super::avx512_enabled;
use std::{collections::HashMap, hash::Hash};

pub fn longest_common_subsequence_length<T>(x: &[T], y: &[T]) -> usize
where
    T: Eq + Hash,
{
    let (x, y) = if x.len() <= y.len() { (x, y) } else { (y, x) };
    if x.is_empty() {
        return 0;
    }
    if x.len() == 1 {
        return usize::from(y.contains(&x[0]));
    }
    // SAFETY: simd_lanes checks the CPU features required by the selected lane count.
    let (mut state, x, y) = unsafe { LcsState::new(x, y, simd_lanes(x.len())) };
    state.lengths(&x, &y, false, |_| {})
}

/// Returns one longest common subsequence as increasing pairs of indices into `x` and `y`.
pub fn longest_common_subsequence<T>(x: &[T], y: &[T]) -> Vec<(usize, usize)>
where
    T: Eq + Hash,
{
    let (x, y, flipped) = if x.len() <= y.len() {
        (x, y, false)
    } else {
        (y, x, true)
    };
    let mut indices = Vec::with_capacity(x.len());
    if x.is_empty() {
        return indices;
    }
    if x.len() == 1 {
        if let Some(j) = y.iter().position(|value| value == &x[0]) {
            indices.push(if flipped { (j, 0) } else { (0, j) });
        }
        return indices;
    }
    // SAFETY: simd_lanes checks the CPU features required by the selected lane count.
    let (mut state, x, y) = unsafe { LcsState::new(x, y, simd_lanes(x.len())) };
    state.reconstruct(&x, &y, 0, 0, flipped, &mut indices);
    indices
}

fn simd_lanes(m: usize) -> usize {
    #[cfg(target_arch = "x86_64")]
    if m >= 256 && is_x86_feature_detected!("avx2") {
        return if avx512_enabled() && is_x86_feature_detected!("avx512f") {
            8
        } else {
            4
        };
    }
    let _ = m;
    1
}

struct LcsState {
    masks: Vec<u64>,
    lanes: usize,
}

impl LcsState {
    // lanes is 1, 4, or 8, with AVX2 available for 4/8 and AVX-512F for 8.
    unsafe fn new<T>(x: &[T], y: &[T], lanes: usize) -> (Self, Vec<usize>, Vec<usize>)
    where
        T: Eq + Hash,
    {
        let mut ids = HashMap::new();
        let mut pattern: Vec<_> = x
            .iter()
            .map(|value| {
                let id = ids.len();
                *ids.entry(value).or_insert(id)
            })
            .collect();
        let absent = ids.len();
        let mut text: Vec<_> = y
            .iter()
            .map(|value| ids.get(value).copied().unwrap_or(absent))
            .collect();
        drop(ids);
        let mut offsets = vec![usize::MAX; absent + 1];
        for &id in &text {
            offsets[id] = 0;
        }
        let mut mask_len = 0;
        for offset in &mut offsets[..absent] {
            if *offset == 0 {
                *offset = mask_len;
                mask_len += lanes;
            }
        }
        offsets[absent] = mask_len;
        // Values exclusive to either input remain distinct when recursion swaps the inputs.
        for offset in &mut offsets[..absent] {
            if *offset == usize::MAX {
                *offset = mask_len + lanes;
            }
        }
        for id in pattern.iter_mut().chain(&mut text) {
            *id = offsets[*id];
        }
        let state = Self {
            masks: vec![0; mask_len + 3 * lanes],
            lanes,
        };
        (state, pattern, text)
    }

    fn lengths<F>(&mut self, x: &[usize], y: &[usize], reverse: bool, mut row: F) -> usize
    where
        F: FnMut(usize),
    {
        let reversed: Vec<_>;
        let pattern = if reverse {
            reversed = x.iter().rev().copied().collect();
            &reversed
        } else {
            x
        };
        let n = y.len();
        // Seven zero-mask columns on each side cover the staggered SIMD lanes.
        let absent = self.masks.len() - self.lanes;
        let mut text = vec![absent; n + 14];
        if reverse {
            for (id, &value) in text[7..n + 7].iter_mut().zip(y.iter().rev()) {
                *id = value;
            }
        } else {
            text[7..n + 7].copy_from_slice(y);
        }
        let masks = &mut self.masks;
        let mut horizontal = vec![0; n + 14];
        let start = 0;
        #[cfg(target_arch = "x86_64")]
        let start = {
            let mut start = start;
            if self.lanes == 8 {
                start = pattern.len() / 512 * 512;
                // SAFETY: construction requires AVX-512F. Mask rows have eight words,
                // and text/horizontal have seven padding entries on each side.
                unsafe {
                    advance_avx512(&pattern[..start], &text, &mut horizontal, masks);
                }
            }
            if self.lanes >= 4 {
                let end = pattern.len() / 256 * 256;
                // SAFETY: construction requires AVX2; mask rows have at least four words,
                // and the same padding and offset bounds apply.
                unsafe {
                    advance_avx2(&pattern[start..end], &text, &mut horizontal, masks);
                }
                start = end;
            }
            start
        };
        for block in pattern[start..].chunks(64) {
            for (i, &id) in block.iter().enumerate() {
                masks[id] |= 1 << i;
            }
            // Zero bits encode increases of the LCS length along the pattern.
            let mut state = u64::MAX;
            for (&id, h) in text[7..n + 7].iter().zip(&mut horizontal[7..n + 7]) {
                let matched = state & masks[id];
                let (sum, carry) = state.overflowing_add(matched);
                let (sum, carry_in) = sum.overflowing_add(u64::from(*h));
                *h = u8::from(carry | carry_in);
                state = sum | (state ^ matched);
            }
            for &id in block {
                masks[id] = 0;
            }
        }
        let mut length = 0;
        for &h in &horizontal[7..n + 7] {
            length += usize::from(h);
            row(length);
        }
        length
    }

    fn reconstruct(
        &mut self,
        x: &[usize],
        y: &[usize],
        x_offset: usize,
        y_offset: usize,
        flipped: bool,
        indices: &mut Vec<(usize, usize)>,
    ) {
        if x.len() > y.len() {
            return self.reconstruct(y, x, y_offset, x_offset, !flipped, indices);
        }
        if x.len() <= 1 {
            if let Some(value) = x.first()
                && let Some(position) = y.iter().position(|other| other == value)
            {
                let pair = (x_offset, y_offset + position);
                indices.push(if flipped { (pair.1, pair.0) } else { pair });
            }
            return;
        }
        if x.len() <= 64 {
            for (i, &id) in x.iter().enumerate() {
                self.masks[id] |= 1 << i;
            }
            let mut state = u64::MAX;
            // Each row stores the LCS length differences along x.
            let mut rows = Vec::with_capacity(y.len());
            for &id in y {
                let matched = state & self.masks[id];
                state = state.wrapping_add(matched) | (state ^ matched);
                rows.push(!state);
            }
            for &id in x {
                self.masks[id] = 0;
            }
            let start = indices.len();
            let (mut i, mut j) = (x.len(), y.len());
            while i != 0 && j != 0 {
                if x[i - 1] == y[j - 1] {
                    let pair = (x_offset + i - 1, y_offset + j - 1);
                    indices.push(if flipped { (pair.1, pair.0) } else { pair });
                    i -= 1;
                    j -= 1;
                } else if rows[j - 1] >> (i - 1) & 1 == 0 {
                    i -= 1;
                } else {
                    j -= 1;
                }
            }
            indices[start..].reverse();
            return;
        }
        let middle = x.len() / 2;
        let mut forward = Vec::with_capacity(y.len() + 1);
        forward.push(0);
        self.lengths(&x[..middle], y, false, |score| forward.push(score));
        let mut split = y.len();
        let mut length = forward[split];
        let mut index = y.len();
        self.lengths(&x[middle..], y, true, |score| {
            index -= 1;
            let candidate = forward[index] + score;
            if candidate > length {
                length = candidate;
                split = index;
            }
        });
        drop(forward);
        if length == 0 {
            return;
        }
        self.reconstruct(
            &x[..middle],
            &y[..split],
            x_offset,
            y_offset,
            flipped,
            indices,
        );
        self.reconstruct(
            &x[middle..],
            &y[split..],
            x_offset + middle,
            y_offset + split,
            flipped,
            indices,
        );
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn advance_avx2(
    pattern: &[usize],
    text: &[usize],
    horizontal: &mut [u8],
    masks: &mut [u64],
) {
    use std::arch::x86_64::*;
    let n = text.len() - 14;
    let offsets = _mm256_set_epi64x(3, 2, 1, 0);
    for group in pattern.as_chunks::<256>().0 {
        for (i, &id) in group.iter().enumerate() {
            masks[id + i / 64] |= 1 << (i % 64);
        }
        let mut state = _mm256_set1_epi64x(-1);
        let mut carry = _mm256_setzero_si256();
        for t in 0..n + 3 {
            let incoming = _mm256_blend_epi32::<3>(
                _mm256_permute4x64_epi64::<0x93>(carry),
                _mm256_set1_epi64x(i64::from(horizontal[t + 7])),
            );
            // SAFETY: t + 4 through t + 7 are inside the padded text. Each
            // gathered index is offset + lane within a complete mask row.
            let eq = unsafe {
                let ids = _mm256_loadu_si256(text.as_ptr().add(t + 4).cast());
                let indices = _mm256_add_epi64(_mm256_permute4x64_epi64::<0x1b>(ids), offsets);
                _mm256_i64gather_epi64::<8>(masks.as_ptr().cast(), indices)
            };
            let matched = _mm256_and_si256(state, eq);
            let sum = _mm256_add_epi64(_mm256_add_epi64(state, matched), incoming);
            carry =
                _mm256_srli_epi64::<63>(_mm256_or_si256(matched, _mm256_andnot_si256(sum, state)));
            horizontal[t + 4] = _mm256_extract_epi64::<3>(carry) as u8;
            state = _mm256_or_si256(sum, _mm256_xor_si256(state, matched));
        }
        for (i, &id) in group.iter().enumerate() {
            masks[id + i / 64] = 0;
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn advance_avx512(
    pattern: &[usize],
    text: &[usize],
    horizontal: &mut [u8],
    masks: &mut [u64],
) {
    use std::arch::x86_64::*;
    let n = text.len() - 14;
    let offsets = _mm512_set_epi64(7, 6, 5, 4, 3, 2, 1, 0);
    let reverse = _mm512_set_epi64(0, 1, 2, 3, 4, 5, 6, 7);
    let rotate = _mm512_set_epi64(6, 5, 4, 3, 2, 1, 0, 7);
    for group in pattern.as_chunks::<512>().0 {
        for (i, &id) in group.iter().enumerate() {
            masks[id + i / 64] |= 1 << (i % 64);
        }
        let mut state = _mm512_set1_epi64(-1);
        let mut carry = _mm512_setzero_si512();
        for t in 0..n + 7 {
            let incoming = _mm512_mask_mov_epi64(
                _mm512_permutexvar_epi64(rotate, carry),
                1,
                _mm512_set1_epi64(i64::from(horizontal[t + 7])),
            );
            // SAFETY: t through t + 7 are inside the padded text. Each gathered
            // index is offset + lane within a complete mask row.
            let eq = unsafe {
                let ids = _mm512_loadu_si512(text.as_ptr().add(t).cast());
                let indices = _mm512_add_epi64(_mm512_permutexvar_epi64(reverse, ids), offsets);
                _mm512_i64gather_epi64::<8>(indices, masks.as_ptr().cast())
            };
            let matched = _mm512_and_si512(state, eq);
            let sum = _mm512_add_epi64(_mm512_add_epi64(state, matched), incoming);
            carry =
                _mm512_srli_epi64::<63>(_mm512_or_si512(matched, _mm512_andnot_si512(sum, state)));
            let out = _mm512_extracti32x4_epi32::<3>(carry);
            horizontal[t] = _mm_cvtsi128_si64(_mm_srli_si128::<8>(out)) as u8;
            state = _mm512_or_si512(sum, _mm512_xor_si512(state, matched));
        }
        for (i, &id) in group.iter().enumerate() {
            masks[id + i / 64] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Xorshift, testutil::exhaustive_sequences};

    fn naive_lengths<T>(x: &[T], y: &[T]) -> Vec<usize>
    where
        T: Eq,
    {
        let mut dp = vec![0; x.len() + 1];
        let mut row = vec![0];
        for b in y {
            let mut diagonal = 0;
            for (i, a) in x.iter().enumerate() {
                let old = dp[i + 1];
                dp[i + 1] = if a == b { diagonal + 1 } else { old.max(dp[i]) };
                diagonal = old;
            }
            row.push(dp[x.len()]);
        }
        row
    }

    fn check<T>(x: &[T], y: &[T])
    where
        T: Eq + Hash,
    {
        let (x, y) = if x.len() <= y.len() { (x, y) } else { (y, x) };
        let expected = naive_lengths(x, y);
        assert_eq!(longest_common_subsequence_length(x, y), expected[y.len()]);
        assert_eq!(longest_common_subsequence_length(y, x), expected[y.len()]);
        for lanes in [1, 4, 8] {
            #[cfg(not(target_arch = "x86_64"))]
            if lanes != 1 {
                continue;
            }
            #[cfg(target_arch = "x86_64")]
            if (lanes >= 4 && !is_x86_feature_detected!("avx2"))
                || (lanes == 8 && !is_x86_feature_detected!("avx512f"))
            {
                continue;
            }
            let mut row = vec![0];
            // SAFETY: the CPU features required by each lane count were checked.
            let (mut state, x, y) = unsafe { LcsState::new(x, y, lanes) };
            let actual = state.lengths(&x, &y, false, |score| row.push(score));
            assert_eq!(actual, expected[y.len()]);
            assert_eq!(row, expected);
        }
    }

    fn check_reconstruction<T>(x: &[T], y: &[T])
    where
        T: Eq + Hash,
    {
        let expected = naive_lengths(x, y)[y.len()];
        for (x, y) in [(x, y), (y, x)] {
            let indices = longest_common_subsequence(x, y);
            assert_eq!(indices.len(), expected);
            assert!(indices.iter().all(|&(i, j)| x[i] == y[j]));
            assert!(
                indices
                    .windows(2)
                    .all(|pair| pair[0].0 < pair[1].0 && pair[0].1 < pair[1].1)
            );
        }
    }

    #[test]
    fn test_longest_common_subsequence_length() {
        let sequences: Vec<_> = exhaustive_sequences(0u8..2, 0..=6).collect();
        for x in &sequences {
            for y in &sequences {
                check(x, y);
            }
        }
        let mut rng = Xorshift::default();
        for m in [0, 1, 2].into_iter().chain(
            [1, 2, 4, 8, 12, 16]
                .into_iter()
                .flat_map(|blocks| blocks * 64 - 1..=blocks * 64 + 1),
        ) {
            for n in [m, m + 1, 2 * m + 3] {
                for alphabet in [2, 26, 100_000] {
                    let x: Vec<u32> = (0..m).map(|_| rng.random(0..alphabet)).collect();
                    let y: Vec<u32> = (0..n).map(|_| rng.random(0..alphabet)).collect();
                    check(&x, &y);
                }
            }
            let x: Vec<u32> = (0..m).map(|i| (i % 2) as u32).collect();
            check(&x, &x);
            check(&vec![0u32; m], &vec![1u32; m]);
            check(&vec![0u32; m], &vec![0u32; m + 7]);
            let mut y = x.clone();
            y.rotate_left(m / 2);
            y.insert(0, u32::MAX);
            check(&x, &y);
        }
        #[derive(PartialEq, Eq, Hash)]
        struct Token(u32);
        let x: Vec<_> = (0..513).map(Token).collect();
        let y: Vec<_> = (100..700).map(Token).collect();
        check(&x, &y);
    }

    #[test]
    fn test_longest_common_subsequence() {
        let sequences: Vec<_> = exhaustive_sequences(0u8..2, 0..=5).collect();
        for x in &sequences {
            for y in &sequences {
                check_reconstruction(x, y);
            }
        }
        let mut rng = Xorshift::default();
        // Halves cross scalar, AVX2, AVX-512, and mixed-kernel boundaries.
        for m in [1, 2, 8, 16, 24]
            .into_iter()
            .flat_map(|blocks| blocks * 64 - 1..=blocks * 64 + 1)
        {
            for alphabet in [2, 26, 100_000] {
                let x: Vec<u32> = (0..m).map(|_| rng.random(0..alphabet)).collect();
                let y: Vec<u32> = (0..m + 7).map(|_| rng.random(0..alphabet)).collect();
                check_reconstruction(&x, &y);
            }
            let x: Vec<u32> = (0..m).map(|i| (i % 2) as u32).collect();
            check_reconstruction(&x, &x);
            check_reconstruction(&vec![0u32; m], &vec![1u32; m]);
            check_reconstruction(&vec![0u32; m], &vec![0u32; m + 7]);
            let mut y = x.clone();
            y.drain(m / 4..m / 2);
            y.extend(std::iter::repeat_n(u32::MAX, m / 2));
            check_reconstruction(&x, &y);
        }
        check_reconstruction(&[1], &vec![0; 4096]);
        check_reconstruction(&[0, 1], &vec![1; 4096]);
        #[derive(PartialEq, Eq, Hash)]
        struct Token(u32);
        let x: Vec<_> = (0..1538).map(Token).collect();
        let y: Vec<_> = (100..1700).map(Token).collect();
        check_reconstruction(&x, &y);
    }
}
