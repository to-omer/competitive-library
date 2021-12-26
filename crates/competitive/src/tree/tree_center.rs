use crate::graph::UndirectedSparseGraph;

#[codesnip::entry("tree_center")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreeCenter {
    One(usize),
    Two(usize, usize),
}
#[codesnip::entry("tree_center", include("SparseGraph"))]
impl UndirectedSparseGraph {
    /// tree center
    pub fn tree_center(&self) -> TreeCenter {
        let n = self.vertices_size();
        assert_ne!(n, 0);
        let mut deq = std::collections::VecDeque::with_capacity(n);
        let mut deg: Vec<_> = self.vertices().map(|u| self.adjacencies(u).len()).collect();
        for u in self.vertices() {
            if self.adjacencies(u).len() <= 1 {
                deq.push_back(u);
            }
        }
        let mut k = 0;
        let mut cnt = deq.len();
        if cnt < n {
            k = deq.len();
            'outer: while let Some(u) = deq.pop_front() {
                k -= 1;
                for a in self.adjacencies(u) {
                    deg[a.to] -= 1;
                    if deg[a.to] == 1 {
                        deq.push_back(a.to);
                        cnt += 1;
                        if cnt == n {
                            break 'outer;
                        }
                    }
                }
                if k == 0 {
                    k = deq.len();
                }
            }
        }
        if deq.len() == k + 1 {
            TreeCenter::One(*deq.back().unwrap())
        } else {
            let u = deq.pop_back().unwrap();
            let v = deq.pop_back().unwrap();
            if u < v {
                TreeCenter::Two(u, v)
            } else {
                TreeCenter::Two(v, u)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graph::UndirectedSparseGraph, tools::Xorshift, tree::MixedTree};

    impl UndirectedSparseGraph {
        fn naive_tree_center(&self) -> TreeCenter {
            let mut md: Vec<_> = self
                .vertices()
                .map(|u| (self.tree_depth(u).into_iter().max().unwrap_or_default(), u))
                .collect();
            md.sort_unstable();
            if md.len() == 1 {
                TreeCenter::One(md[0].1)
            } else if md.len() >= 2 {
                if md[0].0 == md[1].0 {
                    TreeCenter::Two(md[0].1, md[1].1)
                } else {
                    TreeCenter::One(md[0].1)
                }
            } else {
                panic!("vertex size should be larger than one.");
            }
        }
    }

    #[test]
    fn test_center_handmaid() {
        assert_eq!(
            UndirectedSparseGraph::from_edges(1, vec![]).tree_center(),
            TreeCenter::One(0)
        );
        assert_eq!(
            UndirectedSparseGraph::from_edges(2, vec![(0, 1)]).tree_center(),
            TreeCenter::Two(0, 1)
        );
        assert_eq!(
            UndirectedSparseGraph::from_edges(3, vec![(0, 1), (0, 2)]).tree_center(),
            TreeCenter::One(0)
        );
        assert_eq!(
            UndirectedSparseGraph::from_edges(4, vec![(0, 1), (1, 2), (1, 3)]).tree_center(),
            TreeCenter::One(1)
        );
        assert_eq!(
            UndirectedSparseGraph::from_edges(5, vec![(0, 1), (1, 2), (1, 3), (3, 4)])
                .tree_center(),
            TreeCenter::Two(1, 3)
        );
        assert_eq!(
            UndirectedSparseGraph::from_edges(
                7,
                vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)]
            )
            .tree_center(),
            TreeCenter::One(3)
        );
    }

    #[test]
    fn test_center_random() {
        let mut rng = Xorshift::default();
        const N: usize = 200;
        const Q: usize = 200;
        for _ in 0..Q {
            let n = rng.gen(1..=N);
            let g = rng.gen(MixedTree(n));
            assert_eq!(g.tree_center(), g.naive_tree_center());
        }
    }
}
