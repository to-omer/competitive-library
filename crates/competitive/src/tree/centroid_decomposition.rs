use super::{ConvolveSteps, U64Convolve, UndirectedSparseGraph};
use std::mem::swap;

fn centroid_decomposition_dfs(
    parents: &[usize],
    vs: &[usize],
    f: &mut impl FnMut(&[usize], &[usize], usize, usize),
) {
    let n = vs.len();
    assert!(n >= 2);
    if n == 2 {
        return;
    }
    let mut size = vec![1; n];
    let mut c = !0;
    for i in (0..n).rev() {
        if size[i] >= n.div_ceil(2) {
            c = i;
            break;
        }
        size[parents[i]] += size[i];
    }
    let mut color = vec![!0u8; n];
    let mut order = vec![!0usize; n];
    order[c] = 0;
    let mut count = 1usize;
    let mut sum_size = 0usize;
    for u in 1..n {
        if parents[u] == c && sum_size + size[u] <= (n - 1) / 2 {
            sum_size += size[u];
            color[u] = 0;
            order[u] = count;
            count += 1;
        }
    }
    for u in 1..n {
        if color[parents[u]] == 0 {
            color[u] = 0;
            order[u] = count;
            count += 1;
        }
    }
    let lsize = count - 1;
    {
        let mut u = parents[c];
        while u != !0 {
            color[u] = 1;
            order[u] = count;
            count += 1;
            u = parents[u];
        }
    }
    for u in 0..n {
        if u != c && color[u] == !0 {
            color[u] = 1;
            order[u] = count;
            count += 1;
        }
    }
    assert_eq!(count, n);
    let rsize = n - lsize - 1;
    let mut cparents = vec![!0usize; n];
    let mut cvs = vec![!0usize; n];
    let mut lparents = vec![!0usize; lsize + 1];
    let mut lvs = vec![!0usize; lsize + 1];
    let mut rparents = vec![!0usize; rsize + 1];
    let mut rvs = vec![!0usize; rsize + 1];
    for u in 0..n {
        let i = order[u];
        cvs[i] = vs[u];
        if color[u] != 1 {
            lvs[i] = vs[u];
        }
        if color[u] != 0 {
            rvs[if i == 0 { 0 } else { i - lsize }] = vs[u];
        }
    }
    for u in 1..n {
        let mut x = order[u];
        let mut y = order[parents[u]];
        if x > y {
            swap(&mut x, &mut y);
        }
        cparents[y] = x;
        if color[u] != 1 && color[parents[u]] != 1 {
            lparents[y] = x;
        }
        if color[u] != 0 && color[parents[u]] != 0 {
            rparents[if y == 0 { 0 } else { y - lsize }] = if x == 0 { 0 } else { x - lsize };
        }
    }
    f(&cparents, &cvs, lsize, rsize);
    centroid_decomposition_dfs(&lparents, &lvs, f);
    centroid_decomposition_dfs(&rparents, &rvs, f);
}

impl UndirectedSparseGraph {
    /// 1/3 centroid decomposition
    ///
    /// - f: (parents: &[usize], vs: &[usize], lsize: usize, rsize: usize)
    /// - 0: root, 1..=lsize: left subtree, lsize+1..=lsize+rsize: right subtree
    pub fn centroid_decomposition(&self, mut f: impl FnMut(&[usize], &[usize], usize, usize)) {
        let n = self.vertices_size();
        if n <= 1 {
            return;
        }
        let mut vs = Vec::with_capacity(n);
        let mut parents = vec![!0; n];
        vs.push(0usize);
        for l in 0..n {
            let u = vs[l];
            for a in self.adjacencies(u) {
                if a.to != parents[u] {
                    vs.push(a.to);
                    parents[a.to] = u;
                }
            }
        }
        let mut new_idx = vec![0; n];
        for i in 0..n {
            new_idx[vs[i]] = i;
        }
        let mut new_parent = vec![!0; n];
        for i in 1..n {
            new_parent[new_idx[i]] = new_idx[parents[i]];
        }
        centroid_decomposition_dfs(&new_parent, &vs, &mut f);
    }

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
