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
    let prefix = levenshtein_common_prefix(x, y);
    let (x, y) = (&x[prefix..], &y[prefix..]);
    if x.is_empty() {
        return y.len();
    }
    let suffix = if x.last() == y.last() {
        let blocks = x
            .rchunks_exact(32)
            .zip(y.rchunks_exact(32))
            .take_while(|(a, b)| a == b)
            .count();
        let suffix = blocks * 32;
        suffix
            + x[..x.len() - suffix]
                .iter()
                .rev()
                .zip(y[..y.len() - suffix].iter().rev())
                .take_while(|(a, b)| a == b)
                .count()
    } else {
        0
    };
    let (x, y) = (&x[..x.len() - suffix], &y[..y.len() - suffix]);
    if x.is_empty() {
        return y.len();
    }
    let work = y.len().saturating_mul(x.len().div_ceil(64));
    if work >= 65_536
        && let Some(distance) = levenshtein_wavefront(x, y, (work / 64).clamp(2048, 1 << 22))
    {
        return distance;
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

fn levenshtein_common_prefix<T>(x: &[T], y: &[T]) -> usize
where
    T: Eq,
{
    if x.first() != y.first() {
        return 0;
    }
    let blocks = x
        .as_chunks::<32>()
        .0
        .iter()
        .zip(y.as_chunks().0)
        .take_while(|(a, b)| a == b)
        .count();
    let prefix = blocks * 32;
    prefix
        + x[prefix..]
            .iter()
            .zip(&y[prefix..])
            .take_while(|(a, b)| a == b)
            .count()
}

fn levenshtein_wavefront<T>(x: &[T], y: &[T], mut budget: usize) -> Option<usize>
where
    T: Eq,
{
    let limit = budget.isqrt().min(x.len().max(y.len()));
    if x.len().abs_diff(y.len()) > limit {
        return None;
    }
    let first = levenshtein_common_prefix(x, y);
    if first == x.len() && first == y.len() {
        return Some(0);
    }
    budget = budget.checked_sub(1 + first / 32)?;
    let base = limit + 1;
    let mut previous = vec![-1isize; 2 * limit + 3];
    let mut current = previous.clone();
    previous[base] = first as isize;
    for distance in 1..=limit {
        for diagonal in -(distance.min(y.len()) as isize)..=distance.min(x.len()) as isize {
            budget = budget.checked_sub(1)?;
            let index = (base as isize + diagonal) as usize;
            let delete = previous[index - 1];
            let substitute = previous[index];
            let insert = previous[index + 1];
            let mut position = -1;
            if delete >= 0 && delete < x.len() as isize {
                position = delete + 1;
            }
            if substitute >= 0
                && substitute < x.len() as isize
                && substitute - diagonal < y.len() as isize
            {
                position = position.max(substitute + 1);
            }
            if insert >= 0 && insert - (diagonal + 1) < y.len() as isize {
                position = position.max(insert);
            }
            if position >= 0 {
                let matched = levenshtein_common_prefix(
                    &x[position as usize..],
                    &y[(position - diagonal) as usize..],
                );
                position += matched as isize;
                if position == x.len() as isize && position - diagonal == y.len() as isize {
                    return Some(distance);
                }
                // One match run can exceed the budget, adding at most linear work.
                budget = budget.checked_sub(1 + matched / 32)?;
            }
            current[index] = position;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    None
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
    let mut y = y;
    let mut skipped = 0;
    let first = y.find(|value| {
        if masks.contains_key(value) {
            true
        } else {
            skipped += 1;
            row(m.max(skipped));
            false
        }
    });
    let Some(first) = first else {
        return m.max(skipped);
    };
    // Before the first shared value, DP(i, skipped) = max(i, skipped).
    let mut state: Vec<_> = (0..m.div_ceil(64))
        .map(|block| {
            let shift = skipped.saturating_sub(block * 64).min(64) as u32;
            (u64::MAX.checked_shl(shift).unwrap_or(0), 0)
        })
        .collect();
    let top = 1 << ((m - 1) % 64);
    let mut distance = m.max(skipped);
    for value in std::iter::once(first).chain(y) {
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
    let lanes = if avx512 { 8 } else { 4 };
    let m = x.len();
    let n = y.len();
    let mut ids = HashMap::new();
    let mut pattern: Vec<_> = x
        .map(|value| {
            let id = ids.len();
            *ids.entry(value).or_insert(id)
        })
        .collect();
    let absent = ids.len();
    // Padding covers the staggered lanes when short mask tiles use gather.
    let mut text = vec![absent; n + 14];
    for (id, value) in text[7..n + 7].iter_mut().zip(y) {
        *id = ids.get(&value).copied().unwrap_or(absent);
    }
    drop(ids);
    let mut offsets = vec![usize::MAX; absent + 1];
    for &id in &text[7..n + 7] {
        offsets[id] = 0;
    }
    let shared = offsets[..absent]
        .iter()
        .filter(|&&offset| offset == 0)
        .count();
    if shared == 0 {
        for len in 1..=n {
            row(m.max(len));
        }
        return m.max(n);
    }
    // Bound dense mask storage by O(m + n), even for a large alphabet.
    let stride = (m.saturating_add(n) / (shared + 4)).max(lanes) / lanes * lanes;
    let stride = stride.min(m.div_ceil(64).next_multiple_of(lanes));
    // Short tiles favor staggered lanes over scanning carries between words.
    let dense = stride >= 32;
    let stride = if dense { stride } else { lanes };
    let mut mask_len = 0;
    for offset in &mut offsets[..absent] {
        if *offset == 0 {
            *offset = mask_len;
            mask_len += stride;
        }
    }
    offsets[absent] = mask_len;
    // Pattern-only values must not overwrite the zero mask for text-only values.
    for offset in &mut offsets[..absent] {
        if *offset == usize::MAX {
            *offset = mask_len + stride;
        }
    }
    for id in pattern.iter_mut().chain(&mut text) {
        *id = offsets[*id];
    }
    drop(offsets);
    let mut horizontal = vec![0; n + 14];
    horizontal[7..n + 7].fill(1);
    if dense {
        let storage_len = mask_len + 4 * stride;
        let mut storage = vec![0u64; storage_len + 7];
        let alignment = storage.as_ptr().align_offset(64);
        let storage = &mut storage[alignment..alignment + storage_len];
        let (masks, state) = storage.split_at_mut(mask_len + 2 * stride);
        let (positive, negative) = state.split_at_mut(stride);
        // SAFETY: the caller checked the requested ISA. Storage starts on a
        // 64-byte boundary; every mask/state row has a whole number of vectors.
        unsafe {
            if avx512 {
                advance_dense_avx512(
                    &pattern,
                    &text[7..n + 7],
                    &mut horizontal[7..n + 7],
                    masks,
                    positive,
                    negative,
                );
            } else {
                advance_dense_avx2(
                    &pattern,
                    &text[7..n + 7],
                    &mut horizontal[7..n + 7],
                    masks,
                    positive,
                    negative,
                );
            }
        }
    } else {
        let mut masks = vec![0; mask_len + 2 * lanes];
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
unsafe fn advance_dense_avx2(
    pattern: &[usize],
    text: &[usize],
    horizontal: &mut [i8],
    masks: &mut [u64],
    positive: &mut [u64],
    negative: &mut [u64],
) {
    use std::arch::x86_64::*;
    let stride = positive.len();
    let all = _mm256_set1_epi64x(-1);
    let one = _mm256_set1_epi64x(1);
    let positions = _mm256_set_epi64x(3, 2, 1, 0);
    for group in pattern.chunks(stride * 64) {
        for (i, &offset) in group.iter().enumerate() {
            masks[offset + i / 64] |= 1 << (i % 64);
        }
        let words = group.len().div_ceil(64).next_multiple_of(4);
        let positive = &mut positive[..words];
        let negative = &mut negative[..words];
        positive.fill(u64::MAX);
        negative.fill(0);
        let mut top = [0u64; 4];
        top[((group.len() - 1) / 64) % 4] = 1 << ((group.len() - 1) % 64);
        // SAFETY: top contains one complete, possibly unaligned vector.
        let top = unsafe { _mm256_loadu_si256(top.as_ptr().cast()) };
        for (&offset, h) in text.iter().zip(horizontal.iter_mut()) {
            let mut carry = u32::from(*h < 0);
            let mut ph = _mm256_set1_epi64x(i64::from(*h > 0) << 63);
            let mut mh = _mm256_set1_epi64x(i64::from(*h < 0) << 63);
            for ((p, m), mask) in positive
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(negative.as_chunks_mut::<4>().0.iter_mut())
                .zip(masks[offset..offset + words].as_chunks::<4>().0)
            {
                // SAFETY: each chunk is a complete aligned vector in a distinct row.
                let (eq, pv, mv) = unsafe {
                    (
                        _mm256_load_si256(mask.as_ptr().cast()),
                        _mm256_load_si256(p.as_ptr().cast()),
                        _mm256_load_si256(m.as_ptr().cast()),
                    )
                };
                let x = _mm256_and_si256(eq, pv);
                let sum = _mm256_add_epi64(x, pv);
                let generate = _mm256_movemask_pd(_mm256_castsi256_pd(_mm256_or_si256(
                    x,
                    _mm256_andnot_si256(sum, pv),
                ))) as u32;
                let propagate =
                    _mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpeq_epi64(sum, all))) as u32;
                let incoming = (propagate + (generate << 1) + carry) ^ propagate;
                carry = incoming >> 4;
                let increments = _mm256_and_si256(
                    _mm256_srlv_epi64(_mm256_set1_epi64x(incoming as i64), positions),
                    one,
                );
                let xv = _mm256_or_si256(eq, mv);
                let xh =
                    _mm256_or_si256(_mm256_xor_si256(_mm256_add_epi64(sum, increments), pv), eq);
                let next_ph =
                    _mm256_or_si256(mv, _mm256_andnot_si256(_mm256_or_si256(xh, pv), all));
                let next_mh = _mm256_and_si256(pv, xh);
                let in_p = _mm256_blend_epi32::<3>(
                    _mm256_permute4x64_epi64::<0x90>(next_ph),
                    _mm256_permute4x64_epi64::<0xff>(ph),
                );
                let in_m = _mm256_blend_epi32::<3>(
                    _mm256_permute4x64_epi64::<0x90>(next_mh),
                    _mm256_permute4x64_epi64::<0xff>(mh),
                );
                ph = next_ph;
                mh = next_mh;
                let shifted_ph =
                    _mm256_or_si256(_mm256_slli_epi64::<1>(ph), _mm256_srli_epi64::<63>(in_p));
                let shifted_mh =
                    _mm256_or_si256(_mm256_slli_epi64::<1>(mh), _mm256_srli_epi64::<63>(in_m));
                let pv = _mm256_or_si256(
                    shifted_mh,
                    _mm256_andnot_si256(_mm256_or_si256(xv, shifted_ph), all),
                );
                let mv = _mm256_and_si256(shifted_ph, xv);
                // SAFETY: the same complete aligned state chunks were loaded above.
                unsafe {
                    _mm256_store_si256(p.as_mut_ptr().cast(), pv);
                    _mm256_store_si256(m.as_mut_ptr().cast(), mv);
                }
            }
            *h = (_mm256_testz_si256(mh, top) - _mm256_testz_si256(ph, top)) as i8;
        }
        for (i, &offset) in group.iter().enumerate() {
            masks[offset + i / 64] = 0;
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn advance_dense_avx512(
    pattern: &[usize],
    text: &[usize],
    horizontal: &mut [i8],
    masks: &mut [u64],
    positive: &mut [u64],
    negative: &mut [u64],
) {
    use std::arch::x86_64::*;
    let stride = positive.len();
    let all = _mm512_set1_epi64(-1);
    let one = _mm512_set1_epi64(1);
    for group in pattern.chunks(stride * 64) {
        for (i, &offset) in group.iter().enumerate() {
            masks[offset + i / 64] |= 1 << (i % 64);
        }
        let words = group.len().div_ceil(64).next_multiple_of(8);
        let positive = &mut positive[..words];
        let negative = &mut negative[..words];
        positive.fill(u64::MAX);
        negative.fill(0);
        let mut top = [0u64; 8];
        top[((group.len() - 1) / 64) % 8] = 1 << ((group.len() - 1) % 64);
        // SAFETY: top contains one complete, possibly unaligned vector.
        let top = unsafe { _mm512_loadu_si512(top.as_ptr().cast()) };
        for (&offset, h) in text.iter().zip(horizontal.iter_mut()) {
            let mut carry = u32::from(*h < 0);
            let mut ph = _mm512_set1_epi64(i64::from(*h > 0) << 63);
            let mut mh = _mm512_set1_epi64(i64::from(*h < 0) << 63);
            for ((p, m), mask) in positive
                .as_chunks_mut::<8>()
                .0
                .iter_mut()
                .zip(negative.as_chunks_mut::<8>().0.iter_mut())
                .zip(masks[offset..offset + words].as_chunks::<8>().0)
            {
                // SAFETY: each chunk is a complete aligned vector in a distinct row.
                let (eq, pv, mv) = unsafe {
                    (
                        _mm512_load_si512(mask.as_ptr().cast()),
                        _mm512_load_si512(p.as_ptr().cast()),
                        _mm512_load_si512(m.as_ptr().cast()),
                    )
                };
                let x = _mm512_and_si512(eq, pv);
                let sum = _mm512_add_epi64(x, pv);
                let generate = _mm512_cmp_epu64_mask::<_MM_CMPINT_LT>(sum, x) as u32;
                let propagate = _mm512_cmpeq_epi64_mask(sum, all) as u32;
                let incoming = (propagate + (generate << 1) + carry) ^ propagate;
                carry = incoming >> 8;
                let sum = _mm512_mask_add_epi64(sum, incoming as u8, sum, one);
                let xv = _mm512_or_si512(eq, mv);
                let xh = _mm512_ternarylogic_epi64::<0xbe>(sum, pv, eq);
                let next_ph = _mm512_ternarylogic_epi64::<0xf1>(mv, xh, pv);
                let next_mh = _mm512_and_si512(pv, xh);
                let in_p = _mm512_alignr_epi64::<7>(next_ph, ph);
                let in_m = _mm512_alignr_epi64::<7>(next_mh, mh);
                ph = next_ph;
                mh = next_mh;
                let shifted_ph =
                    _mm512_or_si512(_mm512_slli_epi64::<1>(ph), _mm512_srli_epi64::<63>(in_p));
                let shifted_mh =
                    _mm512_or_si512(_mm512_slli_epi64::<1>(mh), _mm512_srli_epi64::<63>(in_m));
                let pv = _mm512_ternarylogic_epi64::<0xf1>(shifted_mh, xv, shifted_ph);
                let mv = _mm512_and_si512(shifted_ph, xv);
                // SAFETY: the same complete aligned state chunks were loaded above.
                unsafe {
                    _mm512_store_si512(p.as_mut_ptr().cast(), pv);
                    _mm512_store_si512(m.as_mut_ptr().cast(), mv);
                }
            }
            *h = i8::from(_mm512_test_epi64_mask(ph, top) != 0)
                - i8::from(_mm512_test_epi64_mask(mh, top) != 0);
        }
        for (i, &offset) in group.iter().enumerate() {
            masks[offset + i / 64] = 0;
        }
    }
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
        #[derive(PartialEq, Eq, Hash)]
        struct Token(u32);
        let x: Vec<_> = (0..513).map(Token).collect();
        let y: Vec<_> = (100..700).map(Token).collect();
        check(&x, &y);
        for m in [1, 63, 64, 65, 255, 256, 257] {
            let x: Vec<_> = (0..m).map(|i| (i % 7) as u32).collect();
            for skipped in [0, 1, 63, 64, 65, 256, 513] {
                let y: Vec<_> = std::iter::repeat_n(u32::MAX, skipped)
                    .chain(x.iter().copied())
                    .collect();
                check(&x, &y);
            }
        }
        // Dense masks span multiple vectors and tiles, with a partial final tile.
        let x: Vec<u32> = (0..4097).map(|i| i % 128).collect();
        let y: Vec<u32> = (0..4104).map(|_| rng.random(0..128)).collect();
        check(&x, &y);
        // Shared large alphabets exercise the small-tile gather path.
        for n in [511, 512, 513, 1023, 1024, 1025] {
            let x: Vec<u32> = (0..n).collect();
            let mut y = x.clone();
            y.rotate_left(n as usize / 3);
            check(&x, &y);
            y.reverse();
            check(&x, &y);
        }
    }

    #[test]
    fn test_levenshtein_wavefront() {
        let sequences: Vec<_> = exhaustive_sequences(0u8..2, 0..=5).collect();
        for x in &sequences {
            for y in &sequences {
                let expected = naive_distances(x, y)[y.len()];
                assert_eq!(levenshtein_wavefront(x, y, usize::MAX), Some(expected));
                for budget in [0, 1, 2, 7, 31, 128] {
                    if let Some(actual) = levenshtein_wavefront(x, y, budget) {
                        assert_eq!(actual, expected);
                    }
                }
            }
        }
        let mut rng = Xorshift::default();
        // Match blocks and the public search threshold both have boundary cases.
        for n in [31, 32, 33, 2047, 2048, 2049] {
            let x: Vec<u8> = (0..n).map(|_| rng.random(0..26)).collect();
            let mut y = x.clone();
            y[0] = 26;
            y[n - 1] = 26;
            y.insert(n / 3, 26);
            y.remove(2 * n / 3);
            let expected = naive_distances(&x, &y)[n];
            for (x, y) in [(&x, &y), (&y, &x)] {
                assert_eq!(levenshtein_wavefront(x, y, 4096), Some(expected));
                assert_eq!(levenshtein_wavefront(x, y, 1), None);
                assert_eq!(levenshtein_distance(x, y), expected);
            }
            let y: Vec<u8> = (0..n).map(|_| rng.random(0..26)).collect();
            let expected = naive_distances(&x, &y)[n];
            assert_eq!(levenshtein_wavefront(&x, &y, 64), None);
            assert_eq!(levenshtein_distance(&x, &y), expected);
        }
        // A long equal run must also spend the search budget.
        let x = vec![0u8; 4096];
        let mut y = x.clone();
        y[0] = 1;
        y[4095] = 1;
        assert_eq!(levenshtein_wavefront(&x, &y, 64), None);
        assert_eq!(levenshtein_wavefront(&x, &y, 4096), Some(2));
        assert_eq!(levenshtein_distance(&x, &y), 2);
        let x: Vec<u8> = (0..2048).map(|_| rng.random(0..26)).collect();
        for clustered in [false, true] {
            let mut y = x.clone();
            for i in 0..40 {
                let index = if clustered && i < 39 {
                    i
                } else {
                    i * 2047 / 39
                };
                y[index] = (y[index] + 1) % 26;
            }
            let expected = naive_distances(&x, &y)[y.len()];
            assert_eq!(levenshtein_wavefront(&x, &y, 4096), Some(expected));
            assert_eq!(levenshtein_distance(&x, &y), expected);
        }
        for n in [31, 32, 33, 4096] {
            let mut x = vec![0u8; n];
            x.extend([1, 2, 3]);
            x.extend(std::iter::repeat_n(0, n));
            let mut y = vec![0u8; n];
            y.extend([4, 5]);
            y.extend(std::iter::repeat_n(0, n));
            assert_eq!(levenshtein_distance(&x, &y), 3);
            assert_eq!(levenshtein_distance(&y, &x), 3);
        }
        #[derive(PartialEq, Eq, Hash)]
        struct Token(u32);
        let x: Vec<_> = (0..4096).map(Token).collect();
        let y: Vec<_> = std::iter::once(u32::MAX)
            .chain(1..4098)
            .map(Token)
            .collect();
        check(&x, &y);
        assert_eq!(levenshtein_wavefront(&x, &y, 4096), Some(3));
        assert_eq!(levenshtein_wavefront(&y, &x, 4096), Some(3));
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
        // Halves cross scalar, AVX2, and AVX-512 boundaries.
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
        #[derive(PartialEq, Eq, Hash)]
        struct Token(u32);
        let x: Vec<_> = (0..1538).map(Token).collect();
        let y: Vec<_> = (100..1700).map(Token).collect();
        check_edits(&x, &y);
    }
}
