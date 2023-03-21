use crate::{graph::UndirectedSparseGraph, tools::Xorshift, tree::TreeCenter};

#[codesnip::entry("tree_hash", include("Xorshift", "tree_center"))]
#[derive(Default, Debug)]
pub struct TreeHasher {
    rv: Vec<u64>,
    rng: Xorshift,
}
#[codesnip::entry("tree_hash")]
impl TreeHasher {
    const MASK30: u64 = (1 << 30) - 1;
    const MASK31: u64 = (1 << 31) - 1;
    const MASK61: u64 = (1 << 61) - 1;
    const MOD: u64 = Self::MASK61;
    #[inline]
    fn mersenne_mod(a: u64) -> u64 {
        let mut res = (a >> 61) + (a & Self::MASK61);
        if res >= Self::MASK61 {
            res -= Self::MASK61;
        }
        res
    }
    #[inline]
    fn mersenne_mul(a: u64, b: u64) -> u64 {
        let au = a >> 31;
        let ad = a & Self::MASK31;
        let bu = b >> 31;
        let bd = b & Self::MASK31;
        let mid = ad * bu + au * bd;
        let midu = mid >> 30;
        let midd = mid & Self::MASK30;
        au * bu * 2 + midu + (midd << 31) + ad * bd
    }
    #[inline]
    fn mersenne_mul_mod(a: u64, b: u64) -> u64 {
        Self::mersenne_mod(Self::mersenne_mul(a, b))
    }
    pub fn new() -> Self {
        Self {
            rv: Vec::new(),
            rng: Xorshift::new(),
        }
    }
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rv: Vec::new(),
            rng: Xorshift::new_with_seed(seed),
        }
    }
    pub fn hash(&mut self, g: &UndirectedSparseGraph) -> u64 {
        match g.tree_center() {
            TreeCenter::One(u) => self.hash_rec(g, u, !0, 0),
            TreeCenter::Two(u, v) => {
                Self::mersenne_mul_mod(self.hash_rooted(g, u, v), self.hash_rooted(g, v, u))
            }
        }
    }
    pub fn hash_rooted(&mut self, g: &UndirectedSparseGraph, root: usize, parent: usize) -> u64 {
        self.hash_rec(g, root, parent, 0)
    }
    fn hash_rec(&mut self, g: &UndirectedSparseGraph, u: usize, p: usize, d: usize) -> u64 {
        let mut s = 1u64;
        if self.rv.len() <= d {
            self.rv.push(Self::mersenne_mod(self.rng.rand64()));
        }
        for a in g.adjacencies(u) {
            if a.to != p {
                s = Self::mersenne_mul_mod(s, self.hash_rec(g, a.to, u, d + 1));
            }
        }
        s += self.rv[d];
        if s >= Self::MOD {
            s -= Self::MOD;
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::MixedTree;
    use std::{
        cmp::Ordering,
        collections::{BTreeMap, HashMap},
    };

    #[allow(clippy::ptr_arg)]
    fn vec_len_cmp<T: Ord>(left: &Vec<T>, right: &Vec<T>) -> Ordering {
        match left.len().cmp(&right.len()) {
            Ordering::Equal => left.cmp(right),
            non_eq => non_eq,
        }
    }

    impl UndirectedSparseGraph {
        fn canonical(&self) -> Vec<bool> {
            match self.tree_center() {
                TreeCenter::One(u) => self.canonical_dfs(u, !0),
                TreeCenter::Two(u, v) => {
                    let mut a = self.canonical_dfs(u, v);
                    let mut b = self.canonical_dfs(v, u);
                    match vec_len_cmp(&a, &b) {
                        Ordering::Less | Ordering::Equal => {
                            a.append(&mut b);
                            a
                        }
                        Ordering::Greater => {
                            b.append(&mut a);
                            b
                        }
                    }
                }
            }
        }
        fn canonical_dfs(&self, u: usize, p: usize) -> Vec<bool> {
            let mut v = vec![vec![false]];
            for a in self.adjacencies(u) {
                if a.to != p {
                    v.push(self.canonical_dfs(a.to, u));
                }
            }
            v.sort_unstable_by(vec_len_cmp);
            v.push(vec![true]);
            v.into_iter().flatten().collect()
        }
    }

    #[test]
    fn test_tree_hash() {
        const N: usize = 200;
        const Q: usize = 1000;
        let mut rng = Xorshift::default();
        let mut hasher = TreeHasher::new();
        let mut h2s = HashMap::<u64, Vec<bool>>::new();
        let mut s2h = BTreeMap::new();
        for g in rng.gen_iter(MixedTree(1..=N)).take(Q) {
            let h = hasher.hash(&g);
            let s = g.canonical();
            h2s.entry(h)
                .and_modify(|v| assert_eq!(v, &s))
                .or_insert_with(|| s.clone());
            s2h.entry(s).and_modify(|v| assert_eq!(*v, h)).or_insert(h);
            assert_eq!(h2s.len(), s2h.len());
        }
        let mut v: Vec<_> = s2h.values().collect();
        v.sort();
        v.dedup();
        assert_eq!(v.len(), s2h.len());
        let mut v: Vec<_> = h2s.values().collect();
        v.sort();
        v.dedup();
        assert_eq!(v.len(), h2s.len());
    }
}
