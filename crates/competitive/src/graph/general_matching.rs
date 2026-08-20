use std::mem::swap;

#[derive(Debug, Clone)]
pub struct GeneralMatching {
    size: usize,
    graph: Vec<Vec<usize>>,
    mate: Vec<usize>,
    matching_size: usize,
    parent: Vec<usize>,
    base: Vec<usize>,
    state: Vec<u8>,
    seen: Vec<usize>,
    queue: Vec<usize>,
    timestamp: usize,
}

impl GeneralMatching {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            graph: vec![vec![]; size],
            mate: vec![size; size],
            matching_size: 0,
            parent: vec![size; size + 1],
            base: (0..=size).collect(),
            state: vec![Self::UNSEEN; size],
            seen: vec![0; size],
            queue: Vec::with_capacity(size),
            timestamp: 0,
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
            if u != self.size && v < u {
                res.push((v, u));
            }
        }
        res
    }
    fn compute(&mut self) {
        if self.matching_size != !0 {
            return;
        }
        self.matching_size = self.mate.iter().filter(|&&mate| mate != self.size).count() / 2;

        for v in 0..self.size {
            if self.mate[v] == self.size && self.augment_from(v) {
                self.matching_size += 1;
            }
        }
    }

    const UNSEEN: u8 = 0;
    const OUTER: u8 = 1;
    const INNER: u8 = 2;

    fn find(&mut self, mut v: usize) -> usize {
        while self.base[v] != v {
            let parent = self.base[v];
            self.base[v] = self.base[parent];
            v = self.base[v];
        }
        v
    }

    fn lca(&mut self, mut u: usize, mut v: usize) -> usize {
        self.timestamp += 1;
        u = self.find(u);
        v = self.find(v);
        loop {
            if u != self.size {
                if self.seen[u] == self.timestamp {
                    return u;
                }
                self.seen[u] = self.timestamp;
                u = self.find(self.parent[self.mate[u]]);
            }
            swap(&mut u, &mut v);
        }
    }

    fn contract(&mut self, mut v: usize, mut child: usize, ancestor: usize) {
        while self.find(v) != ancestor {
            self.parent[v] = child;
            child = self.mate[v];
            if self.state[child] == Self::INNER {
                self.state[child] = Self::OUTER;
                self.queue.push(child);
            }
            if self.base[v] == v {
                self.base[v] = ancestor;
            }
            if self.base[child] == child {
                self.base[child] = ancestor;
            }
            v = self.parent[child];
        }
    }

    fn augment_from(&mut self, root: usize) -> bool {
        for (v, base) in self.base.iter_mut().enumerate() {
            *base = v;
        }
        self.state.fill(Self::UNSEEN);
        self.queue.clear();
        self.state[root] = Self::OUTER;
        self.queue.push(root);
        let mut head = 0;
        while head < self.queue.len() {
            let u = self.queue[head];
            head += 1;
            for edge in 0..self.graph[u].len() {
                let v = self.graph[u][edge];
                if self.state[v] == Self::UNSEEN {
                    self.parent[v] = u;
                    self.state[v] = Self::INNER;
                    if self.mate[v] == self.size {
                        let mut v = v;
                        let mut u = u;
                        while u != self.size {
                            let next = self.mate[u];
                            self.mate[u] = v;
                            self.mate[v] = u;
                            v = next;
                            u = self.parent[v];
                        }
                        return true;
                    }
                    let v = self.mate[v];
                    self.state[v] = Self::OUTER;
                    self.queue.push(v);
                } else if self.state[v] == Self::OUTER && self.find(u) != self.find(v) {
                    let ancestor = self.lca(u, v);
                    self.contract(u, v, ancestor);
                    self.contract(v, u, ancestor);
                }
            }
        }
        false
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
            rand!(rng, split: 0..=edges.len());
            let mut gm = GeneralMatching::from_edges(n, &edges[..split]);
            assert_eq!(
                gm.maximum_matching().len(),
                brute_maximum_matching(n, &edges[..split])
            );
            for &(u, v) in &edges[split..] {
                gm.add_edge(u, v);
            }
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
