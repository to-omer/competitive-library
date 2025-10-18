use std::ops::Range;

#[derive(Debug, Clone)]
pub struct CartesianTree {
    pub root: usize,
    pub parents: Vec<usize>,
    pub children: Vec<[usize; 2]>,
}

impl CartesianTree {
    pub fn new<T>(a: &[T]) -> Self
    where
        T: PartialOrd,
    {
        let mut parents = vec![!0; a.len()];
        let mut children = vec![[!0; 2]; a.len()];
        let mut stack = vec![];
        for i in 0..a.len() {
            let mut prev = !0usize;
            while let Some(last) = stack.pop_if(|last| a[i] < a[*last]) {
                prev = last;
            }
            if prev != !0 {
                parents[prev] = i;
            }
            if let Some(&last) = stack.last() {
                parents[i] = last;
            }
            stack.push(i);
        }
        let mut root = !0;
        for i in 0..a.len() {
            if parents[i] != !0 {
                children[parents[i]][(i > parents[i]) as usize] = i;
            } else {
                root = i;
            }
        }
        Self {
            root,
            parents,
            children,
        }
    }
    pub fn with_ranges(&self, mut f: impl FnMut(usize, Range<usize>)) {
        let mut stack = vec![(self.root, 0, self.parents.len())];
        while let Some((v, l, r)) = stack.pop() {
            f(v, l..r);
            if self.children[v][1] != !0 {
                stack.push((self.children[v][1], v + 1, r));
            }
            if self.children[v][0] != !0 {
                stack.push((self.children[v][0], l, v));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::MinOperation, crecurse, data_structure::SegmentTree, rand, tools::Xorshift,
    };

    #[test]
    fn test_cartesian_tree() {
        const Q: usize = 1000;
        const N: usize = 100;
        const A: i64 = 100;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N, a: [0..A; n]);
            let mut seg = SegmentTree::<MinOperation<_>>::from_vec(
                a.iter().enumerate().map(|(i, &a)| (a, i)).collect(),
            );
            let mut parents = vec![!0; n];
            let mut root = !0;
            let mut children = vec![[!0; 2]; n];
            let mut ranges = vec![];
            crecurse!(
                unsafe fn dfs(l: usize, r: usize, p: usize, ci: usize) {
                    if l >= r {
                        return;
                    }
                    let m = seg.fold(l..r).1;
                    ranges.push((m, l..r));
                    if p == !0 {
                        root = m;
                    } else {
                        parents[m] = p;
                        children[p][ci] = m;
                    }
                    seg.set(m, (i64::MAX, usize::MAX));
                    dfs!(l, m, m, 0);
                    dfs!(m + 1, r, m, 1);
                }
            )(0, n, !0, !0);

            let ct = CartesianTree::new(&a);
            assert_eq!(ct.root, root);
            assert_eq!(ct.parents, parents);
            assert_eq!(ct.children, children);
            let mut ct_ranges = vec![];
            ct.with_ranges(|v, range| {
                ct_ranges.push((v, range));
            });
            assert_eq!(ct_ranges, ranges);
        }
    }
}
