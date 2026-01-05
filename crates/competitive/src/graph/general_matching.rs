use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct GeneralMatching {
    size: usize,
    graph: Vec<Vec<usize>>,
    mate: Vec<usize>,
    matching_size: usize,
}

impl GeneralMatching {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            graph: vec![vec![]; size],
            mate: vec![!0; size],
            matching_size: 0,
        }
    }
    pub fn add_edge(&mut self, u: usize, v: usize) {
        assert!(u < self.size);
        assert!(v < self.size);
        if u == v {
            return;
        }
        self.graph[u].push(v);
        self.graph[v].push(u);
        self.matching_size = !0;
    }
    pub fn from_edges(size: usize, edges: &[(usize, usize)]) -> Self {
        let mut this = Self::new(size);
        for &(u, v) in edges {
            this.add_edge(u, v);
        }
        this
    }
    pub fn maximum_matching(&mut self) -> Vec<(usize, usize)> {
        self.compute();
        let mut res = Vec::with_capacity(self.matching_size);
        for v in 0..self.size {
            let u = self.mate[v];
            if u != !0 && v < u {
                res.push((v, u));
            }
        }
        res
    }
    fn compute(&mut self) {
        if self.matching_size != !0 {
            return;
        }
        let mut cnt = 0usize;
        for &m in &self.mate {
            if m != !0 {
                cnt += 1;
            }
        }
        self.matching_size = cnt / 2;

        let mut p = vec![!0; self.size];
        for v in 0..self.size {
            if self.mate[v] == !0 {
                let finish = self.find_path(v, &mut p);
                if finish != !0 {
                    self.augment(finish, &p);
                    self.matching_size += 1;
                }
            }
        }
    }
    fn augment(&mut self, mut v: usize, p: &[usize]) {
        while v != !0 {
            let pv = p[v];
            let nv = if pv != !0 { self.mate[pv] } else { !0 };
            self.mate[v] = pv;
            if pv != !0 {
                self.mate[pv] = v;
            }
            v = nv;
        }
    }
    fn find_path(&self, root: usize, p: &mut [usize]) -> usize {
        let n = self.size;
        p.fill(!0);
        let mut used = vec![false; n];
        let mut base: Vec<usize> = (0..n).collect();
        let mut q = VecDeque::with_capacity(n);
        let mut lca_vis = vec![0usize; n];
        let mut lca_iter = 0usize;

        used[root] = true;
        q.push_back(root);
        while let Some(v) = q.pop_front() {
            for &u in &self.graph[v] {
                if base[v] == base[u] || self.mate[v] == u {
                    continue;
                }
                if u == root || (self.mate[u] != !0 && p[self.mate[u]] != !0) {
                    let cur = {
                        let mut a = v;
                        let mut b = u;
                        lca_iter += 1;
                        let iter = lca_iter;
                        loop {
                            a = base[a];
                            lca_vis[a] = iter;
                            if self.mate[a] == !0 {
                                break;
                            }
                            a = p[self.mate[a]];
                        }
                        loop {
                            b = base[b];
                            if lca_vis[b] == iter {
                                break b;
                            }
                            if self.mate[b] == !0 {
                                break b;
                            }
                            b = p[self.mate[b]];
                        }
                    };
                    let mut blossom = vec![false; n];
                    mark_path(&self.mate, &base, p, &mut blossom, v, cur, u);
                    mark_path(&self.mate, &base, p, &mut blossom, u, cur, v);
                    for i in 0..n {
                        if blossom[base[i]] {
                            base[i] = cur;
                            if !used[i] {
                                used[i] = true;
                                q.push_back(i);
                            }
                        }
                    }
                } else if p[u] == !0 {
                    p[u] = v;
                    if self.mate[u] == !0 {
                        return u;
                    }
                    let m = self.mate[u];
                    used[m] = true;
                    q.push_back(m);
                }
            }
        }
        !0
    }
}

fn mark_path(
    mate: &[usize],
    base: &[usize],
    p: &mut [usize],
    blossom: &mut [bool],
    mut v: usize,
    b: usize,
    mut child: usize,
) {
    while base[v] != b {
        blossom[base[v]] = true;
        blossom[base[mate[v]]] = true;
        p[v] = child;
        child = mate[v];
        v = p[mate[v]];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    fn brute_maximum_matching(n: usize, edges: &[(usize, usize)]) -> usize {
        let mut adj = vec![vec![false; n]; n];
        for &(u, v) in edges {
            adj[u][v] = true;
            adj[v][u] = true;
        }
        let mut dp = vec![0usize; 1 << n];
        for mask in 1usize..1 << n {
            let i = mask.trailing_zeros() as usize;
            let mask_without_i = mask & !(1 << i);
            let mut best = dp[mask_without_i];
            let mut m = mask_without_i;
            while m != 0 {
                let j = m.trailing_zeros() as usize;
                if adj[i][j] {
                    let val = 1 + dp[mask_without_i & !(1 << j)];
                    if val > best {
                        best = val;
                    }
                }
                m &= m - 1;
            }
            dp[mask] = best;
        }
        dp[(1 << n) - 1]
    }

    #[test]
    fn test_general_matching() {
        const Q: usize = 200;
        const N: usize = 10;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N);
            let mut edges = vec![];
            for i in 0..n {
                for j in i + 1..n {
                    rand!(rng, b: 0..2usize);
                    if b == 1 {
                        edges.push((i, j));
                    }
                }
            }
            let mut gm = GeneralMatching::from_edges(n, &edges);
            let matching = gm.maximum_matching();
            let mut used = vec![false; n];
            let mut adj = vec![vec![false; n]; n];
            for &(u, v) in &edges {
                adj[u][v] = true;
                adj[v][u] = true;
            }
            for &(u, v) in &matching {
                assert!(u < v);
                assert!(adj[u][v]);
                assert!(!used[u]);
                assert!(!used[v]);
                used[u] = true;
                used[v] = true;
            }
            assert_eq!(matching.len(), brute_maximum_matching(n, &edges));
        }
    }
}
