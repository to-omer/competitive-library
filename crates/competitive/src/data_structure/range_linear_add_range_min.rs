use super::RangeBoundsExt;
use std::ops::{Range, RangeBounds};

const BLOCK_SHIFT: u32 = 5;
const FANOUT: usize = 8;
const ROOT_CAPACITY: usize = 128;
const SCAN_LIMIT: u8 = 64;

type Line = (i128, i128);

/// Range minimum queries with additions of an affine function of the index.
///
/// For `i64` values, construction and q operations take O(n + q log² n) time and O(n) space.
#[derive(Debug, Clone)]
pub struct RangeLinearAddRangeMin {
    values: Vec<i64>,
    levels: Vec<Level>,
}

#[derive(Debug, Clone)]
struct Node {
    // Let L interpolate `left` and `right` over the full block width.
    // All values satisfy a[i] >= L(i), and at least one satisfies a[i] <= L(i) + spread.
    left: i128,
    right: i128,
    spread: i128,
    lazy: Line,
}

impl Node {
    fn new(minimum: i128) -> Self {
        Self {
            left: minimum,
            right: minimum,
            spread: 0,
            lazy: (0, 0),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Envelope {
    visits: u8,
    // In index order, each child beats its predecessor below its cut.
    switches: Vec<(i128, usize)>,
}

#[derive(Debug, Clone)]
struct Level {
    shift: u32,
    nodes: Vec<Node>,
    envelopes: Vec<Envelope>,
}

impl RangeLinearAddRangeMin {
    pub fn from_keys(keys: impl ExactSizeIterator<Item = i64>) -> Self {
        let values: Vec<_> = keys.collect();
        let blocks = values
            .chunks(1 << BLOCK_SHIFT)
            .map(|v| Node::new(i128::from(*v.iter().min().unwrap())))
            .collect();
        let mut level = Level {
            shift: BLOCK_SHIFT,
            nodes: blocks,
            envelopes: Vec::new(),
        };
        let mut levels = Vec::new();
        while level.nodes.len() > ROOT_CAPACITY {
            let nodes: Vec<_> = level
                .nodes
                .chunks(FANOUT)
                .map(|v| Node::new(v.iter().map(|n| n.left).min().unwrap()))
                .collect();
            let next = Level {
                shift: level.shift + FANOUT.ilog2(),
                envelopes: vec![Envelope::default(); nodes.len()],
                nodes,
            };
            levels.push(level);
            level = next;
        }
        levels.push(level);
        Self { values, levels }
    }

    /// Adds `slope * i + intercept` to every element at index `i` in the range.
    /// Each resulting element value must fit in `i64`.
    pub fn update<R>(&mut self, range: R, slope: i64, intercept: i64)
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.values.len())
            .expect("invalid range");
        if !range.is_empty() {
            self.update_at(
                self.levels.len() - 1,
                range,
                (0, 0),
                (slope.into(), intercept.into()),
            );
        }
    }

    /// Returns the minimum in the range, or `i64::MAX` for an empty range.
    pub fn fold<R>(&mut self, range: R) -> i64
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.values.len())
            .expect("invalid range");
        if range.is_empty() {
            return i64::MAX;
        }
        self.fold_at(self.levels.len() - 1, range, (0, 0), i64::MAX as i128) as i64
    }

    fn split(&self, range: Range<usize>, shift: u32) -> ([Range<usize>; 2], Range<usize>) {
        let mask = (1 << shift) - 1;
        let n = self.values.len();
        let l = (range.start + mask) >> shift;
        let r = (range.end + if range.end == n { mask } else { 0 }) >> shift;
        if l > r {
            return ([range, 0..0], l..l);
        }
        (
            [
                range.start..(l << shift).min(n),
                (r << shift).min(n)..range.end,
            ],
            l..r,
        )
    }

    fn update_at(
        &mut self,
        height: usize,
        range: Range<usize>,
        ancestors: Line,
        (slope, intercept): Line,
    ) {
        let shift = self.levels[height].shift;
        let width = 1 << shift;
        let (edges, full) = self.split(range, shift);
        for edge in edges {
            if edge.is_empty() {
                continue;
            }
            let k = edge.start >> shift;
            let start = k << shift;
            let tag = self.levels[height].nodes[k].lazy;
            let (b, c) = (ancestors.0 + tag.0, ancestors.1 + tag.1);
            let (lower, upper, tag) = if height == 0 {
                let end = (start + width).min(self.values.len());
                let mut minimum = i64::MAX;
                for (part, b, c) in [
                    (start..edge.start, b, c),
                    (edge.clone(), b + slope, c + intercept),
                    (edge.end..end, b, c),
                ] {
                    for (i, x) in part.clone().zip(&mut self.values[part]) {
                        *x = (*x as i128 + b * i as i128 + c) as i64;
                        minimum = minimum.min(*x);
                    }
                }
                // The raw values now include the ancestors' pending additions.
                (
                    minimum as i128,
                    minimum as i128,
                    (-ancestors.0, -ancestors.1),
                )
            } else {
                self.update_at(height - 1, edge, (b, c), (slope, intercept));
                let first = k * FANOUT;
                let nodes = &self.levels[height - 1].nodes;
                let (mut lower, mut upper) = (i128::MAX, i128::MAX);
                for child in &nodes[first..(first + FANOUT).min(nodes.len())] {
                    lower = lower.min(child.left.min(child.right));
                    upper = upper.min(child.left.max(child.right) + child.spread);
                }
                let envelope = &mut self.levels[height].envelopes[k];
                envelope.visits = 0;
                envelope.switches.clear();
                (lower, upper, tag)
            };
            let node = &mut self.levels[height].nodes[k];
            node.lazy = tag;
            node.left = lower + tag.0 * start as i128 + tag.1;
            node.right = node.left + (tag.0 << shift);
            node.spread = upper - lower;
        }
        let step = slope << shift;
        let mut delta = slope * (full.start << shift) as i128 + intercept;
        for node in &mut self.levels[height].nodes[full] {
            node.left += delta;
            delta += step;
            node.right += delta;
            node.lazy.0 += slope;
            node.lazy.1 += intercept;
        }
    }

    fn fold_at(
        &mut self,
        height: usize,
        range: Range<usize>,
        ancestors: Line,
        mut best: i128,
    ) -> i128 {
        let shift = self.levels[height].shift;
        let width = 1 << shift;
        let (edges, full) = self.split(range, shift);
        let step = ancestors.0 << shift;
        let initial = ancestors.0 * (full.start << shift) as i128 + ancestors.1;
        if height != 0 {
            let mut delta = initial;
            for node in &self.levels[height].nodes[full.clone()] {
                let left = node.left + delta;
                delta += step;
                best = best.min(left.max(node.right + delta) + node.spread);
            }
        }
        for edge in edges {
            if edge.is_empty() {
                continue;
            }
            let k = edge.start >> shift;
            let tag = self.levels[height].nodes[k].lazy;
            let line = (ancestors.0 + tag.0, ancestors.1 + tag.1);
            best = if height == 0 {
                best.min(self.minimum(edge, line) as i128)
            } else {
                self.fold_at(height - 1, edge, line, best)
            };
        }
        let mut delta = initial;
        for k in full {
            let node = &self.levels[height].nodes[k];
            let left = node.left + delta;
            delta += step;
            let right = node.right + delta;
            if left.min(right) >= best {
                continue;
            }
            if left == right && node.spread == 0 {
                best = left;
                continue;
            }
            let start = k << shift;
            let end = (start + width).min(self.values.len());
            let line = (ancestors.0 + node.lazy.0, ancestors.1 + node.lazy.1);
            let upper = left.max(right) + node.spread;
            let minimum = if height == 0 {
                i128::from(self.minimum(start..end, line))
            } else if self.levels[height].envelopes[k].visits < SCAN_LIMIT {
                self.levels[height].envelopes[k].visits += 1;
                // Use this node's own upper bound so the cached minimum is exact.
                self.fold_at(height - 1, start..end, line, upper)
            } else {
                self.ensure_envelope(height, k, line.0);
                let (block, line) = self.winning_block(height, k, ancestors);
                i128::from(self.minimum(block, line))
            };
            best = best.min(minimum);
            let node = &mut self.levels[height].nodes[k];
            node.right = minimum - delta;
            node.left = node.right + step;
            node.spread = 0;
        }
        best
    }

    fn minimum(&self, range: Range<usize>, (b, c): Line) -> i64 {
        range
            .clone()
            .zip(&self.values[range])
            .map(|(i, &v)| (v as i128 + b * i as i128 + c) as i64)
            .min()
            .unwrap()
    }

    fn winning_block(
        &self,
        mut height: usize,
        mut k: usize,
        (mut slope, mut intercept): Line,
    ) -> (Range<usize>, Line) {
        while height > 0 {
            let level = &self.levels[height];
            let (a, b) = level.nodes[k].lazy;
            slope += a;
            intercept += b;
            let switches = &level.envelopes[k].switches;
            let i = switches[1..].partition_point(|&(cut, _)| slope < cut);
            k = switches[i].1;
            height -= 1;
        }
        let start = k << BLOCK_SHIFT;
        let end = (start + (1 << BLOCK_SHIFT)).min(self.values.len());
        let (a, b) = self.levels[0].nodes[k].lazy;
        (start..end, (slope + a, intercept + b))
    }

    fn support(&self, height: usize, k: usize, slope: i128) -> (usize, i128) {
        let (range, (a, b)) = self.winning_block(height, k, (slope, 0));
        // Trial slopes in a crossing search can give values outside i64.
        range
            .clone()
            .zip(&self.values[range])
            .map(|(x, &y)| (x, i128::from(y) + a * x as i128 + b))
            .min_by_key(|&(_, y)| y)
            .unwrap()
    }

    // First integer slope with left_min <= right_min. Larger right indices
    // make right_min - left_min increase by at least one per slope step.
    fn crossing(&self, height: usize, left: usize, right: usize, mut slope: i128) -> i128 {
        let (mut lo, mut hi) = (i128::MIN, i128::MAX);
        let mut secant = true;
        loop {
            let (x, y) = self.support(height, left, slope);
            let (z, w) = self.support(height, right, slope);
            let gap = w - y;
            if gap == 0 {
                return slope;
            }
            if gap > 0 {
                hi = slope;
                lo = lo.max(slope - gap - 1);
            } else {
                lo = slope;
                hi = hi.min(slope - gap);
            }
            if hi - lo == 1 {
                return hi;
            }
            slope = if secant {
                let dx = (z - x) as i128;
                let q = gap.div_euclid(dx);
                (slope - q).clamp(lo + 1, hi - 1)
            } else {
                lo + (hi - lo) / 2
            };
            secant = !secant;
        }
    }

    fn ensure_envelope(&mut self, height: usize, k: usize, slope: i128) {
        if !self.levels[height].envelopes[k].switches.is_empty() {
            return;
        }
        let mut table = std::mem::take(&mut self.levels[height].envelopes[k].switches);
        let first = k * FANOUT;
        for child in first..(first + FANOUT).min(self.levels[height - 1].nodes.len()) {
            if height > 1 {
                self.ensure_envelope(
                    height - 1,
                    child,
                    slope + self.levels[height - 1].nodes[child].lazy.0,
                );
            }
            let cut = loop {
                let Some(&(prev, old)) = table.last() else {
                    break i128::MAX;
                };
                let cut = self.crossing(height - 1, old, child, slope);
                if cut < prev {
                    break cut;
                }
                // The previous child has no winning integer slope.
                table.pop();
            };
            table.push((cut, child));
        }
        let envelope = &mut self.levels[height].envelopes[k];
        envelope.visits = SCAN_LIMIT;
        envelope.switches = table;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{WithEmptySegment, Xorshift, testutil::sequence_cases};

    #[test]
    fn test_range_linear_add_range_min() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        let inputs = sequence_cases(&mut input_rng, -1000..=1000, 0..=1025)
            .chain([(-256i64..=256).map(|x| x * x).collect()]);
        for mut values in inputs {
            let n = values.len();
            let mut data = RangeLinearAddRangeMin::from_keys(values.iter().copied());
            assert_eq!(
                data.fold(..),
                values.iter().copied().min().unwrap_or(i64::MAX)
            );
            for step in 0..=2 * n {
                let (l, r) = rng.random(WithEmptySegment(n));
                let range = if step % 4 == 0 { 0..n } else { l..r };
                let b = rng.random(-99i64..=99);
                let c = rng.random(-1000i64..=1000);
                data.update(range.clone(), b, c);
                for (i, x) in range.clone().zip(&mut values[range.clone()]) {
                    *x += b * i as i64 + c;
                }
                let (l, r) = rng.random(WithEmptySegment(n));
                for query in [range, l..r, 0..n] {
                    let expected = values[query.clone()]
                        .iter()
                        .copied()
                        .min()
                        .unwrap_or(i64::MAX);
                    assert_eq!(data.fold(query), expected);
                }
            }
        }
    }

    #[test]
    fn test_wide_slope_crossings() {
        for (initial, sign) in [([i64::MAX, i64::MIN], 1), ([i64::MIN, i64::MAX], -1)] {
            let mut data = RangeLinearAddRangeMin::from_keys(initial.into_iter());
            let mut expected = initial.map(i128::from);
            for (range, b, c) in [
                (0..2, sign * i64::MAX, 0),
                (0..2, sign * i64::MAX, 0),
                (0..2, sign, -sign),
                (0..2, sign, -sign),
                (0..1, 0, sign),
            ] {
                data.update(range.clone(), b, c);
                for i in range {
                    expected[i] += i128::from(b) * i as i128 + i128::from(c);
                }
                for query in [0..1, 1..2, 0..2] {
                    let min = *expected[query.clone()].iter().min().unwrap();
                    assert_eq!(data.fold(query), i64::try_from(min).unwrap());
                }
            }
        }
    }

    #[test]
    fn test_group_bounds_at_i64_limits() {
        use std::ops::Bound::{Excluded, Included};

        let len = ROOT_CAPACITY * (1 << BLOCK_SHIFT) + 1;
        for (limit, slope) in [(i64::MAX, 99), (i64::MIN, -99)] {
            let mut values: Vec<_> = (0..len).map(|i| limit - slope * i as i64).collect();
            let mut data = RangeLinearAddRangeMin::from_keys(values.iter().copied());
            data.update(.., slope, 0);
            values.fill(limit);
            assert_eq!(data.fold(..), limit);
            data.update((Excluded(4), Included(23)), -slope, 0);
            for (i, value) in (5..=23).zip(&mut values[5..=23]) {
                *value -= slope * i;
            }
            for l in 0..32 {
                for r in l..32 {
                    assert_eq!(data.fold(l..=r), *values[l..=r].iter().min().unwrap());
                }
            }
            assert_eq!(data.fold(32..), limit);
            assert_eq!(data.fold(len..), i64::MAX);
        }
    }

    #[test]
    fn test_envelope_crossings_and_invalidation() {
        let mut rng = Xorshift::default();
        let block = 1 << BLOCK_SHIFT;
        let limit = usize::from(SCAN_LIMIT);
        for span in [block * FANOUT, block * FANOUT * FANOUT] {
            let len = span / FANOUT * ROOT_CAPACITY + rng.random(span / 2..span);
            let parabola = (0..len)
                .map(|i| {
                    let x = (i % span) as i64 - (span / 2) as i64;
                    x * x / (span / 64) as i64 + rng.random(-16i64..=16)
                })
                .collect();
            let extremes = (0..len)
                .map(|i| {
                    let sign = if (i / block).is_multiple_of(2) { -1 } else { 1 };
                    sign * (i64::MAX - 10_000_000) + (i % block) as i64
                })
                .collect();
            let full_roots = len / span;
            for (mut values, root) in [
                (vec![0; len], full_roots),
                (parabola, rng.random(0..full_roots)),
                (extremes, rng.random(0..full_roots)),
            ] {
                let mut data = RangeLinearAddRangeMin::from_keys(values.iter().copied());
                let height = data.levels.len() - 1;
                let target = root * span..((root + 1) * span).min(len);
                let mut slope = 0;
                for width in [0, 1, rng.random(2..target.len())] {
                    if width != 0 {
                        let l = rng.random(target.start..=target.end - width);
                        let range = l..l + width;
                        let b = rng.random(-3..=3);
                        let c = rng.random(-1_000_000..=-100_000);
                        data.update(range.clone(), b, c);
                        for i in range {
                            values[i] += b * i as i64 + c;
                        }
                        assert!(data.levels[height].envelopes[root].switches.is_empty());
                    }
                    // Changing the slope and querying this node in isolation forces a visit.
                    for turn in 0..rng.random(limit + 2..=2 * limit) {
                        let magnitude = rng.random(1i64..=49);
                        let next = if slope > 0 { -magnitude } else { magnitude };
                        let b = next - slope;
                        let c = rng.random(-100..=100);
                        data.update(.., b, c);
                        for (i, x) in values.iter_mut().enumerate() {
                            *x += b * i as i64 + c;
                        }
                        slope = next;
                        let (l, r) = rng.random(WithEmptySegment(len));
                        let query = if turn % 4 == 0 { 0..len } else { l..r };
                        for range in [target.clone(), query] {
                            let expected = values[range.clone()].iter().copied().min();
                            assert_eq!(data.fold(range), expected.unwrap_or(i64::MAX));
                        }
                    }
                    assert!(data.levels[height].envelopes[root].switches.len() > 1);
                }
            }
        }
    }
}
