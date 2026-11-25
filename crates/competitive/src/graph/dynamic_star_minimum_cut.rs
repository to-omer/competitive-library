use super::{Cluster, StaticTopTreeDp, UndirectedSparseGraph, extreme_set_tree};

/// Global minimum cut of an undirected graph with mutable edges to an extra vertex.
pub struct DynamicStarMinimumCut {
    n: usize,
    cuts: StaticTopTreeDp<'static, CutCluster>,
}

impl DynamicStarMinimumCut {
    /// Each star weight connects its vertex to the extra vertex.
    /// Takes the same fixed graphs as [`extreme_set_tree`].
    pub fn new(star: Vec<u64>, edges: impl IntoIterator<Item = (usize, usize, u64)>) -> Self {
        let n = star.len();
        let (parent, boundary) = extreme_set_tree(n, edges);
        let root = parent.len();
        let graph =
            UndirectedSparseGraph::from_edges(root + 1, parent.into_iter().enumerate().collect());
        let vertices = boundary
            .into_iter()
            .zip(star.into_iter().chain(std::iter::repeat(0)))
            .collect();
        let cuts = graph
            .static_top_tree(root)
            .into_dp(vertices, vec![(); root]);
        Self { n, cuts }
    }

    pub fn minimum_cut(&self) -> u128 {
        self.cuts.fold_all().1
    }

    /// Replaces the weight of the edge from vertex `v` to the extra vertex in O(log n) time.
    pub fn set_weight(&mut self, v: usize, weight: u64) {
        assert!(v < self.n);
        self.cuts.apply_vertex(v, |vertex| vertex.1 = weight);
    }
}

struct CutCluster;

#[derive(Clone, Copy)]
struct CutPath {
    sum: u128,
    on_path: u128,
    off_path: u128,
}

impl Cluster for CutCluster {
    type Vertex = (u128, u64);
    type Edge = ();
    type Path = CutPath;
    type Point = (u128, u128);

    fn unit_path() -> CutPath {
        CutPath {
            sum: 0,
            on_path: u128::MAX,
            off_path: u128::MAX,
        }
    }
    fn unit_point() -> Self::Point {
        (0, u128::MAX)
    }
    fn compress(upper: &CutPath, lower: &CutPath) -> CutPath {
        // All leaves below the joining edge contribute to every candidate on the upper path.
        CutPath {
            sum: upper.sum + lower.sum,
            on_path: lower.on_path.min(upper.on_path.saturating_add(lower.sum)),
            off_path: upper.off_path.min(lower.off_path),
        }
    }
    fn rake(left: &Self::Point, right: &Self::Point) -> Self::Point {
        (left.0 + right.0, left.1.min(right.1))
    }
    fn add_edge(path: &CutPath) -> Self::Point {
        (path.sum, path.on_path.min(path.off_path))
    }
    fn add_vertex(point: &Self::Point, &(cut, star): &Self::Vertex, _: Option<&()>) -> CutPath {
        let sum = point.0 + u128::from(star);
        CutPath {
            sum,
            on_path: cut.saturating_add(sum),
            off_path: point.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        Xorshift,
        testutil::{exhaustive_sequences, sequence_cases},
    };

    #[test]
    fn minimum_cut() {
        let mut rng = Xorshift::default();
        let mut updates = Xorshift::default();
        for n in 1..=8 {
            let pairs: Vec<_> = (0..n).flat_map(|u| (0..u).map(move |v| (u, v))).collect();
            let binary = (n <= 5)
                .then(|| exhaustive_sequences([0, 1], pairs.len()..=pairs.len()))
                .into_iter()
                .flatten();
            let weighted = sequence_cases(&mut rng, 0..=u64::MAX, pairs.len()..=pairs.len());
            for weights in binary.chain(weighted) {
                let edges: Vec<_> = pairs
                    .iter()
                    .zip(weights)
                    .flat_map(|(&(u, v), w)| [(u, v, w); 2])
                    .chain([(0, 0, u64::MAX)])
                    .collect();
                let fixed: Vec<u128> = (0usize..1 << n)
                    .map(|side| {
                        edges
                            .iter()
                            .filter(|&&(a, b, _)| ((side >> a) ^ (side >> b)) & 1 != 0)
                            .map(|e| u128::from(e.2))
                            .sum()
                    })
                    .collect();
                let oracle = |star: &[u64]| {
                    (1usize..1 << n)
                        .map(|side| {
                            fixed[side]
                                + star
                                    .iter()
                                    .enumerate()
                                    .filter(|&(i, _)| side >> i & 1 != 0)
                                    .map(|(_, &w)| u128::from(w))
                                    .sum::<u128>()
                        })
                        .min()
                        .unwrap()
                };
                let mut star: Vec<u64> = (1..=n as u64).collect();
                let mut graph = DynamicStarMinimumCut::new(star.clone(), edges.iter().copied());
                assert_eq!(graph.minimum_cut(), oracle(&star));
                let sweeps = [0, u64::MAX, 1, 0]
                    .into_iter()
                    .flat_map(|w| (0..n).map(move |v| (v, w)));
                let mixed = (0..2 * n).map(|_| (updates.rand(n as u64) as usize, updates.rand64()));
                for (v, w) in sweeps.chain(mixed) {
                    star[v] = w;
                    graph.set_weight(v, w);
                    assert_eq!(graph.minimum_cut(), oracle(&star));
                }
            }
        }
    }
}
