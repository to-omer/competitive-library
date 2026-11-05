#[cfg(target_arch = "x86_64")]
use super::avx512_enabled;
use std::{collections::HashMap, hash::Hash};

/// Returns the edit distance with unit insertion, deletion, and substitution costs.
/// Uses O(m + n + n ceil(m / 64)) expected time and O(m + n) space, where m <= n.
pub fn levenshtein_distance<T>(x: &[T], y: &[T]) -> usize
where
    T: Eq + Hash,
{
    let (x, y) = if x.len() <= y.len() { (x, y) } else { (y, x) };
    if x.is_empty() {
        return y.len();
    }
    levenshtein_distance_impl(x.iter(), y.iter(), |_| {})
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevenshteinEdit {
    Match,
    Substitute,
    Insert,
    Delete,
}

/// Returns the unit-cost distance and one optimal edit sequence from `x` to `y`.
/// Edits run left to right: `Insert` consumes one element of `y`, `Delete` of `x`,
/// and `Match`/`Substitute` of each.
pub fn levenshtein_distance_with_edits<T>(x: &[T], y: &[T]) -> (usize, Vec<LevenshteinEdit>)
where
    T: Eq + Hash,
{
    let mut edits = Vec::with_capacity(x.len().max(y.len()));
    let distance = levenshtein_edits(x, y, false, &mut edits);
    (distance, edits)
}

fn levenshtein_edits<T>(x: &[T], y: &[T], flipped: bool, edits: &mut Vec<LevenshteinEdit>) -> usize
where
    T: Eq + Hash,
{
    if x.len() > y.len() {
        return levenshtein_edits(y, x, !flipped, edits);
    }
    if x.len() <= 1 {
        let insert = if flipped {
            LevenshteinEdit::Delete
        } else {
            LevenshteinEdit::Insert
        };
        if x.is_empty() {
            edits.extend(std::iter::repeat_n(insert, y.len()));
            return y.len();
        }
        let position = y.iter().position(|value| value == &x[0]);
        let index = position.unwrap_or(0);
        edits.extend(std::iter::repeat_n(insert, index));
        edits.push(if position.is_some() {
            LevenshteinEdit::Match
        } else {
            LevenshteinEdit::Substitute
        });
        edits.extend(std::iter::repeat_n(insert, y.len() - index - 1));
        return y.len() - usize::from(position.is_some());
    }
    let middle = x.len() / 2;
    let mut forward = Vec::with_capacity(y.len() + 1);
    forward.push(middle);
    levenshtein_distance_impl(x[..middle].iter(), y.iter(), |score| forward.push(score));
    let mut split = y.len();
    let mut distance = forward[split] + x.len() - middle;
    let mut index = y.len();
    // Reversed prefix scores are suffix scores in the original coordinates.
    levenshtein_distance_impl(x[middle..].iter().rev(), y.iter().rev(), |score| {
        index -= 1;
        let candidate = forward[index] + score;
        if candidate < distance {
            distance = candidate;
            split = index;
        }
    });
    drop(forward);
    levenshtein_edits(&x[..middle], &y[..split], flipped, edits);
    levenshtein_edits(&x[middle..], &y[split..], flipped, edits);
    distance
}

fn levenshtein_distance_impl<I, F>(x: I, y: I, row: F) -> usize
where
    I: ExactSizeIterator,
    I::Item: Eq + Hash,
    F: FnMut(usize),
{
    #[cfg(target_arch = "x86_64")]
    if x.len() >= 256 && is_x86_feature_detected!("avx2") {
        let avx512 = avx512_enabled() && is_x86_feature_detected!("avx512f");
        // SAFETY: AVX2 and, when requested, AVX-512F support were checked.
        return unsafe { levenshtein_distance_simd(x, y, avx512, row) };
    }
    levenshtein_distance_scalar(x, y, row)
}

fn levenshtein_distance_scalar<I, F>(x: I, y: I, mut row: F) -> usize
where
    I: ExactSizeIterator,
    I::Item: Eq + Hash,
    F: FnMut(usize),
{
    let m = x.len();
    let mut masks: HashMap<I::Item, Vec<(usize, u64)>> = HashMap::new();
    for (i, value) in x.enumerate() {
        let entries = masks.entry(value).or_default();
        let block = i / 64;
        if let Some(last) = entries.last_mut().filter(|last| last.0 == block) {
            last.1 |= 1 << (i % 64);
        } else {
            entries.push((block, 1 << (i % 64)));
        }
    }
    let mut state = vec![(u64::MAX, 0); m.div_ceil(64)];
    let top = 1 << ((m - 1) % 64);
    let mut distance = m;
    for value in y {
        let mut entries = masks.get(&value).map(Vec::as_slice).unwrap_or(&[]);
        let (mut carry, mut hp, mut hm) = (false, 1, 0);
        let mut last = (0, 0);
        for (block, (pv, mv)) in state.iter_mut().enumerate() {
            let eq = match entries.first() {
                Some(&(index, mask)) if index == block => {
                    entries = &entries[1..];
                    mask
                }
                _ => 0,
            };
            let xv = eq | *mv;
            let (sum, overflow) = (eq & *pv).overflowing_add(*pv);
            let (sum, overflow_carry) = sum.overflowing_add(u64::from(carry));
            carry = overflow | overflow_carry;
            let xh = (sum ^ *pv) | eq;
            let ph = *mv | !(xh | *pv);
            let mh = *pv & xh;
            let ph_shifted = (ph << 1) | hp;
            let mh_shifted = (mh << 1) | hm;
            hp = ph >> 63;
            hm = mh >> 63;
            *pv = mh_shifted | !(xv | ph_shifted);
            *mv = ph_shifted & xv;
            last = (ph, mh);
        }
        distance += usize::from(last.0 & top != 0);
        distance -= usize::from(last.1 & top != 0);
        row(distance);
    }
    distance
}

#[cfg(target_arch = "x86_64")]
unsafe fn levenshtein_distance_simd<I, F>(x: I, y: I, avx512: bool, mut row: F) -> usize
where
    I: ExactSizeIterator,
    I::Item: Eq + Hash,
    F: FnMut(usize),
{
    // Adjacent lane masks reuse cache lines across the staggered steps.
    let lanes = if avx512 { 8 } else { 4 };
    let m = x.len();
    let mut ids = HashMap::new();
    let mut pattern: Vec<_> = x
        .map(|value| {
            let id = ids.len();
            *ids.entry(value).or_insert(id)
        })
        .collect();
    let absent = ids.len();
    let n = y.len();
    // Seven zero-mask columns on each side cover the staggered SIMD lanes.
    let mut text = vec![absent; n + 14];
    for (id, value) in text[7..n + 7].iter_mut().zip(y) {
        *id = ids.get(&value).copied().unwrap_or(absent);
    }
    drop(ids);
    // Only values shared by both sequences need distinct mask rows.
    let mut offsets = vec![usize::MAX; absent + 1];
    for &id in &text[7..n + 7] {
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
    // Pattern-only values must not share the zero mask used by text padding.
    for offset in &mut offsets[..absent] {
        if *offset == usize::MAX {
            *offset = mask_len + lanes;
        }
    }
    for id in pattern.iter_mut().chain(&mut text) {
        *id = offsets[*id];
    }
    drop(offsets);
    let mut masks = vec![0; mask_len + 2 * lanes];
    let mut horizontal = vec![0; n + 14];
    horizontal[7..n + 7].fill(1);
    let mut start = 0;
    if avx512 {
        start = pattern.len() / 512 * 512;
        // SAFETY: the caller checked AVX-512F. Mask offsets identify complete
        // eight-word rows, and text/horizontal have seven padding entries on each side.
        unsafe {
            advance_avx512(&pattern[..start], &text, &mut horizontal, &mut masks);
        }
    }
    let end = pattern.len() / 256 * 256;
    // SAFETY: the caller checked AVX2. The same padding and offset bounds apply;
    // each mask row has at least four words, and pattern chunks are complete.
    unsafe {
        advance_avx2(&pattern[start..end], &text, &mut horizontal, &mut masks);
    }
    for block in pattern[end..].chunks(64) {
        for (i, &id) in block.iter().enumerate() {
            masks[id] |= 1 << i;
        }
        let shift = block.len() - 1;
        let (mut pv, mut mv) = (u64::MAX, 0);
        for (&id, h) in text[7..n + 7].iter().zip(&mut horizontal[7..n + 7]) {
            let neg = u64::from(*h < 0);
            let pos = u64::from(*h > 0);
            let eq = masks[id];
            let xv = eq | mv;
            let eq = eq | neg;
            let xh = ((eq & pv).wrapping_add(pv) ^ pv) | eq;
            let ph = mv | !(xh | pv);
            let mh = pv & xh;
            *h = ((ph >> shift) & 1) as i8 - ((mh >> shift) & 1) as i8;
            let ph = (ph << 1) | pos;
            let mh = (mh << 1) | neg;
            pv = mh | !(xv | ph);
            mv = ph & xv;
        }
        for &id in block {
            masks[id] = 0;
        }
    }
    let mut delta = 0isize;
    for &h in &horizontal[7..n + 7] {
        delta += h as isize;
        row((m as isize + delta) as usize);
    }
    (m as isize + delta) as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn advance_avx2(
    pattern: &[usize],
    text: &[usize],
    horizontal: &mut [i8],
    masks: &mut [u64],
) {
    use std::arch::x86_64::*;
    let n = text.len() - 14;
    let offsets = _mm256_set_epi64x(3, 2, 1, 0);
    let all = _mm256_set1_epi64x(-1);
    for group in pattern.as_chunks::<256>().0 {
        for (i, &id) in group.iter().enumerate() {
            masks[id + i / 64] |= 1 << (i % 64);
        }
        let mut pv = all;
        let mut mv = _mm256_setzero_si256();
        let mut hp = _mm256_setzero_si256();
        let mut hm = _mm256_setzero_si256();
        for t in 0..n + 3 {
            let incoming = horizontal[t + 7];
            let pos = _mm256_blend_epi32::<3>(
                _mm256_permute4x64_epi64::<0x93>(hp),
                _mm256_set1_epi64x(i64::from(incoming > 0)),
            );
            let neg = _mm256_blend_epi32::<3>(
                _mm256_permute4x64_epi64::<0x93>(hm),
                _mm256_set1_epi64x(i64::from(incoming < 0)),
            );
            // SAFETY: t + 4 through t + 7 are inside the padded text. Each
            // gathered index is offset + lane within a complete mask row.
            let eq = unsafe {
                let ids = _mm256_loadu_si256(text.as_ptr().add(t + 4).cast());
                let indices = _mm256_add_epi64(_mm256_permute4x64_epi64::<0x1b>(ids), offsets);
                _mm256_i64gather_epi64::<8>(masks.as_ptr().cast(), indices)
            };
            let xv = _mm256_or_si256(eq, mv);
            let eq = _mm256_or_si256(eq, neg);
            let xh = _mm256_or_si256(
                _mm256_xor_si256(_mm256_add_epi64(_mm256_and_si256(eq, pv), pv), pv),
                eq,
            );
            let ph = _mm256_or_si256(mv, _mm256_xor_si256(_mm256_or_si256(xh, pv), all));
            let mh = _mm256_and_si256(pv, xh);
            hp = _mm256_srli_epi64::<63>(ph);
            hm = _mm256_srli_epi64::<63>(mh);
            horizontal[t + 4] = _mm256_extract_epi64::<3>(_mm256_sub_epi64(hp, hm)) as i8;
            let ph = _mm256_or_si256(_mm256_slli_epi64::<1>(ph), pos);
            let mh = _mm256_or_si256(_mm256_slli_epi64::<1>(mh), neg);
            pv = _mm256_or_si256(mh, _mm256_xor_si256(_mm256_or_si256(xv, ph), all));
            mv = _mm256_and_si256(ph, xv);
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
    horizontal: &mut [i8],
    masks: &mut [u64],
) {
    use std::arch::x86_64::*;
    let n = text.len() - 14;
    let offsets = _mm512_set_epi64(7, 6, 5, 4, 3, 2, 1, 0);
    let reverse = _mm512_set_epi64(0, 1, 2, 3, 4, 5, 6, 7);
    let rotate = _mm512_set_epi64(6, 5, 4, 3, 2, 1, 0, 7);
    let all = _mm512_set1_epi64(-1);
    for group in pattern.as_chunks::<512>().0 {
        for (i, &id) in group.iter().enumerate() {
            masks[id + i / 64] |= 1 << (i % 64);
        }
        let mut pv = all;
        let mut mv = _mm512_setzero_si512();
        let mut hp = _mm512_setzero_si512();
        let mut hm = _mm512_setzero_si512();
        for t in 0..n + 7 {
            let incoming = horizontal[t + 7];
            let pos = _mm512_mask_mov_epi64(
                _mm512_permutexvar_epi64(rotate, hp),
                1,
                _mm512_set1_epi64(i64::from(incoming > 0)),
            );
            let neg = _mm512_mask_mov_epi64(
                _mm512_permutexvar_epi64(rotate, hm),
                1,
                _mm512_set1_epi64(i64::from(incoming < 0)),
            );
            // SAFETY: t through t + 7 are inside the padded text. Each gathered
            // index is offset + lane within a complete mask row.
            let eq = unsafe {
                let ids = _mm512_loadu_si512(text.as_ptr().add(t).cast());
                let indices = _mm512_add_epi64(_mm512_permutexvar_epi64(reverse, ids), offsets);
                _mm512_i64gather_epi64::<8>(indices, masks.as_ptr().cast())
            };
            let xv = _mm512_or_si512(eq, mv);
            let eq = _mm512_or_si512(eq, neg);
            let xh = _mm512_or_si512(
                _mm512_xor_si512(_mm512_add_epi64(_mm512_and_si512(eq, pv), pv), pv),
                eq,
            );
            let ph = _mm512_or_si512(mv, _mm512_xor_si512(_mm512_or_si512(xh, pv), all));
            let mh = _mm512_and_si512(pv, xh);
            hp = _mm512_srli_epi64::<63>(ph);
            hm = _mm512_srli_epi64::<63>(mh);
            let out = _mm512_extracti32x4_epi32::<3>(_mm512_sub_epi64(hp, hm));
            horizontal[t] = _mm_cvtsi128_si64(_mm_srli_si128::<8>(out)) as i8;
            let ph = _mm512_or_si512(_mm512_slli_epi64::<1>(ph), pos);
            let mh = _mm512_or_si512(_mm512_slli_epi64::<1>(mh), neg);
            pv = _mm512_or_si512(mh, _mm512_xor_si512(_mm512_or_si512(xv, ph), all));
            mv = _mm512_and_si512(ph, xv);
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

    fn naive_distances<T>(x: &[T], y: &[T]) -> Vec<usize>
    where
        T: Eq,
    {
        let mut dp: Vec<_> = (0..=x.len()).collect();
        let mut scores = vec![x.len()];
        for (j, b) in y.iter().enumerate() {
            let mut diagonal = dp[0];
            dp[0] = j + 1;
            for (i, a) in x.iter().enumerate() {
                let old = dp[i + 1];
                dp[i + 1] = (old.min(dp[i]) + 1).min(diagonal + usize::from(a != b));
                diagonal = old;
            }
            scores.push(dp[x.len()]);
        }
        scores
    }

    fn check<T>(x: &[T], y: &[T])
    where
        T: Eq + Hash,
    {
        let (x, y) = if x.len() <= y.len() { (x, y) } else { (y, x) };
        let scores = naive_distances(x, y);
        let expected = scores[y.len()];
        assert_eq!(levenshtein_distance(x, y), expected);
        assert_eq!(levenshtein_distance(y, x), expected);
        if x.is_empty() {
            return;
        }
        let mut row = vec![x.len()];
        assert_eq!(
            levenshtein_distance_scalar(x.iter(), y.iter(), |score| row.push(score)),
            expected
        );
        assert_eq!(row, scores);
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") {
            for avx512 in [false, true] {
                if avx512 && !is_x86_feature_detected!("avx512f") {
                    continue;
                }
                row.truncate(1);
                // SAFETY: AVX2 and, when requested, AVX-512F support were checked.
                let actual = unsafe {
                    levenshtein_distance_simd(x.iter(), y.iter(), avx512, |score| row.push(score))
                };
                assert_eq!(actual, expected);
                assert_eq!(row, scores);
            }
        }
    }

    fn check_edits<T>(x: &[T], y: &[T])
    where
        T: Eq + Hash,
    {
        let expected = naive_distances(x, y)[y.len()];
        for (x, y) in [(x, y), (y, x)] {
            let (distance, edits) = levenshtein_distance_with_edits(x, y);
            assert_eq!(distance, expected);
            let (mut i, mut j, mut cost) = (0, 0, 0);
            let mut restored = Vec::new();
            for edit in edits {
                cost += usize::from(edit != LevenshteinEdit::Match);
                match edit {
                    LevenshteinEdit::Match => {
                        assert!(x[i] == y[j]);
                        restored.push(&x[i]);
                        i += 1;
                        j += 1;
                    }
                    LevenshteinEdit::Substitute => {
                        assert!(x[i] != y[j]);
                        restored.push(&y[j]);
                        i += 1;
                        j += 1;
                    }
                    LevenshteinEdit::Insert => {
                        restored.push(&y[j]);
                        j += 1;
                    }
                    LevenshteinEdit::Delete => {
                        assert!(i < x.len());
                        i += 1;
                    }
                }
            }
            assert_eq!((i, j, cost), (x.len(), y.len(), distance));
            assert!(restored.into_iter().eq(y));
        }
    }

    #[test]
    fn test_levenshtein_distance() {
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
            let mut y = x.clone();
            y.insert(0, u32::MAX);
            check(&x, &y);
            for i in [0, m / 2, m.saturating_sub(1)] {
                if let Some(value) = y.get_mut(i) {
                    *value = u32::MAX;
                }
            }
            check(&x, &y);
        }
        let x: Vec<_> = "日本語の編集距離".chars().collect();
        let y: Vec<_> = "日本語と編集距離".chars().collect();
        check(&x, &y);
        #[derive(PartialEq, Eq, Hash)]
        struct Token(u32);
        let x: Vec<_> = (0..513).map(Token).collect();
        let y: Vec<_> = (100..700).map(Token).collect();
        check(&x, &y);
    }

    #[test]
    fn test_levenshtein_distance_with_edits() {
        let sequences: Vec<_> = exhaustive_sequences(0u8..2, 0..=5).collect();
        for x in &sequences {
            for y in &sequences {
                check_edits(x, y);
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
                check_edits(&x, &y);
            }
            let x: Vec<u32> = (0..m).map(|i| (i % 2) as u32).collect();
            check_edits(&x, &x);
            check_edits(&vec![0u32; m], &vec![1u32; m]);
            let mut y = x.clone();
            y.insert(0, u32::MAX);
            y.drain(m / 4..m / 2);
            y.extend(std::iter::repeat_n(u32::MAX, m / 2));
            check_edits(&x, &y);
        }
        check_edits(&[1], &vec![0; 4096]);
        check_edits(&[0, 1], &vec![1; 4096]);
        let x: Vec<_> = "日本語の編集距離".chars().collect();
        let y: Vec<_> = "日本語と編集距離".chars().collect();
        check_edits(&x, &y);
        #[derive(PartialEq, Eq, Hash)]
        struct Token(u32);
        let x: Vec<_> = (0..1538).map(Token).collect();
        let y: Vec<_> = (100..1700).map(Token).collect();
        check_edits(&x, &y);
    }
}
