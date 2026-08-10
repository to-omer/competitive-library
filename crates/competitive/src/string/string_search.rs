use super::{RangeMinimumQuery, SuffixArray};
use std::{cmp::Ordering, ops::Range};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Delimited<T> {
    Separator(usize),
    Value(T),
}

trait Pattern<T> {
    fn len(&self) -> usize;
    fn eq_text(&self, index: usize, text: &T) -> bool;
    fn cmp_text(&self, index: usize, text: &T) -> Ordering;
}

impl<T> Pattern<T> for [T]
where
    T: Ord,
{
    fn len(&self) -> usize {
        self.len()
    }

    fn eq_text(&self, index: usize, text: &T) -> bool {
        text == &self[index]
    }

    fn cmp_text(&self, index: usize, text: &T) -> Ordering {
        text.cmp(&self[index])
    }
}

struct DelimitedPattern<'a, T> {
    pattern: &'a [T],
}

impl<T> Pattern<Delimited<T>> for DelimitedPattern<'_, T>
where
    T: Ord,
{
    fn len(&self) -> usize {
        self.pattern.len()
    }

    fn eq_text(&self, index: usize, text: &Delimited<T>) -> bool {
        matches!(text, Delimited::Value(value) if value == &self.pattern[index])
    }

    fn cmp_text(&self, index: usize, text: &Delimited<T>) -> Ordering {
        match text {
            Delimited::Separator(_) => Ordering::Less,
            Delimited::Value(value) => value.cmp(&self.pattern[index]),
        }
    }
}

#[derive(Debug)]
pub struct StringSearch<T> {
    text: Vec<T>,
    suffix_array: SuffixArray,
    lcp_array: Vec<usize>,
    rank: Vec<usize>,
    rmq: RangeMinimumQuery<usize>,
}

impl<T> StringSearch<T>
where
    T: Ord,
{
    pub fn new(text: Vec<T>) -> Self {
        let suffix_array = SuffixArray::new(&text);

        let (lcp_array, rank) = suffix_array.lcp_array_with_rank(&text);
        let rmq = RangeMinimumQuery::new(lcp_array.clone());

        Self {
            text,
            suffix_array,
            lcp_array,
            rank,
            rmq,
        }
    }

    pub fn text(&self) -> &[T] {
        &self.text
    }

    pub fn suffix_array(&self) -> &SuffixArray {
        &self.suffix_array
    }

    pub fn lcp_array(&self) -> &[usize] {
        &self.lcp_array
    }

    pub fn rank(&self) -> &[usize] {
        &self.rank
    }

    pub fn longest_common_prefix(&self, a: Range<usize>, b: Range<usize>) -> usize {
        debug_assert!(a.start <= a.end && a.end <= self.text.len());
        debug_assert!(b.start <= b.end && b.end <= self.text.len());
        let len = (a.end - a.start).min(b.end - b.start);
        self.lcp_suffix(a.start, b.start).min(len)
    }

    pub fn compare(&self, a: Range<usize>, b: Range<usize>) -> Ordering {
        debug_assert!(a.start <= a.end && a.end <= self.text.len());
        debug_assert!(b.start <= b.end && b.end <= self.text.len());
        let len_a = a.end - a.start;
        let len_b = b.end - b.start;
        let len = len_a.min(len_b);
        let lcp = self.lcp_suffix(a.start, b.start).min(len);
        if lcp == len {
            return len_a.cmp(&len_b);
        }
        self.text[a.start + lcp].cmp(&self.text[b.start + lcp])
    }

    pub fn range(&self, pattern: &[T]) -> Range<usize> {
        let left = self.bound_prefix(pattern, false);
        let right = self.bound_prefix(pattern, true);
        left..right
    }

    pub fn positions(&self, range: Range<usize>) -> impl DoubleEndedIterator<Item = usize> + '_ {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= self.text.len() + 1);
        range.map(move |i| self.suffix_array[i])
    }

    pub fn kth_substrings(&self) -> KthSubstrings<'_, T> {
        KthSubstrings::new(self)
    }

    fn lcp_suffix(&self, a: usize, b: usize) -> usize {
        self.lcp_sa(self.rank[a], self.rank[b])
    }

    fn lcp_sa(&self, a: usize, b: usize) -> usize {
        if a == b {
            return self.text.len() - self.suffix_array[a];
        }
        let (l, r) = if a < b { (a, b) } else { (b, a) };
        self.rmq.fold(l, r)
    }

    fn compare_suffix_pattern<P>(
        &self,
        suffix_start: usize,
        pattern: &P,
        start: usize,
    ) -> (Ordering, usize)
    where
        P: Pattern<T> + ?Sized,
    {
        let n = self.text.len();
        let m = pattern.len();
        let mut i = start;
        while i < m && suffix_start + i < n && pattern.eq_text(i, &self.text[suffix_start + i]) {
            i += 1;
        }
        let ord = if i == m {
            Ordering::Equal
        } else if suffix_start + i == n {
            Ordering::Less
        } else {
            pattern.cmp_text(i, &self.text[suffix_start + i])
        };
        (ord, i)
    }

    fn bound_prefix<P>(&self, pattern: &P, upper: bool) -> usize
    where
        P: Pattern<T> + ?Sized,
    {
        if pattern.len() == 0 {
            return if upper { self.text.len() + 1 } else { 0 };
        }
        let pred = |ord: Ordering| {
            if upper {
                ord == Ordering::Greater
            } else {
                ord != Ordering::Less
            }
        };
        let (cmp_last, lcp_last) =
            self.compare_suffix_pattern(self.suffix_array[self.text.len()], pattern, 0);
        if !pred(cmp_last) {
            return self.text.len() + 1;
        }
        let mut l = 0usize;
        let mut r = self.text.len();
        let mut lcp_l = 0usize;
        let mut lcp_r = lcp_last;
        while r - l > 1 {
            let m = (l + r) >> 1;
            let start = match lcp_l.cmp(&lcp_r) {
                Ordering::Less => lcp_l.min(self.lcp_sa(l, m)),
                Ordering::Greater => lcp_r.min(self.lcp_sa(m, r)),
                Ordering::Equal => lcp_l,
            };
            let (cmp_m, lcp_m) = self.compare_suffix_pattern(self.suffix_array[m], pattern, start);
            if pred(cmp_m) {
                r = m;
                lcp_r = lcp_m;
            } else {
                l = m;
                lcp_l = lcp_m;
            }
        }
        r
    }

    fn geq_suffix(&self, range: Range<usize>) -> usize {
        let n = self.text.len();
        debug_assert!(range.start <= range.end && range.end <= n);
        let mut l = 0usize;
        let mut r = n;
        while r - l > 1 {
            let m = (l + r) >> 1;
            let ord = self.compare(self.suffix_array[m]..n, range.start..range.end);
            if matches!(ord, Ordering::Less) {
                l = m;
            } else {
                r = m;
            }
        }
        r
    }
}

#[derive(Debug)]
pub struct KthSubstrings<'a, T> {
    search: &'a StringSearch<T>,
    prefix: Vec<u64>,
}

impl<'a, T> KthSubstrings<'a, T>
where
    T: Ord,
{
    fn new(search: &'a StringSearch<T>) -> Self {
        let n = search.text.len();
        let mut prefix = Vec::with_capacity(n + 1);
        prefix.push(0);
        let mut total = 0u64;
        for i in 1..=n {
            total += (n - search.suffix_array[i] - search.lcp_array[i - 1]) as u64;
            prefix.push(total);
        }
        Self { search, prefix }
    }

    pub fn kth_distinct_substring(&self, k: u64) -> Option<Range<usize>> {
        let idx = self.prefix.partition_point(|&x| x <= k);
        if idx == self.prefix.len() {
            return None;
        }
        debug_assert!(idx > 0);
        let start = self.search.suffix_array[idx];
        let len = self.search.lcp_array[idx - 1] + (k - self.prefix[idx - 1]) as usize + 1;
        Some(start..start + len)
    }

    pub fn index_of_distinct_substring(&self, range: Range<usize>) -> u64 {
        debug_assert!(range.start < range.end && range.end <= self.search.text.len());
        let m = range.len();
        let idx = self.search.geq_suffix(range);
        self.prefix[idx - 1] + (m - self.search.lcp_array[idx - 1] - 1) as u64
    }
}

#[derive(Debug)]
pub struct MultipleStringSearch<T> {
    texts: Vec<Vec<T>>,
    offsets: Vec<usize>,
    position_map: Vec<(usize, usize)>,
    search: StringSearch<Delimited<T>>,
}

impl<T> MultipleStringSearch<T>
where
    T: Ord + Clone,
{
    pub fn new(texts: Vec<Vec<T>>) -> Self {
        assert!(!texts.is_empty());
        let total_len: usize = texts.iter().map(|text| text.len() + 1).sum();
        let mut concat = Vec::with_capacity(total_len - 1);
        let mut offsets = Vec::with_capacity(texts.len());
        let mut position_map = Vec::with_capacity(total_len);
        for (i, text) in texts.iter().enumerate() {
            offsets.push(concat.len());
            for (pos, value) in text.iter().cloned().enumerate() {
                concat.push(Delimited::Value(value));
                position_map.push((i, pos));
            }
            if i + 1 < texts.len() {
                concat.push(Delimited::Separator(!i));
            }
            position_map.push((i, text.len()));
        }
        let search = StringSearch::new(concat);
        Self {
            texts,
            offsets,
            position_map,
            search,
        }
    }

    pub fn texts(&self) -> &[Vec<T>] {
        &self.texts
    }

    pub fn longest_common_prefix(
        &self,
        a: (usize, Range<usize>),
        b: (usize, Range<usize>),
    ) -> usize {
        let a = self.to_global_range(a);
        let b = self.to_global_range(b);
        self.search.longest_common_prefix(a, b)
    }

    pub fn compare(&self, a: (usize, Range<usize>), b: (usize, Range<usize>)) -> Ordering {
        let a = self.to_global_range(a);
        let b = self.to_global_range(b);
        self.search.compare(a, b)
    }

    pub fn range(&self, pattern: &[T]) -> Range<usize> {
        let pattern = DelimitedPattern { pattern };
        let left = self.search.bound_prefix(&pattern, false);
        let right = self.search.bound_prefix(&pattern, true);
        left..right
    }

    pub fn positions(
        &self,
        range: Range<usize>,
    ) -> impl DoubleEndedIterator<Item = (usize, usize)> + '_ {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= self.position_map.len());
        range.map(move |i| self.position_map[self.search.suffix_array[i]])
    }

    pub fn kth_substrings(&self) -> MultipleKthSubstrings<'_, T> {
        MultipleKthSubstrings::new(self)
    }

    fn to_global_range(&self, (index, range): (usize, Range<usize>)) -> Range<usize> {
        debug_assert!(index < self.texts.len());
        let len = self.texts[index].len();
        debug_assert!(range.start <= range.end && range.end <= len);
        let base = self.offsets[index];
        base + range.start..base + range.end
    }

    fn suffix_len(&self, a: usize) -> usize {
        let (text_idx, pos) = self.position_map[self.search.suffix_array[a]];
        self.texts[text_idx].len() - pos
    }
}

#[derive(Debug)]
pub struct MultipleKthSubstrings<'a, T> {
    search: &'a MultipleStringSearch<T>,
    prefix: Vec<u64>,
}

impl<'a, T> MultipleKthSubstrings<'a, T>
where
    T: Ord + Clone,
{
    fn new(search: &'a MultipleStringSearch<T>) -> Self {
        let n = search.search.text.len();
        let mut prefix = Vec::with_capacity(n);
        prefix.push(0);
        let mut total = 0u64;
        for i in 1..=n {
            let len = search.suffix_len(i);
            let prev_len = search.suffix_len(i - 1);
            let lcp_prev = search.search.lcp_array[i - 1].min(len).min(prev_len);
            total += (len - lcp_prev) as u64;
            prefix.push(total);
        }
        Self { search, prefix }
    }

    pub fn kth_distinct_substring(&self, k: u64) -> Option<(usize, Range<usize>)> {
        let idx = self.prefix.partition_point(|&x| x <= k);
        if idx == self.prefix.len() {
            return None;
        }
        let (text_idx, pos) = self.search.position_map[self.search.search.suffix_array[idx]];
        let len = self.search.suffix_len(idx) - (self.prefix[idx] - k) as usize + 1;
        Some((text_idx, pos..pos + len))
    }

    pub fn index_of_distinct_substring(&self, (text_idx, range): (usize, Range<usize>)) -> u64 {
        debug_assert!(text_idx < self.search.texts.len());
        debug_assert!(range.start < range.end && range.end <= self.search.texts[text_idx].len());
        let m = range.len();
        let range = self.search.to_global_range((text_idx, range));
        let idx = self.search.search.geq_suffix(range);
        let len = self.search.suffix_len(idx);
        let prev_len = self.search.suffix_len(idx - 1);
        let lcp_prev = self.search.search.lcp_array[idx - 1].min(len).min(prev_len);
        self.prefix[idx - 1] + (m - lcp_prev - 1) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{WithEmptySegment as Wes, Xorshift};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn test_longest_common_prefix_and_compare() {
        let mut rng = Xorshift::default();
        for _ in 0..500 {
            let n = rng.random(0..=80);
            let m = rng.random(1..=20);
            let s: Vec<_> = rng.random_iter(0..m).take(n).collect();
            let search = StringSearch::new(s.clone());
            if n == 0 {
                assert_eq!(search.longest_common_prefix(0..0, 0..0), 0);
                assert_eq!(search.compare(0..0, 0..0), Ordering::Equal);
                continue;
            }
            for _ in 0..200 {
                let (al, ar) = rng.random(Wes(n));
                let (bl, br) = rng.random(Wes(n));
                let lcp = s[al..ar]
                    .iter()
                    .zip(s[bl..br].iter())
                    .take_while(|(x, y)| x == y)
                    .count();
                assert_eq!(search.longest_common_prefix(al..ar, bl..br), lcp);
                let expected = s[al..ar].cmp(&s[bl..br]);
                assert_eq!(search.compare(al..ar, bl..br), expected);
            }
        }
    }

    #[test]
    fn test_range() {
        let mut rng = Xorshift::default();
        for _ in 0..500 {
            let n = rng.random(0..=80);
            let csize = rng.random(1..=20);
            let s: Vec<usize> = rng.random_iter(0..csize).take(n).collect();
            let search = StringSearch::new(s.clone());
            let mut sa: Vec<_> = (0..=n).collect();
            sa.sort_unstable_by_key(|&i| &s[i..]);
            for _ in 0..200 {
                let pattern = if n == 0 || rng.random(0..=1) == 0 {
                    let m = rng.random(0..=n + 2);
                    rng.random_iter(0..csize).take(m).collect()
                } else {
                    let (l, r) = rng.random(Wes(n));
                    s[l..r].to_vec()
                };
                let cmp = |pos| {
                    if s[pos..].starts_with(&pattern) {
                        Ordering::Equal
                    } else {
                        s[pos..].cmp(&pattern)
                    }
                };
                let left = sa
                    .iter()
                    .position(|&pos| cmp(pos) != Ordering::Less)
                    .unwrap_or(sa.len());
                let right = sa
                    .iter()
                    .rposition(|&pos| cmp(pos) != Ordering::Greater)
                    .map_or(left, |i| i + 1);
                let range = search.range(&pattern);
                assert_eq!(range, left..right);
                let positions: Vec<_> = search.positions(range).collect();
                assert_eq!(positions, sa[left..right]);
            }
        }
    }

    #[test]
    fn test_kth_substring() {
        let mut rng = Xorshift::default();
        for _ in 0..500 {
            let n = rng.random(0..=80);
            let csize = rng.random(1..=20);
            let s: Vec<usize> = rng.random_iter(0..csize).take(n).collect();
            let search = StringSearch::new(s.clone());
            let kth = search.kth_substrings();
            let mut set = BTreeSet::new();
            for i in 0..n {
                for j in i + 1..=n {
                    set.insert(s[i..j].to_vec());
                }
            }
            let substrings: Vec<_> = set.into_iter().collect();
            for (k, expected) in substrings.iter().enumerate() {
                let range = kth.kth_distinct_substring(k as u64).unwrap();
                assert_eq!(&s[range.clone()], expected.as_slice());
                assert_eq!(kth.index_of_distinct_substring(range), k as u64);
            }
            assert_eq!(kth.kth_distinct_substring(substrings.len() as u64), None);
            let mut index_map = BTreeMap::new();
            for (idx, substring) in substrings.iter().enumerate() {
                index_map.insert(substring.clone(), idx as _);
            }
            for i in 0..n {
                for j in i + 1..=n {
                    let key = s[i..j].to_vec();
                    let expected = *index_map.get(&key).unwrap();
                    assert_eq!(kth.index_of_distinct_substring(i..j), expected);
                }
            }
        }
    }

    #[test]
    fn test_multiple_longest_common_prefix_and_compare() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let k = rng.random(1..=6);
            let csize = rng.random(1..=20);
            let mut texts = Vec::with_capacity(k);
            for _ in 0..k {
                let n = rng.random(0..=40);
                let s: Vec<_> = rng.random_iter(0..csize).take(n).collect();
                texts.push(s);
            }
            let search = MultipleStringSearch::new(texts.clone());
            for _ in 0..200 {
                let i = rng.random(0..k);
                let j = rng.random(0..k);
                let (al, ar) = rng.random(Wes(texts[i].len()));
                let (bl, br) = rng.random(Wes(texts[j].len()));
                let lcp = texts[i][al..ar]
                    .iter()
                    .zip(texts[j][bl..br].iter())
                    .take_while(|(x, y)| x == y)
                    .count();
                assert_eq!(search.longest_common_prefix((i, al..ar), (j, bl..br)), lcp);
                assert_eq!(
                    search.compare((i, al..ar), (j, bl..br)),
                    texts[i][al..ar].cmp(&texts[j][bl..br])
                );
            }
        }
    }

    #[test]
    fn test_multiple_range() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let k = rng.random(1..=6);
            let csize = rng.random(1..=20);
            let mut texts = Vec::with_capacity(k);
            for _ in 0..k {
                let n = rng.random(0..=40);
                let s: Vec<_> = rng.random_iter(0..csize).take(n).collect();
                texts.push(s);
            }
            let search = MultipleStringSearch::new(texts.clone());
            let mut sa: Vec<_> = (0..k)
                .flat_map(|i| (0..=texts[i].len()).map(move |pos| (i, pos)))
                .collect();
            sa.sort_unstable_by_key(|&(i, pos)| (&texts[i][pos..], !i));
            for _ in 0..200 {
                let pattern = if rng.random(0..=1) == 0 {
                    let m = rng.random(0..=50);
                    rng.random_iter(0..csize).take(m).collect()
                } else {
                    let idx = rng.random(0..k);
                    let (l, r) = rng.random(Wes(texts[idx].len()));
                    texts[idx][l..r].to_vec()
                };
                let cmp = |i: usize, pos: usize| {
                    if texts[i][pos..].starts_with(&pattern) {
                        Ordering::Equal
                    } else {
                        texts[i][pos..].cmp(&pattern)
                    }
                };
                let left = sa
                    .iter()
                    .position(|&(i, pos)| cmp(i, pos) != Ordering::Less)
                    .unwrap_or(sa.len());
                let right = sa
                    .iter()
                    .rposition(|&(i, pos)| cmp(i, pos) != Ordering::Greater)
                    .map_or(left, |idx| idx + 1);
                let range = search.range(&pattern);
                assert_eq!(range, left..right);
                let positions: Vec<_> = search.positions(range).collect();
                assert_eq!(positions, sa[left..right]);
            }
        }
    }

    #[test]
    fn test_multiple_kth_substring() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let k = rng.random(1..=6);
            let csize = rng.random(1..=20);
            let mut texts = Vec::with_capacity(k);
            for _ in 0..k {
                let n = rng.random(0..=40);
                let s: Vec<_> = rng.random_iter(0..csize).take(n).collect();
                texts.push(s);
            }
            let search = MultipleStringSearch::new(texts.clone());
            let kth = search.kth_substrings();
            let mut set = BTreeSet::new();
            for text in &texts {
                for i in 0..text.len() {
                    for j in i + 1..=text.len() {
                        set.insert(text[i..j].to_vec());
                    }
                }
            }
            let substrings: Vec<_> = set.into_iter().collect();
            for (idx, expected) in substrings.iter().enumerate() {
                let (text_idx, range) = kth.kth_distinct_substring(idx as u64).unwrap();
                assert_eq!(&texts[text_idx][range.clone()], expected.as_slice());
                assert_eq!(kth.index_of_distinct_substring((text_idx, range)), idx as _);
            }
            assert_eq!(kth.kth_distinct_substring(substrings.len() as u64), None);
            let mut index_map = BTreeMap::new();
            for (idx, substring) in substrings.iter().enumerate() {
                index_map.insert(substring.clone(), idx as u64);
            }
            for (text_idx, text) in texts.iter().enumerate() {
                for i in 0..text.len() {
                    for j in i + 1..=text.len() {
                        let key = text[i..j].to_vec();
                        let expected = *index_map.get(&key).unwrap();
                        assert_eq!(kth.index_of_distinct_substring((text_idx, i..j)), expected);
                    }
                }
            }
        }
    }
}
