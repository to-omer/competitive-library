use crate::graph::SparseGraph;

#[codesnip::entry("tree_center")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreeCenter {
    One(usize),
    Two(usize, usize),
}
#[codesnip::entry("tree_center", include("SparseGraph"))]
impl<D> SparseGraph<D> {
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
    use crate::graph::UndirectedSparseGraph;

    #[test]
    fn test_center() {
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
            UndirectedSparseGraph::from_edges(6, vec![(0, 1), (1, 2), (1, 3), (3, 4)])
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
}
