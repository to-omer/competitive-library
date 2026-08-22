use super::{Bounded, IntBase, RadixSortKey, SliceSortExt, Zero};
use std::{
    mem::swap,
    ops::{Add, Mul, Range},
};

pub trait LiChaoLine<X>: Copy {
    type Output: Bounded + Copy + Ord;

    fn infinity() -> Self;
    fn evaluate(&self, x: X) -> Self::Output;
}

impl<X, Y> LiChaoLine<X> for (X, Y)
where
    X: Copy + Into<Y> + Zero,
    Y: Bounded + Copy + Ord + Add<Output = Y> + Mul<Output = Y>,
{
    type Output = Y;

    fn infinity() -> Self {
        (X::zero(), Y::maximum())
    }

    fn evaluate(&self, x: X) -> Self::Output {
        self.0.into() * x.into() + self.1
    }
}

enum Branch {
    Left,
    Right,
}

fn place_line<X, L>(
    current: &mut L,
    candidate: &mut L,
    left: X,
    middle: X,
    right: X,
    candidate_left: L::Output,
    candidate_right: L::Output,
) -> Option<(Branch, L::Output, L::Output)>
where
    X: Copy,
    L: LiChaoLine<X>,
{
    let current_left = current.evaluate(left);
    let current_right = current.evaluate(right);
    if candidate_left < current_left {
        if candidate_right < current_right {
            swap(current, candidate);
            return None;
        }
        let candidate_middle = candidate.evaluate(middle);
        let current_middle = current.evaluate(middle);
        if candidate_middle < current_middle {
            swap(current, candidate);
            Some((Branch::Right, current_middle, current_right))
        } else {
            Some((Branch::Left, candidate_left, candidate_middle))
        }
    } else if candidate_right < current_right {
        let candidate_middle = candidate.evaluate(middle);
        let current_middle = current.evaluate(middle);
        if candidate_middle < current_middle {
            swap(current, candidate);
            Some((Branch::Left, current_left, current_middle))
        } else {
            Some((Branch::Right, candidate_middle, candidate_right))
        }
    } else {
        None
    }
}

#[derive(Debug, Clone)]
struct LiChaoSegment<X, L> {
    range: Range<X>,
    line: L,
}

impl<X, L> LiChaoSegment<X, L>
where
    X: Copy + Ord,
    L: LiChaoLine<X>,
{
    fn evaluate(&self, x: X) -> L::Output {
        if self.range.contains(&x) {
            self.line.evaluate(x)
        } else {
            L::infinity().evaluate(x)
        }
    }

    fn covers(&self, left: X, right: X) -> bool {
        self.range.start == left && self.range.end == right
    }
}

#[derive(Debug, Clone)]
struct LiChaoNode<X, L> {
    segment: LiChaoSegment<X, L>,
    children: [u32; 2],
}

#[derive(Debug, Clone)]
pub struct LiChaoTree<X, L> {
    range: Range<X>,
    nodes: Vec<LiChaoNode<X, L>>,
}

impl<X, L> LiChaoTree<X, L>
where
    X: IntBase,
    L: LiChaoLine<X>,
{
    pub fn new(range: Range<X>) -> Self {
        assert!(range.start < range.end);
        Self {
            range: range.clone(),
            nodes: vec![LiChaoNode {
                segment: LiChaoSegment {
                    range,
                    line: L::infinity(),
                },
                children: [!0; 2],
            }],
        }
    }

    fn push_node(&mut self, segment: LiChaoSegment<X, L>) -> u32 {
        let index = self.nodes.len() as u32;
        self.nodes.push(LiChaoNode {
            segment,
            children: [!0; 2],
        });
        index
    }

    pub fn add_line(&mut self, line: L) {
        self.add_segment_at(
            0,
            LiChaoSegment {
                range: self.range.clone(),
                line,
            },
            self.range.start,
            self.range.end,
        );
    }

    pub fn add_segment(&mut self, range: Range<X>, line: L) {
        assert!(self.range.start <= range.start && range.end <= self.range.end);
        if range.start < range.end {
            self.add_segment_at(
                0,
                LiChaoSegment { range, line },
                self.range.start,
                self.range.end,
            );
        }
    }

    fn add_segment_at(
        &mut self,
        mut index: u32,
        mut segment: LiChaoSegment<X, L>,
        mut left: X,
        mut right: X,
    ) {
        loop {
            let last = right - X::one();
            let middle = if left == last {
                left
            } else {
                left.midpoint(last).min(last - X::one())
            };
            let split = middle + X::one();
            if self.nodes[index as usize].segment.covers(left, right) && segment.covers(left, right)
            {
                let candidate_left = segment.line.evaluate(left);
                let candidate_right = segment.line.evaluate(last);
                let child = match place_line(
                    &mut self.nodes[index as usize].segment.line,
                    &mut segment.line,
                    left,
                    middle,
                    last,
                    candidate_left,
                    candidate_right,
                ) {
                    None => return,
                    Some((Branch::Left, _, _)) => {
                        right = split;
                        segment.range.end = right;
                        0
                    }
                    Some((Branch::Right, _, _)) => {
                        left = split;
                        segment.range.start = left;
                        1
                    }
                };
                let next = self.nodes[index as usize].children[child];
                if next == !0 {
                    let next = self.push_node(segment);
                    self.nodes[index as usize].children[child] = next;
                    return;
                }
                index = next;
                continue;
            }
            let segment_right = segment.range.end - X::one();
            if self.nodes[index as usize]
                .segment
                .evaluate(segment.range.start)
                <= segment.line.evaluate(segment.range.start)
                && self.nodes[index as usize].segment.evaluate(segment_right)
                    <= segment.line.evaluate(segment_right)
            {
                return;
            }
            let current = &self.nodes[index as usize].segment;
            let current_left = current.range.start;
            let current_right = current.range.end - X::one();
            if current.line.evaluate(current_left) >= segment.evaluate(current_left)
                && current.line.evaluate(current_right) >= segment.evaluate(current_right)
            {
                self.nodes[index as usize].segment = segment;
                return;
            }
            if segment.covers(left, right) {
                swap(&mut self.nodes[index as usize].segment, &mut segment);
            }
            let child;
            if segment.range.end <= split {
                child = 0;
                right = split;
            } else if middle < segment.range.start {
                child = 1;
                left = split;
            } else {
                let right_segment = LiChaoSegment {
                    range: split..segment.range.end,
                    line: segment.line,
                };
                segment.range.end = split;
                let next = self.nodes[index as usize].children[0];
                if next == !0 {
                    let next = self.push_node(segment);
                    self.nodes[index as usize].children[0] = next;
                } else {
                    self.add_segment_at(next, segment, left, split);
                }
                let next = self.nodes[index as usize].children[1];
                if next == !0 {
                    let next = self.push_node(right_segment);
                    self.nodes[index as usize].children[1] = next;
                } else {
                    self.add_segment_at(next, right_segment, split, right);
                }
                return;
            }
            let next = self.nodes[index as usize].children[child];
            if next == !0 {
                let next = self.push_node(segment);
                self.nodes[index as usize].children[child] = next;
                return;
            }
            index = next;
        }
    }

    pub fn query_min(&self, x: X) -> Option<L::Output> {
        assert!(self.range.contains(&x));
        let infinity = L::infinity().evaluate(x);
        let mut result = infinity;
        let (mut index, mut left, mut right) = (0, self.range.start, self.range.end);
        while index != !0 {
            let node = &self.nodes[index as usize];
            result = result.min(node.segment.evaluate(x));
            let last = right - X::one();
            let middle = if left == last {
                left
            } else {
                left.midpoint(last).min(last - X::one())
            };
            let split = middle + X::one();
            if x <= middle {
                index = node.children[0];
                right = split;
            } else {
                index = node.children[1];
                left = split;
            }
        }
        (result != infinity).then_some(result)
    }
}

#[derive(Debug, Clone, Copy)]
enum LiChaoEvent<X, L> {
    Line(L),
    Segment(X, X, L),
    Query(X, u32),
}

#[derive(Debug, Clone)]
pub struct OfflineLiChaoTree<X, L> {
    events: Vec<LiChaoEvent<X, L>>,
    queries: usize,
}

impl<X, L> Default for OfflineLiChaoTree<X, L> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            queries: 0,
        }
    }
}

impl<X, L> OfflineLiChaoTree<X, L>
where
    X: Copy + Ord + RadixSortKey,
    L: LiChaoLine<X>,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_line(&mut self, line: L) {
        self.events.push(LiChaoEvent::Line(line));
    }

    pub fn add_segment(&mut self, range: Range<X>, line: L) {
        self.events
            .push(LiChaoEvent::Segment(range.start, range.end, line));
    }

    pub fn query_min(&mut self, x: X) -> usize {
        let index = self.queries;
        self.events.push(LiChaoEvent::Query(x, index as u32));
        self.queries += 1;
        index
    }

    pub fn execute(self) -> Vec<Option<L::Output>> {
        let mut markers = Vec::with_capacity(2 * self.events.len());
        for (i, event) in self.events.iter().enumerate() {
            let i = i as u32;
            match *event {
                LiChaoEvent::Line(_) => {}
                LiChaoEvent::Segment(left, right, _) => {
                    markers.push((left, i << 2));
                    markers.push((right, i << 2 | 1));
                }
                LiChaoEvent::Query(x, _) => markers.push((x, i << 2 | 2)),
            }
        }
        markers.radix_sort_by_key(|&(x, _)| x);

        let mut positions = vec![[0u32; 2]; self.events.len()];
        let mut coordinates = Vec::with_capacity(self.queries);
        let mut left = 0;
        while left < markers.len() {
            let x = markers[left].0;
            let mut right = left + 1;
            while right < markers.len() && markers[right].0 == x {
                right += 1;
            }
            let index = coordinates.len() as u32;
            let mut queried = false;
            for &(_, marker) in &markers[left..right] {
                let event = (marker >> 2) as usize;
                match marker & 3 {
                    0 => positions[event][0] = index,
                    1 => positions[event][1] = index,
                    _ => {
                        positions[event][0] = index;
                        queried = true;
                    }
                }
            }
            if queried {
                coordinates.push(x);
            }
            left = right;
        }
        if let Some(&x) = coordinates.last() {
            coordinates.resize(coordinates.len().next_power_of_two(), x);
            coordinates.push(x);
        }

        let mut tree = IndexedLiChaoTree::new(&coordinates);
        let mut result = vec![None; self.queries];
        for (i, event) in self.events.into_iter().enumerate() {
            match event {
                LiChaoEvent::Line(line) => tree.add_line(line),
                LiChaoEvent::Segment(_, _, line) => {
                    let [left, right] = positions[i];
                    tree.add_segment(left as usize..right as usize, line);
                }
                LiChaoEvent::Query(_, output) => {
                    result[output as usize] = tree.query_min(positions[i][0] as usize);
                }
            }
        }
        result
    }
}

struct IndexedLiChaoTree<'a, X, L>
where
    L: LiChaoLine<X>,
{
    size: usize,
    coordinates: &'a [X],
    lines: Vec<L>,
}

impl<X, L> IndexedLiChaoTree<'_, X, L>
where
    X: Copy,
    L: LiChaoLine<X>,
{
    fn new(coordinates: &[X]) -> IndexedLiChaoTree<'_, X, L> {
        let size = coordinates.len().saturating_sub(1);
        IndexedLiChaoTree {
            size,
            coordinates,
            lines: vec![L::infinity(); 2 * size],
        }
    }

    fn add_line(&mut self, line: L) {
        if self.size != 0 {
            self.add_line_at(1, self.size.trailing_zeros() as usize, line);
        }
    }

    fn add_line_at(&mut self, mut index: usize, height: usize, mut line: L) {
        let mut left = (index << height) - self.size;
        let mut right = left + (1 << height);
        let mut values = (
            line.evaluate(self.coordinates[left]),
            line.evaluate(self.coordinates[right]),
        );
        loop {
            if left + 1 == right {
                if values.0 < self.lines[index].evaluate(self.coordinates[left]) {
                    self.lines[index] = line;
                }
                return;
            }
            let middle = (left + right) / 2;
            match place_line(
                &mut self.lines[index],
                &mut line,
                self.coordinates[left],
                self.coordinates[middle],
                self.coordinates[right],
                values.0,
                values.1,
            ) {
                None => return,
                Some((Branch::Left, left_value, right_value)) => {
                    index *= 2;
                    right = middle;
                    values = (left_value, right_value);
                }
                Some((Branch::Right, left_value, right_value)) => {
                    index = 2 * index + 1;
                    left = middle;
                    values = (left_value, right_value);
                }
            }
        }
    }

    fn add_segment(&mut self, range: Range<usize>, line: L) {
        let n = self.size;
        if range.start == range.end {
            return;
        }
        let mut left = n + range.start - 1;
        let mut right = n + range.end;
        let width = (left ^ right).ilog2();
        let mask = (1usize << width) - 1;
        let fixed = left;
        left = !left & mask;
        while left != 0 {
            let height = left.trailing_zeros();
            left &= left - 1;
            self.add_line_at((fixed >> height) ^ 1, height as usize, line);
        }
        let fixed = right;
        right &= mask;
        while right != 0 {
            let height = right.trailing_zeros();
            right &= right - 1;
            self.add_line_at((fixed >> height) ^ 1, height as usize, line);
        }
    }

    fn query_min(&self, index: usize) -> Option<L::Output> {
        let x = self.coordinates[index];
        let infinity = L::infinity().evaluate(x);
        let mut result = infinity;
        let mut index = self.size + index;
        while index != 0 {
            result = result.min(self.lines[index].evaluate(x));
            index >>= 1;
        }
        (result != infinity).then_some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    #[test]
    fn test_li_chao_tree() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let mut online: LiChaoTree<i32, (i32, i64)> = LiChaoTree::new(-21..22);
            let mut offline: OfflineLiChaoTree<i32, (i32, i64)> = OfflineLiChaoTree::new();
            let mut lines = Vec::new();
            let mut expected = Vec::new();
            for _ in 0..200 {
                rand!(rng, ty: 0..3, mut l: -20..=20, mut r: -20..=20, a: -20..=20, b: -100..=100, x: -20..=20);
                if l > r {
                    swap(&mut l, &mut r);
                }
                match ty {
                    0 => {
                        online.add_line((a, b));
                        offline.add_line((a, b));
                        lines.push((-21..22, a, b));
                    }
                    1 => {
                        online.add_segment(l..r, (a, b));
                        offline.add_segment(l..r, (a, b));
                        lines.push((l..r, a, b));
                    }
                    _ => {
                        let result = lines
                            .iter()
                            .filter(|(range, _, _)| range.contains(&x))
                            .map(|(_, a, b)| i64::from(*a) * i64::from(x) + b)
                            .min();
                        assert_eq!(online.query_min(x), result);
                        offline.query_min(x);
                        expected.push(result);
                    }
                }
            }
            assert_eq!(offline.execute(), expected);
        }
    }
}
