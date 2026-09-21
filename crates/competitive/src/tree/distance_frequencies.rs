use super::{ConvolveSteps, U64Convolve, UndirectedSparseGraph};

impl UndirectedSparseGraph {
    pub fn distance_frequencies(&self) -> Vec<u64> {
        let n = self.vertices_size();
        let mut table = vec![0u64; n];
        if n == 0 {
            return table;
        }
        table[0] = n as u64;
        if n == 1 {
            return table;
        }
        table[1] = (n * 2 - 2) as u64;
        self.centroid_decomposition(|parents, vs, lsize, _rsize| {
            let n = vs.len();
            let mut dist = vec![0usize; n];
            for i in 1..n {
                dist[i] = dist[parents[i]] + 1;
            }
            let d_max = dist.iter().max().cloned().unwrap_or_default();
            let mut f = vec![0u64; d_max + 1];
            let mut g = vec![0u64; d_max + 1];
            for i in 1..=lsize {
                f[dist[i]] += 1;
            }
            for i in lsize + 1..n {
                g[dist[i]] += 1;
            }
            while f.last().is_some_and(|&x| x == 0) {
                f.pop();
            }
            while g.last().is_some_and(|&x| x == 0) {
                g.pop();
            }
            let h = U64Convolve::convolve(f, g);
            for (i, &x) in h.iter().enumerate() {
                table[i] += x * 2;
            }
        });
        table
    }
}

#[cfg(test)]
mod tests {
    use crate::{tools::Xorshift, tree::MixedTree};
    #[test]
    fn test_distance_frequencies() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let g = rng.random(MixedTree(1usize..100));
            let n = g.vertices_size();
            let result = g.distance_frequencies();
            let mut expected = vec![0u64; n];
            for u in 0..n {
                let depth = g.tree_depth(u);
                for v in 0..n {
                    expected[depth[v] as usize] += 1;
                }
            }
            assert_eq!(result, expected);
        }
    }
}
