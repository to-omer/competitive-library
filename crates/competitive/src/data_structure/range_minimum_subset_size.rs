use super::{AdditiveOperation, BinaryIndexedTree};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct RangeMinimumSubsetSize {
    values: Vec<i64>,
    events: Vec<RangeMinimumSubsetSizeEvent<i64>>,
    ranges: Vec<Range<usize>>,
    targets: Vec<i128>,
}

impl RangeMinimumSubsetSize {
    pub fn new(values: impl IntoIterator<Item = i64>) -> Self {
        let values: Vec<_> = values.into_iter().map(|x| x.max(0)).collect();
        let events = values
            .iter()
            .enumerate()
            .map(|(i, &x)| RangeMinimumSubsetSizeEvent::Update(i, x, 1))
            .collect();
        Self {
            values,
            events,
            ranges: Vec::new(),
            targets: Vec::new(),
        }
    }

    pub fn set(&mut self, index: usize, value: i64) {
        let value = value.max(0);
        if self.values[index] != value {
            self.events.push(RangeMinimumSubsetSizeEvent::Update(
                index,
                self.values[index],
                -1,
            ));
            self.events
                .push(RangeMinimumSubsetSizeEvent::Update(index, value, 1));
            self.values[index] = value;
        }
    }

    /// Records a minimum subset size query for `sum >= target`; returns its result index.
    pub fn query(&mut self, range: Range<usize>, target: i128) -> usize {
        let id = self.ranges.len();
        self.ranges.push(range);
        self.targets.push(target);
        if target > 0 {
            self.events.push(RangeMinimumSubsetSizeEvent::Query(id));
        }
        id
    }

    /// Results in query order; `None` for unreachable targets.
    pub fn execute(self) -> Vec<Option<usize>> {
        let mut xs = vec![0];
        xs.extend(self.events.iter().filter_map(|e| match e {
            RangeMinimumSubsetSizeEvent::Update(_, x, _) => Some(*x),
            RangeMinimumSubsetSizeEvent::Query(_) => None,
        }));
        xs.sort_unstable();
        xs.dedup();
        let events: Vec<_> = self
            .events
            .into_iter()
            .map(|e| match e {
                RangeMinimumSubsetSizeEvent::Update(p, x, delta) => {
                    RangeMinimumSubsetSizeEvent::Update(p, xs.binary_search(&x).unwrap(), delta)
                }
                RangeMinimumSubsetSizeEvent::Query(id) => RangeMinimumSubsetSizeEvent::Query(id),
            })
            .collect();
        let mut eids: Vec<_> = (0..events.len()).collect();
        let mut buf = vec![0; eids.len()];
        let mut context = RangeMinimumSubsetSizeSolver {
            count: vec![0; self.ranges.len()],
            remaining: self.targets,
            ranges: self.ranges,
            bit: BinaryIndexedTree::new(self.values.len()),
            right: vec![false; events.len()],
            events,
            xs,
        };
        context.dfs(0, context.xs.len(), &mut eids, &mut buf);
        context
            .count
            .into_iter()
            .zip(context.remaining)
            .map(
                |(count, remaining)| {
                    if remaining > 0 { None } else { Some(count) }
                },
            )
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
enum RangeMinimumSubsetSizeEvent<T> {
    Update(usize, T, i64),
    Query(usize),
}

struct RangeMinimumSubsetSizeSolver {
    xs: Vec<i64>,
    events: Vec<RangeMinimumSubsetSizeEvent<usize>>,
    ranges: Vec<Range<usize>>,
    remaining: Vec<i128>,
    count: Vec<usize>,
    bit: BinaryIndexedTree<(AdditiveOperation<i64>, AdditiveOperation<i128>)>,
    right: Vec<bool>,
}

impl RangeMinimumSubsetSizeSolver {
    fn dfs(&mut self, lo: usize, hi: usize, eids: &mut [usize], buf: &mut [usize]) {
        if eids.is_empty() {
            return;
        }
        if lo + 1 == hi {
            if self.xs[lo] > 0 {
                let x = i128::from(self.xs[lo]);
                for &eid in eids.iter() {
                    if let RangeMinimumSubsetSizeEvent::Query(id) = self.events[eid] {
                        let remaining = self.remaining[id];
                        self.count[id] += ((remaining - 1) / x + 1) as usize;
                        self.remaining[id] = 0;
                    }
                }
            }
            return;
        }
        let mid = (lo + hi) / 2;
        let mut nl = 0;
        for &eid in eids.iter() {
            match self.events[eid] {
                RangeMinimumSubsetSizeEvent::Update(p, v, delta) => {
                    self.right[eid] = v >= mid;
                    if self.right[eid] {
                        self.bit
                            .update(p, (delta, i128::from(delta) * i128::from(self.xs[v])));
                    } else {
                        nl += 1;
                    }
                }
                RangeMinimumSubsetSizeEvent::Query(id) => {
                    let range = &self.ranges[id];
                    let (count, sum) = self.bit.fold(range.start, range.end);
                    self.right[eid] = sum >= self.remaining[id];
                    if !self.right[eid] {
                        self.remaining[id] -= sum;
                        self.count[id] += count as usize;
                        nl += 1;
                    }
                }
            }
        }
        let (mut il, mut ir) = (0, nl);
        for &eid in eids.iter() {
            if let RangeMinimumSubsetSizeEvent::Update(p, v, delta) = self.events[eid]
                && self.right[eid]
            {
                self.bit
                    .update(p, (-delta, -i128::from(delta) * i128::from(self.xs[v])));
            }
            if self.right[eid] {
                buf[ir] = eid;
                ir += 1;
            } else {
                buf[il] = eid;
                il += 1;
            }
        }
        let (el, er) = buf.split_at_mut(nl);
        let (bl, br) = eids.split_at_mut(nl);
        self.dfs(lo, mid, el, bl);
        self.dfs(mid, hi, er, br);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        WithEmptySegment, Xorshift,
        testutil::{exhaustive_sequences, integer_boundary_values, sample_usize},
    };

    fn expected(values: &[i64], range: Range<usize>, target: i128) -> Option<usize> {
        let values = &values[range];
        if values.len() <= 4 {
            return (0..1usize << values.len())
                .filter(|&mask| {
                    values
                        .iter()
                        .enumerate()
                        .filter(|&(i, _)| mask >> i & 1 != 0)
                        .map(|(_, &x)| i128::from(x))
                        .sum::<i128>()
                        >= target
                })
                .map(|mask| mask.count_ones() as usize)
                .min();
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable_by_key(|x| std::cmp::Reverse(*x));
        let mut sum = 0;
        if target <= sum {
            return Some(0);
        }
        for (i, x) in sorted.into_iter().enumerate() {
            sum += i128::from(x);
            if sum >= target {
                return Some(i + 1);
            }
        }
        None
    }

    fn add_queries(
        solver: &mut RangeMinimumSubsetSize,
        values: &[i64],
        want: &mut Vec<Option<usize>>,
    ) {
        let maxsum: i128 = values.iter().map(|&x| i128::from(x.max(0))).sum();
        for l in 0..=values.len() {
            for r in l..=values.len() {
                for target in -2..=maxsum + 2 {
                    assert_eq!(solver.query(l..r, target), want.len());
                    want.push(expected(values, l..r, target));
                }
            }
        }
    }

    #[test]
    fn test_range_minimum_subset_size() {
        let alphabet = [-2, 0, 1, 3];
        for mut values in exhaustive_sequences(alphabet, 0..=4) {
            let n = values.len();
            let mut solver = RangeMinimumSubsetSize::new(values.iter().copied());
            let mut want = vec![];
            add_queries(&mut solver, &values, &mut want);
            for p in 0..n {
                for &x in &alphabet {
                    let old = values[p];
                    solver.set(p, x);
                    values[p] = x;
                    add_queries(&mut solver, &values, &mut want);
                    solver.set(p, old);
                    values[p] = old;
                    add_queries(&mut solver, &values, &mut want);
                }
            }
            assert_eq!(solver.execute(), want);
        }
    }

    #[test]
    fn test_consecutive_updates() {
        let alphabet = [-2, 0, 1, 3];
        for initial in exhaustive_sequences(alphabet, 1..=3) {
            let n = initial.len();
            for p0 in 0..n {
                for &x0 in &alphabet {
                    for p1 in 0..n {
                        for &x1 in &alphabet {
                            let mut values = initial.clone();
                            let mut solver = RangeMinimumSubsetSize::new(values.iter().copied());
                            let mut want = vec![];
                            add_queries(&mut solver, &values, &mut want);
                            solver.set(p0, x0);
                            values[p0] = x0;
                            add_queries(&mut solver, &values, &mut want);
                            solver.set(p1, x1);
                            values[p1] = x1;
                            add_queries(&mut solver, &values, &mut want);
                            assert_eq!(solver.execute(), want);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_updates() {
        let mut rng = Xorshift::default();
        let boundaries = integer_boundary_values!(i64);
        for n in sample_usize(&mut rng, 4, 1..=128, 32) {
            let mut values: Vec<_> = rng.random_iter(-2..=3).take(n).collect();
            let mut solver = RangeMinimumSubsetSize::new(values.iter().copied());
            let mut expected_answers = Vec::new();
            for i in 0..boundaries.len() + 200 {
                let value = if i < boundaries.len() {
                    boundaries[i]
                } else {
                    rng.random(-2..=3)
                };
                let index = rng.random(0..n);
                solver.set(index, value);
                values[index] = value;
                let (l, r) = rng.random(WithEmptySegment(n));
                let sum: i128 = values[l..r].iter().map(|&x| i128::from(x.max(0))).sum();
                for target in [i128::MIN, -1, 0, 1, sum - 1, sum, sum + 1, i128::MAX] {
                    assert_eq!(solver.query(l..r, target), expected_answers.len());
                    expected_answers.push(expected(&values, l..r, target));
                }
            }
            assert_eq!(solver.execute(), expected_answers);
        }
        assert_eq!(RangeMinimumSubsetSize::new([]).execute(), []);
    }
}
