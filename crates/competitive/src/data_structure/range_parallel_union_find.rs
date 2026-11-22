use super::{UnionFind, gcd};

/// Unites corresponding vertices of two equal-length ranges in an indexed set.
/// Uses O(n log n) space. Excluding callbacks, q range unions take O((n log n + q) α(n)) time.
#[derive(Clone, Debug)]
pub struct RangeParallelUnionFind {
    n: usize,
    vertices: UnionFind,
    blocks: Vec<UnionFind>,
    periods: Vec<PeriodicRange>,
    recent: usize,
}

#[derive(Clone, Copy, Debug)]
struct PeriodicRange {
    start: usize,
    end: usize,
    period: usize,
}

impl RangeParallelUnionFind {
    /// Creates `n` singleton components, with `n <= i32::MAX`.
    pub fn new(n: usize) -> Self {
        assert!(n <= i32::MAX as usize);
        let blocks = std::iter::successors(Some(4usize), |w| w.checked_mul(4))
            .take_while(|&w| w < n)
            .map(|w| UnionFind::new(n - w + 1))
            .collect();
        Self {
            n,
            vertices: UnionFind::new(n),
            blocks,
            periods: (0..n)
                .step_by(4)
                .map(|mid| PeriodicRange {
                    start: mid,
                    end: mid + 1,
                    period: 1,
                })
                .collect(),
            recent: 0,
        }
    }

    pub fn find_root(&mut self, x: usize) -> usize {
        self.vertices.find_root(x)
    }

    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.vertices.same(x, y)
    }

    /// Unites `a+i` with `b+i` for `0 <= i < len`, in increasing offset order.
    /// Calls `merge(surviving_root, absorbed_root)` once per successful vertex union.
    pub fn unite(&mut self, a: usize, b: usize, len: usize, mut merge: impl FnMut(usize, usize)) {
        let (first, second) = (a.min(b), a.max(b));
        assert!(len <= self.n && second <= self.n - len);
        if len == 0 || a == b {
            return;
        }
        if len < 4 {
            for i in 0..len {
                self.unite_block(a + i, b + i, 0, &mut merge);
            }
            return;
        }
        let distance = second - first;
        let end = second + len;
        let recent = self.periods[self.recent];
        if first >= recent.start && end <= recent.end && distance.is_multiple_of(recent.period) {
            return;
        }

        // The enclosing dyadic interval has width at least eight, so its midpoint is divisible by four.
        let shift = (first ^ (end - 1)).ilog2();
        let half = 1usize << shift;
        let mid = (end - 1) >> shift << shift;
        let left = mid - half;
        let right = (mid + half).min(self.n);
        let slot = mid / 4;
        let mut range = self.periods[slot];
        let remainder = distance % range.period;
        let aligned = remainder == 0;

        // Inside the certified interval each vertex is connected to its period-shifted copy.
        let prefix = range.start.saturating_sub(first).min(len);
        let suffix = range.end.saturating_sub(second).min(len);
        if aligned && prefix == 0 && suffix == len {
            self.recent = slot;
            return;
        }
        if prefix < suffix {
            let residues = if aligned {
                0
            } else {
                range.period.min(suffix - prefix)
            };
            self.unite_blocks(a, b, prefix + residues, &mut merge);
            self.unite_blocks(a + suffix, b + suffix, len - suffix, &mut merge);
        } else {
            self.unite_blocks(a, b, len, &mut merge);
        }
        // A full cycle of residue pairs in the overlap identifies residues modulo the gcd.
        let overlap = range.end.min(end) - range.start.max(first);
        if overlap >= distance && (aligned || overlap - distance >= range.period) {
            if !aligned {
                range.period = gcd(range.period as u64, remainder as u64) as usize;
            }
            range.start = range.start.min(first);
            range.end = range.end.max(end);
        } else if first <= range.start && range.end <= end && distance <= len {
            range = PeriodicRange {
                start: first,
                end,
                period: distance,
            };
        }
        while range.start > left
            && self
                .vertices
                .same(range.start - 1, range.start - 1 + range.period)
        {
            range.start -= 1;
        }
        while range.end < right && self.vertices.same(range.end - range.period, range.end) {
            range.end += 1;
        }
        self.periods[slot] = range;
        self.recent = slot;
    }

    fn unite_blocks(
        &mut self,
        a: usize,
        b: usize,
        len: usize,
        merge: &mut impl FnMut(usize, usize),
    ) {
        if len == 0 {
            return;
        }
        let shift = len.ilog2() & !1;
        let width = 1 << shift;
        let mut offset = 0;
        while offset < len - width {
            self.unite_block(a + offset, b + offset, shift, merge);
            offset += width;
        }
        self.unite_block(a + len - width, b + len - width, shift, merge);
    }

    fn unite_block(
        &mut self,
        a: usize,
        b: usize,
        shift: u32,
        merge: &mut impl FnMut(usize, usize),
    ) {
        if shift == 0 {
            let x = self.vertices.find_root(a);
            let y = self.vertices.find_root(b);
            if x != y {
                self.vertices.unite(x, y);
                if self.vertices.find_root(x) == x {
                    merge(x, y);
                } else {
                    merge(y, x);
                }
            }
        } else {
            let level = (shift / 2 - 1) as usize;
            let x = self.blocks[level].find_root(a);
            let y = self.blocks[level].find_root(b);
            if x != y {
                for i in 0..4 {
                    let offset = i << (shift - 2);
                    self.unite_block(a + offset, b + offset, shift - 2, merge);
                }
                self.blocks[level].unite(x, y);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RangeParallelUnionFind, UnionFind};
    use crate::{rand, tools::Xorshift};

    fn check_queries(n: usize, queries: impl IntoIterator<Item = (usize, usize, usize)>) {
        let mut uf = RangeParallelUnionFind::new(n);
        let mut naive = UnionFind::new(n);
        let mut members: Vec<_> = (0..n).map(|v| 1u128 << v).collect();
        let mut expected = vec![0u128; n];
        let mut events = Vec::new();
        for (a, b, len) in queries {
            events.clear();
            for i in 0..len {
                let x = naive.find_root(a + i);
                let y = naive.find_root(b + i);
                if naive.unite(x, y) {
                    if naive.find_root(x) == x {
                        events.push((x, y));
                    } else {
                        events.push((y, x));
                    }
                }
            }
            let mut events = events.iter();
            uf.unite(a, b, len, |to, from| {
                assert_eq!(events.next(), Some(&(to, from)));
                let absorbed = std::mem::take(&mut members[from]);
                assert_ne!(absorbed, 0);
                members[to] |= absorbed;
            });
            assert_eq!(events.next(), None);
            expected.fill(0);
            for v in 0..n {
                expected[naive.find_root(v)] |= 1u128 << v;
            }
            assert_eq!(members, expected);
            for v in 0..n {
                let root = uf.find_root(v);
                assert_eq!(root, naive.find_root(v));
                let other = n - 1 - v;
                assert_eq!(uf.same(v, other), naive.same(v, other));
            }
        }
    }

    #[test]
    fn test_range_parallel_union_find() {
        let mut rng = Xorshift::default();
        check_queries(0, [(0, 0, 0)]);
        for n in 1..=128 {
            for period in [1, 2, 4, 8, 16, 32, 64] {
                if period < n {
                    let len = n - period;
                    let mut queries =
                        vec![(n, 0, 0), (0, 0, n), (0, period, len), (period, 0, len)];
                    let mut step = period;
                    while step > 1 {
                        step /= 2;
                        queries.push((step, 0, n - step));
                    }
                    check_queries(n, queries);
                }
            }
            for _ in 0..45 {
                let queries = (0..48).map(|i| {
                    rand!(rng, exp: 0..=n.ilog2(), delta: 0..3usize);
                    let power = 1usize << exp;
                    rand!(rng, len: 0..=n);
                    let len = match i % 3 {
                        0 => (power + delta - 1).min(n),
                        1 => len.min(4),
                        _ => len,
                    };
                    rand!(rng, a: 0..=n - len, b: 0..=n - len);
                    (a, b, len)
                });
                check_queries(n, queries);
            }
        }
    }
}
