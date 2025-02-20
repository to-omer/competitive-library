use super::{Associative, Bounded, Magma, MaxOperation, MinOperation, SegmentTree, Unital};
use std::{cmp::Reverse, iter::Flatten, slice::Iter};

#[derive(Debug, Clone)]
/// max-heap
pub struct PartiallyRetroactivePriorityQueue<T>
where
    T: Clone + Ord + Bounded,
{
    n: usize,
    in_edges: SegmentTree<MaxOperation<(T, Reverse<usize>)>>,
    out_edges: SegmentTree<MinOperation<(T, usize)>>,
    flow: SegmentTree<SumMinimum>,
}

#[derive(Debug, Default, Clone, Copy)]
struct SumMinimum {
    sum: i32,
    prefix_min: i32,
    suffix_min: i32,
}

impl SumMinimum {
    fn singleton(x: i32) -> Self {
        Self {
            sum: x,
            prefix_min: 0.min(x),
            suffix_min: 0.max(x),
        }
    }
}

impl Magma for SumMinimum {
    type T = Self;
    fn operate(x: &Self::T, y: &Self::T) -> Self::T {
        Self {
            sum: x.sum + y.sum,
            prefix_min: x.prefix_min.min(x.sum + y.prefix_min),
            suffix_min: y.suffix_min.max(x.suffix_min + y.sum),
        }
    }
}

impl Associative for SumMinimum {}

impl Unital for SumMinimum {
    fn unit() -> Self::T {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct Changed<T> {
    pub inserted: [Option<T>; 2],
    pub removed: [Option<T>; 2],
}

impl<T> Changed<T> {
    pub fn inserted(&self) -> Flatten<Iter<'_, Option<T>>> {
        self.inserted.iter().flatten()
    }
    pub fn removed(&self) -> Flatten<Iter<'_, Option<T>>> {
        self.removed.iter().flatten()
    }
}

impl<T> Default for Changed<T> {
    fn default() -> Self {
        Self {
            inserted: [None, None],
            removed: [None, None],
        }
    }
}

impl<T> PartiallyRetroactivePriorityQueue<T>
where
    T: Clone + Ord + Bounded,
{
    pub fn new(n: usize) -> Self {
        let in_edges = SegmentTree::new(n);
        let out_edges = SegmentTree::new(n);
        let flow = SegmentTree::new(n);
        Self {
            n,
            in_edges,
            out_edges,
            flow,
        }
    }
    fn update_flow(&mut self, l: usize, r: usize, x: i32) {
        let s = self.flow.get(l).sum + x;
        self.flow.set(l, SumMinimum::singleton(s));
        let s = self.flow.get(r).sum - x;
        self.flow.set(r, SumMinimum::singleton(s));
    }
    pub unsafe fn set_push_unchecked(&mut self, i: usize, x: T) -> Option<T> {
        assert!(i < self.n);
        let p = self.flow.fold(i..self.n).sum;
        let j = if p < 0 {
            self.flow
                .rposition_acc(0..i, |s| s.suffix_min + p >= 0)
                .unwrap_or(0)
        } else {
            i
        };
        let (min, k) = self.out_edges.fold(j..self.n);
        if x <= min {
            self.in_edges.set(i, (x.clone(), Reverse(i)));
            return Some(x);
        }
        if i <= k {
            self.update_flow(i, k, 1);
        } else {
            self.update_flow(k, i, -1);
        }
        self.out_edges.set(i, (x.clone(), i));
        self.out_edges.clear(k);
        self.in_edges.set(k, (min.clone(), Reverse(k)));
        if min == T::minimum() {
            None
        } else {
            Some(min)
        }
    }
    pub unsafe fn unset_pop_unchecked(&mut self, i: usize) -> Option<T> {
        assert!(i < self.n);
        if self.out_edges.get(i) == (T::minimum(), i) {
            self.out_edges.clear(i);
            return None;
        }
        let p = self.flow.fold(i..self.n).sum;
        let j = if p < 0 {
            self.flow
                .rposition_acc(0..i, |s| s.suffix_min + p >= 0)
                .unwrap_or(0)
        } else {
            i
        };
        let (min, k) = self.out_edges.fold(j..self.n);
        assert_ne!(k, !0);
        if i <= k {
            self.update_flow(i, k, 1);
        } else {
            self.update_flow(k, i, -1);
        }
        self.in_edges.clear(i);
        self.out_edges.clear(k);
        self.in_edges.set(k, (min.clone(), Reverse(k)));
        if min == T::minimum() {
            None
        } else {
            Some(min)
        }
    }
    pub unsafe fn set_pop_unchecked(&mut self, i: usize) -> Option<T> {
        assert!(i < self.n);
        let p = self.flow.fold(0..=i).sum;
        let j = if p > 0 {
            self.flow
                .position_acc(i + 1..self.n - 1, |s| p + s.prefix_min <= 0)
                .unwrap_or(self.n - 1)
        } else {
            i
        };
        let (max, Reverse(k)) = self.in_edges.fold(0..=j);
        if max == T::minimum() {
            self.out_edges.set(i, (T::minimum(), i));
            return None;
        }
        if k <= i {
            self.update_flow(k, i, 1);
        } else {
            self.update_flow(i, k, -1);
        }
        self.in_edges.set(i, (T::minimum(), Reverse(i)));
        self.in_edges.clear(k);
        self.out_edges.set(k, (max.clone(), k));
        Some(max)
    }
    pub unsafe fn unset_push_unchecked(&mut self, i: usize) -> Option<T> {
        assert!(i < self.n);
        let (max, Reverse(k)) = self.in_edges.get(i);
        if k == i && max != T::minimum() {
            self.in_edges.clear(i);
            return Some(max);
        }
        let p = self.flow.fold(0..=i).sum;
        let j = if p > 0 {
            self.flow
                .position_acc(i + 1..self.n - 1, |s| p + s.prefix_min <= 0)
                .unwrap_or(self.n - 1)
        } else {
            i
        };
        let (max, Reverse(k)) = self.in_edges.fold(0..=j);
        if k <= i {
            self.update_flow(k, i, 1);
        } else {
            self.update_flow(i, k, -1);
        }
        self.out_edges.clear(i);
        self.in_edges.clear(k);
        self.out_edges.set(k, (max.clone(), k));
        if max == T::minimum() {
            None
        } else {
            Some(max)
        }
    }
    pub fn set_no_op(&mut self, i: usize) -> Changed<T> {
        assert!(i < self.n);
        let mut changed = Changed::default();
        let (max, Reverse(k)) = self.in_edges.get(i);
        let (min, kk) = self.out_edges.get(i);
        if k != i && kk != i {
            return changed;
        }
        if i == k && max == T::minimum() || i == kk && min == T::minimum() {
            changed.inserted[0] = unsafe { self.unset_pop_unchecked(i) };
        } else {
            changed.removed[0] = unsafe { self.unset_push_unchecked(i) };
        }
        changed
    }
    pub fn set_push(&mut self, i: usize, x: T) -> Changed<T> {
        assert!(i < self.n);
        let mut changed = self.set_no_op(i);
        changed.inserted[1] = unsafe { self.set_push_unchecked(i, x) };
        changed
    }
    pub fn set_pop(&mut self, i: usize) -> Changed<T> {
        assert!(i < self.n);
        let mut changed = self.set_no_op(i);
        changed.removed[1] = unsafe { self.set_pop_unchecked(i) };
        changed
    }
    #[allow(dead_code)]
    fn check(&self) -> Vec<T> {
        let mut pq = vec![];
        for i in 0..self.n {
            let (max, Reverse(k)) = self.in_edges.get(i);
            let (min, kk) = self.out_edges.get(i);
            if k == i {
                if max == T::minimum() {
                    // pop 1 element
                } else {
                    // push (not pop)
                    pq.push(max);
                }
            } else if kk == i {
                if min == T::minimum() {
                    // pop 0 element
                } else {
                    // push (poped)
                }
            } else {
                // nop
            }
        }
        pq.sort_unstable();
        pq
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::BinaryHeap;

    #[derive(Debug, Clone, Copy)]
    enum Query {
        Push(i64),
        Pop,
    }

    #[test]
    fn test_partially_retroactive_priority_queue() {
        let mut rng = Xorshift::default();
        for t in 0..100 {
            let n = rng.random(1..=100);
            let mut a = vec![None; n];
            let mut prpq = PartiallyRetroactivePriorityQueue::<i64>::new(n);
            let mut pq = Vec::new();
            for _ in 0..1000 {
                let i = rng.random(0..n);
                let q = if rng.gen_bool(t as f64 / 99.) {
                    Query::Push(rng.random(-3..=3))
                } else {
                    Query::Pop
                };
                a[i] = Some(q);
                let changed = match q {
                    Query::Push(x) => prpq.set_push(i, x),
                    Query::Pop => prpq.set_pop(i),
                };
                for &x in changed.inserted() {
                    pq.push(x);
                }
                for &x in changed.removed() {
                    if let Some(i) = pq.iter().position(|&y| x == y) {
                        pq.remove(i);
                    }
                }
                pq.sort_unstable();
                let mut heap = BinaryHeap::new();
                for q in &a {
                    match q {
                        Some(Query::Push(x)) => {
                            heap.push(*x);
                        }
                        Some(Query::Pop) => {
                            heap.pop();
                        }
                        None => {}
                    }
                }
                let heap = heap.into_sorted_vec();
                let pq_ = prpq.check();
                assert_eq!(pq, heap);
                assert_eq!(heap, pq_);
            }
        }
    }
}
