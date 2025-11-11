use super::{union_find, UnionFindBase};
use std::{cmp::Ordering, collections::BinaryHeap, ops::AddAssign};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Count01 {
    cnt0: usize,
    cnt1: usize,
}

impl PartialOrd for Count01 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Count01 {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.cnt1 == 0, other.cnt1 == 0) {
            (true, true) => self.cnt0.cmp(&other.cnt0),
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => (self.cnt0 * other.cnt1).cmp(&(other.cnt0 * self.cnt1)),
        }
    }
}

impl AddAssign for Count01 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            cnt0: self.cnt0 + other.cnt0,
            cnt1: self.cnt1 + other.cnt1,
        }
    }
}

impl Count01 {
    pub fn new(cnt0: usize, cnt1: usize) -> Self {
        Self { cnt0, cnt1 }
    }
}

pub fn solve_01_on_tree(
    n: usize,
    c01: impl Fn(usize) -> (usize, usize),
    root: usize,
    parent: impl Fn(usize) -> usize,
) -> (usize, Vec<usize>) {
    pub type UF<T, M> =
        UnionFindBase<(), union_find::PathCompression, union_find::FnMerger<T, M>, (), ()>;
    let mut cost = 0usize;
    let c01 = |u| {
        let c = c01(u);
        Count01::new(c.0, c.1)
    };
    let mut uf = UF::new_with_merger(n, &c01, |x, y| {
        cost += x.cnt1 * y.cnt0;
        *x += *y;
    });
    let mut label = vec![0; n];
    let mut heap = BinaryHeap::from_iter((0..n).filter(|&u| u != root).map(|u| (c01(u), u, 0)));
    let mut next: Vec<_> = (0..n).collect();
    let mut ord = Vec::with_capacity(n);
    while let Some((_c, u, l)) = heap.pop() {
        if label[u] != l {
            continue;
        }
        let p = uf.find_root(parent(u));
        uf.unite(u, p);
        if !uf.same(p, root) {
            label[p] += 1;
            heap.push((*uf.merge_data(p), p, label[p]));
        }
        next.swap(u, p);
    }
    let mut u = next[root];
    ord.push(u);
    while u != root {
        u = next[u];
        ord.push(u);
    }
    ord.reverse();
    (cost, ord)
}
