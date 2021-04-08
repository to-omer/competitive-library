#[codesnip::skip]
use crate::{
    graph::UndirectedSparseGraph,
    tools::{RandomSpec, Xorshift},
};

/// Generate Tree with Prüfer sequence
pub struct PruferSequence<T>(pub T);

impl<T: RandomSpec<usize>> RandomSpec<UndirectedSparseGraph> for PruferSequence<T> {
    fn rand(&self, rng: &mut Xorshift) -> UndirectedSparseGraph {
        let n = rng.gen(&self.0);
        let edges = from_prufer_sequence(
            n,
            &rng.gen_iter(0..n)
                .take(n.saturating_sub(2))
                .collect::<Vec<usize>>(),
        );
        UndirectedSparseGraph::from_edges(n, edges)
    }
}

pub struct PathTree<T>(pub T);

impl<T: RandomSpec<usize>> RandomSpec<UndirectedSparseGraph> for PathTree<T> {
    fn rand(&self, rng: &mut Xorshift) -> UndirectedSparseGraph {
        let n = rng.gen(&self.0);
        let edges = (1..n).map(|u| (u - 1, u)).collect();
        UndirectedSparseGraph::from_edges(n, edges)
    }
}

pub struct StarTree<T>(pub T);

impl<T: RandomSpec<usize>> RandomSpec<UndirectedSparseGraph> for StarTree<T> {
    fn rand(&self, rng: &mut Xorshift) -> UndirectedSparseGraph {
        let n = rng.gen(&self.0);
        let edges = (1..n).map(|u| (0, u)).collect();
        UndirectedSparseGraph::from_edges(n, edges)
    }
}

pub struct MixedTree<T>(pub T);

impl<T: RandomSpec<usize>> RandomSpec<UndirectedSparseGraph> for MixedTree<T> {
    fn rand(&self, rng: &mut Xorshift) -> UndirectedSparseGraph {
        fn rand_inner(n: usize, rng: &mut Xorshift) -> Vec<(usize, usize)> {
            let mut edges = Vec::with_capacity(n.saturating_sub(1));
            if n >= 2 {
                let k = rng.gen(1..n);
                for n in [k, n - k].iter().cloned() {
                    let ty = rng.rand(6);
                    edges.extend(match ty {
                        0 => from_prufer_sequence(
                            n,
                            &rng.gen_iter(0..n)
                                .take(n.saturating_sub(2))
                                .collect::<Vec<usize>>(),
                        ),
                        1 => (1..n).map(|u| (u - 1, u)).collect(),
                        2 => (1..n).map(|u| (0, u)).collect(),
                        _ => rand_inner(n, rng),
                    });
                }
                for (u, v) in edges[k - 1..].iter_mut() {
                    *u += k;
                    *v += k;
                }
                edges.push((rng.gen(0..k), rng.gen(k..n)));
            }
            edges
        }
        let n = rng.gen(&self.0);
        let edges = rand_inner(n, rng);
        UndirectedSparseGraph::from_edges(n, edges)
    }
}

fn from_prufer_sequence(n: usize, prufer: &[usize]) -> Vec<(usize, usize)> {
    use std::collections::BinaryHeap;
    let mut edges = Vec::with_capacity(n.saturating_sub(1));
    if n >= 2 {
        let mut deg = vec![0usize; n];
        prufer.iter().for_each(|&a| deg[a] += 1);
        let mut heap: BinaryHeap<usize> = (0..n).filter(|&u| deg[u] == 0).collect();
        for &a in prufer {
            deg[a] -= 1;
            let b = heap.pop().unwrap();
            edges.push((a, b));
            if deg[a] == 0 {
                heap.push(a);
            }
        }
        edges.push((heap.pop().unwrap(), heap.pop().unwrap()));
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_connected(g: &UndirectedSparseGraph) -> bool {
        let n = g.vertices_size();
        if n == 0 {
            return true;
        }
        let mut vis = vec![false; n];
        let mut stack = vec![0];
        let mut acc = 0usize;
        vis[0] = true;
        while let Some(u) = stack.pop() {
            acc += 1;
            for a in g.adjacencies(u) {
                if !vis[a.to] {
                    vis[a.to] = true;
                    stack.push(a.to);
                }
            }
        }
        acc == n
    }

    fn is_tree(g: &UndirectedSparseGraph) -> bool {
        let n = g.vertices_size();
        let m = g.edges_size();
        (n == 0 || m + 1 == n) && is_connected(g)
    }

    #[test]
    fn prufer_sequence_small() {
        const Q: usize = 10_000;
        const N: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let g = rng.gen(PruferSequence(1..=N));
            assert!(is_tree(&g));
        }
    }

    #[test]
    fn prufer_sequence_big() {
        const Q: usize = 20;
        const N: usize = 10_000;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let g = rng.gen(PruferSequence(N - Q..=N));
            assert!(is_tree(&g));
        }
    }

    #[test]
    fn path_small() {
        const N: usize = 20;
        let mut rng = Xorshift::default();
        for n in 0..=N {
            let g = rng.gen(PathTree(n));
            assert!(is_tree(&g));
        }
    }

    #[test]
    fn path_big() {
        const Q: usize = 20;
        const N: usize = 10_000;
        let mut rng = Xorshift::default();
        for n in N - Q..=N {
            let g = rng.gen(PathTree(n));
            assert!(is_tree(&g));
        }
    }

    #[test]
    fn star_small() {
        const N: usize = 20;
        let mut rng = Xorshift::default();
        for n in 0..=N {
            let g = rng.gen(StarTree(n));
            assert!(is_tree(&g));
        }
    }

    #[test]
    fn star_big() {
        const Q: usize = 20;
        const N: usize = 10_000;
        let mut rng = Xorshift::default();
        for n in N - Q..=N {
            let g = rng.gen(StarTree(n));
            assert!(is_tree(&g));
        }
    }

    #[test]
    fn mixed_small() {
        const Q: usize = 10_000;
        const N: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let g = rng.gen(MixedTree(1..=N));
            assert!(is_tree(&g));
        }
    }

    #[test]
    fn mixed_big() {
        const Q: usize = 20;
        const N: usize = 10_000;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let g = rng.gen(MixedTree(N - Q..=N));
            assert!(is_tree(&g));
        }
    }
}
