use super::{RangeMinimumQuery, SuffixArray};
use std::{cmp::Ordering, ops::Range};

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
        let n = text.len();
        let suffix_array = SuffixArray::new(&text);

        let mut rank = vec![0usize; n + 1];
        for i in 0..=n {
            rank[suffix_array[i]] = i;
        }

        let mut h = 0usize;
        let mut lcp_array = vec![0usize; n];
        for i in 0..n {
            let r = rank[i] - 1;
            let j = suffix_array[r];
            while i + h < n && j + h < n && text[i + h] == text[j + h] {
                h += 1;
            }
            lcp_array[r] = h;
            h = h.saturating_sub(1);
        }

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

    fn compare_suffix_pattern(
        &self,
        suffix_start: usize,
        pattern: &[T],
        start: usize,
    ) -> (Ordering, usize) {
        let n = self.text.len();
        let m = pattern.len();
        let mut i = start;
        while i < m && suffix_start + i < n && self.text[suffix_start + i] == pattern[i] {
            i += 1;
        }
        let ord = if i == m {
            Ordering::Equal
        } else if suffix_start + i == n {
            Ordering::Less
        } else {
            self.text[suffix_start + i].cmp(&pattern[i])
        };
        (ord, i)
    }

    fn bound_prefix(&self, pattern: &[T], upper: bool) -> usize {
        if pattern.is_empty() {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{WithEmptySegment as Wes, Xorshift};

    #[test]
    fn test_lcp_array() {
        let mut rng = Xorshift::default();
        for _ in 0..500 {
            let n = rng.random(0..=80);
            let m = rng.random(1..=20);
            let s: Vec<_> = rng.random_iter(0..m).take(n).collect();
            let search = StringSearch::new(s.to_vec());
            assert_eq!(search.text(), s.as_slice());
            assert_eq!(search.rank().len(), s.len() + 1);
            let mut sa = (0..=n).collect::<Vec<_>>();
            sa.sort_unstable_by_key(|&i| &s[i..]);
            for (i, &pos) in sa.iter().enumerate() {
                assert_eq!(search.suffix_array()[i], pos);
            }
            let lcp = search.lcp_array();
            if n == 0 {
                assert!(lcp.is_empty());
                continue;
            }
            for i in 1..=n {
                let h = s[sa[i - 1]..]
                    .iter()
                    .zip(s[sa[i]..].iter())
                    .take_while(|(a, b)| a == b)
                    .count();
                assert_eq!(lcp[i - 1], h);
            }
        }
    }

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
            let mut sa = (0..=n).collect::<Vec<_>>();
            sa.sort_unstable_by_key(|&i| &s[i..]);
            for _ in 0..200 {
                let pattern = if n == 0 || rng.random(0..=1) == 0 {
                    let m = rng.random(0..=n + 2);
                    rng.random_iter(0..csize).take(m).collect::<Vec<_>>()
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
                assert_eq!(search.range(&pattern), left..right);
            }
        }
    }
}
