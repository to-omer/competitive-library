use super::{FibHashMap, UnionFind};
use std::{cmp::Reverse, collections::BinaryHeap};

/// Builds the inclusion tree of extreme sets of a nonempty undirected graph.
/// A set is extreme if every proper nonempty subset has a strictly larger boundary.
/// Leaves 0..n represent single vertices. The root (all vertices) is always included.
/// Returns each nonroot node's parent and each node's boundary weight. Parents follow children.
pub fn extreme_set_tree(
    n: usize,
    edges: impl IntoIterator<Item = (usize, usize, u64)>,
) -> (Vec<usize>, Vec<u128>) {
    assert!(n > 0);
    let mut components = UnionFind::new(n);
    let mut weights = FibHashMap::default();
    let mut degree = vec![0; n];
    for (u, v, w) in edges {
        if u != v && w != 0 {
            let w = u128::from(w);
            *weights.entry((u.min(v), u.max(v))).or_default() += w;
            degree[u] += w;
            degree[v] += w;
            components.unite(u, v);
        }
    }
    let mut members = vec![Vec::new(); n];
    let mut local = vec![0; n];
    for (v, index) in local.iter_mut().enumerate() {
        let group = &mut members[components.find_root(v)];
        *index = group.len();
        group.push(v);
    }
    let mut groups = vec![Vec::new(); n];
    for ((u, v), w) in weights {
        groups[components.find_root(u)].push((local[u], local[v], w));
    }
    let mut tree = ExtremeSetTreeBuilder {
        parent: vec![0; n],
        min_boundary: degree.clone(),
        boundary: degree,
        retained: vec![true; n],
    };
    let mut roots = Vec::new();
    for (vertices, edges) in members.into_iter().zip(groups) {
        if vertices.is_empty() {
            continue;
        }
        let root = if edges.len() + 1 == vertices.len() {
            tree.contract_tree(vertices, edges)
        } else {
            tree.contract_graph(vertices, edges)
        };
        roots.push(root);
    }
    tree.finish(roots)
}

struct ExtremeSetTreeBuilder {
    parent: Vec<usize>,
    boundary: Vec<u128>,
    min_boundary: Vec<u128>,
    retained: Vec<bool>,
}

impl ExtremeSetTreeBuilder {
    fn join(&mut self, a: usize, b: usize, weight: u128) -> usize {
        let v = self.boundary.len();
        self.parent[a] = v;
        self.parent[b] = v;
        self.parent.push(0);
        let cut = (self.boundary[a] - weight) + (self.boundary[b] - weight);
        let below = self.min_boundary[a].min(self.min_boundary[b]);
        self.boundary.push(cut);
        self.min_boundary.push(cut.min(below));
        self.retained.push(cut < below);
        v
    }

    fn contract_tree(
        &mut self,
        mut node: Vec<usize>,
        mut edges: Vec<(usize, usize, u128)>,
    ) -> usize {
        // Every internal edge of an extreme subtree outweighs each of its boundary edges.
        edges.sort_unstable_by_key(|&(_, _, w)| Reverse(w));
        let mut components = UnionFind::new(node.len());
        for (u, v, w) in edges {
            let a = components.find_root(u);
            let b = components.find_root(v);
            let merged = self.join(node[a], node[b], w);
            components.unite(a, b);
            node[components.find_root(a)] = merged;
        }
        node[components.find_root(0)]
    }

    fn contract_graph(&mut self, mut node: Vec<usize>, edges: Vec<(usize, usize, u128)>) -> usize {
        let mut adj = vec![FibHashMap::default(); node.len()];
        for (u, v, w) in edges {
            adj[u].insert(v, w);
            adj[v].insert(u, w);
        }
        let mut pairs = Vec::new();
        for (u, row) in adj.iter().enumerate() {
            for (&v, &w) in row {
                if u < v && self.is_dominant_edge(node[u], node[v], w) {
                    pairs.push((u, v));
                }
            }
        }
        while let Some((u, v)) = pairs.pop() {
            if let Some(&w) = adj[u].get(&v)
                && self.is_dominant_edge(node[u], node[v], w)
            {
                let a = self.contract(&mut node, &mut adj, u, v);
                for (&b, &w) in &adj[a] {
                    if self.is_dominant_edge(node[a], node[b], w) {
                        pairs.push((a, b));
                    }
                }
            }
        }
        let mut order = MinimumDegreeOrder::new(&node, &self.boundary, &adj);
        while order.vertices.len() > 1 {
            let a = order.vertices.pop().unwrap();
            let b = order.vertices.pop().unwrap();
            let a = self.contract(&mut node, &mut adj, a, b);
            order.push_merged(a, self.boundary[node[a]], &node, &adj);
        }
        node[order.vertices[0]]
    }

    fn is_dominant_edge(&self, a: usize, b: usize, w: u128) -> bool {
        // Deleting the separated endpoint from a nonsingleton cut cannot increase its boundary.
        w >= self.boundary[a].max(self.boundary[b]) - w
    }

    fn contract(
        &mut self,
        node: &mut [usize],
        adj: &mut [FibHashMap<usize, u128>],
        mut a: usize,
        mut b: usize,
    ) -> usize {
        if adj[a].len() < adj[b].len() {
            std::mem::swap(&mut a, &mut b);
        }
        let internal = adj[a].remove(&b).unwrap_or(0);
        adj[b].remove(&a);
        node[a] = self.join(node[a], node[b], internal);
        node[b] = usize::MAX;
        for (v, w) in std::mem::take(&mut adj[b]) {
            adj[v].remove(&b);
            *adj[a].entry(v).or_default() += w;
            *adj[v].entry(a).or_default() += w;
        }
        a
    }

    fn finish(mut self, roots: Vec<usize>) -> (Vec<usize>, Vec<u128>) {
        if roots.len() == 1 {
            self.parent.pop();
        } else {
            let root = self.boundary.len();
            for v in roots {
                self.parent[v] = root;
            }
            self.boundary.push(0);
            self.retained.push(true);
        }
        let Self {
            mut parent,
            mut boundary,
            retained,
            ..
        } = self;
        let mut next = 0;
        let mut id: Vec<_> = retained
            .iter()
            .map(|&keep| {
                let i = next;
                next += usize::from(keep);
                i
            })
            .collect();
        for (v, p) in parent.iter_mut().enumerate().rev() {
            *p = id[*p];
            if !retained[v] {
                id[v] = *p;
            }
        }
        let mut keep = retained.iter();
        parent.retain(|_| *keep.next().unwrap());
        let mut keep = retained.into_iter();
        boundary.retain(|_| keep.next().unwrap());
        (parent, boundary)
    }
}

const BLOCK_SIZE: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PeelState {
    Waiting,
    Active,
    Removed,
}

struct MinimumDegreeOrder {
    vertices: Vec<usize>,
    removal_degree: Vec<u128>,
    position: Vec<usize>,
    residual: Vec<u128>,
    heap: BinaryHeap<Reverse<(u128, usize, usize)>>,
    active_neighbors: Vec<Vec<(usize, u128)>>,
    state: Vec<PeelState>,
    merged_weight: Vec<u128>,
    suffix: Vec<usize>,
    block_max: Vec<u128>,
    block_weight: Vec<u128>,
}

impl MinimumDegreeOrder {
    fn new(node: &[usize], boundary: &[u128], adj: &[FibHashMap<usize, u128>]) -> Self {
        let n = node.len();
        let mut order: Vec<_> = (0..n).filter(|&v| node[v] != usize::MAX).collect();
        let mut removal_degree = vec![0; n];
        let mut residual = vec![0; n];
        for &v in &order {
            residual[v] = boundary[node[v]];
        }
        let mut heap: BinaryHeap<_> = order
            .iter()
            .map(|&v| Reverse((residual[v], node[v], v)))
            .collect();
        let mut position = vec![usize::MAX; n];
        for (i, slot) in order.iter_mut().enumerate() {
            let (d, v) = loop {
                let Reverse((d, _, v)) = heap.pop().unwrap();
                if position[v] == usize::MAX && d == residual[v] {
                    break (d, v);
                }
            };
            removal_degree[v] = d;
            *slot = v;
            position[v] = i;
            for (&u, &w) in &adj[v] {
                if position[u] == usize::MAX {
                    residual[u] -= w;
                    heap.push(Reverse((residual[u], node[u], u)));
                }
            }
        }
        heap.clear();
        let block_max: Vec<_> = order
            .chunks(BLOCK_SIZE)
            .map(|chunk| chunk.iter().map(|&v| removal_degree[v]).max().unwrap())
            .collect();
        let block_weight = vec![0; block_max.len()];

        Self {
            vertices: order,
            removal_degree,
            position,
            residual,
            heap,
            active_neighbors: vec![Vec::new(); n],
            state: vec![PeelState::Waiting; n],
            merged_weight: vec![0; n],
            suffix: Vec::new(),
            block_max,
            block_weight,
        }
    }

    fn push_merged(
        &mut self,
        a: usize,
        degree: u128,
        node: &[usize],
        adj: &[FibHashMap<usize, u128>],
    ) {
        for (&u, &w) in &adj[a] {
            self.merged_weight[u] = w;
            self.block_weight[self.position[u] / BLOCK_SIZE] += w;
        }
        let mut d = degree;
        let mut first = None;
        for (block, chunk) in self.vertices.chunks(BLOCK_SIZE).enumerate() {
            // The final degree in a block bounds all earlier degrees from below.
            if d - self.block_weight[block] >= self.block_max[block] {
                d -= self.block_weight[block];
                continue;
            }
            if let Some(i) = chunk.iter().position(|&v| {
                if d < self.removal_degree[v] {
                    return true;
                }
                d -= self.merged_weight[v];
                false
            }) {
                first = Some(block * BLOCK_SIZE + i);
                break;
            }
        }
        for &u in adj[a].keys() {
            self.merged_weight[u] = 0;
            self.block_weight[self.position[u] / BLOCK_SIZE] = 0;
        }
        if let Some(first) = first {
            self.repair(first, a, d, node, adj);
        } else {
            self.removal_degree[a] = 0;
            self.position[a] = self.vertices.len();
            self.vertices.push(a);
        }
    }

    fn repair(
        &mut self,
        first: usize,
        a: usize,
        mut d: u128,
        node: &[usize],
        adj: &[FibHashMap<usize, u128>],
    ) {
        self.suffix.clear();
        self.suffix.extend_from_slice(&self.vertices[first..]);
        self.vertices.truncate(first);
        let mut cursor = 0;
        let mut v = a;
        loop {
            let early = self.suffix.get(cursor) != Some(&v);
            self.removal_degree[v] = d;
            self.vertices.push(v);
            self.state[v] = PeelState::Removed;
            for &(u, w) in &self.active_neighbors[v] {
                if self.state[u] != PeelState::Removed {
                    self.residual[u] -= w;
                    self.heap.push(Reverse((self.residual[u], node[u], u)));
                }
            }
            if early {
                // Only neighbors of an early removal can outrank the next vertex in the old order.
                for &u in adj[v].keys() {
                    if self.position[u] < first || self.state[u] != PeelState::Waiting {
                        continue;
                    }
                    self.state[u] = PeelState::Active;
                    let mut residual = 0;
                    for (&x, &w) in &adj[u] {
                        if self.position[x] >= first && self.state[x] != PeelState::Removed {
                            residual += w;
                            self.active_neighbors[x].push((u, w));
                        }
                    }
                    self.residual[u] = residual;
                    self.heap.push(Reverse((residual, node[u], u)));
                }
            }
            while cursor < self.suffix.len()
                && self.state[self.suffix[cursor]] == PeelState::Removed
            {
                cursor += 1;
            }
            let Some(&head) = self.suffix.get(cursor) else {
                break;
            };
            while let Some(&Reverse((k, _, x))) = self.heap.peek() {
                if self.state[x] != PeelState::Removed && self.residual[x] == k {
                    break;
                }
                self.heap.pop();
            }
            let Some(&Reverse(best)) = self.heap.peek() else {
                self.vertices.extend(
                    self.suffix[cursor..]
                        .iter()
                        .copied()
                        .filter(|&u| self.state[u] != PeelState::Removed),
                );
                break;
            };
            let head_key = if self.state[head] == PeelState::Active {
                self.residual[head]
            } else {
                self.removal_degree[head]
            };
            (d, _, v) = best.min((head_key, node[head], head));
        }
        self.heap.clear();
        for (i, &v) in self.vertices[first..].iter().enumerate() {
            self.position[v] = first + i;
            self.active_neighbors[v].clear();
            self.state[v] = PeelState::Waiting;
        }
        let first_block = first / BLOCK_SIZE;
        for (maximum, chunk) in self.block_max[first_block..]
            .iter_mut()
            .zip(self.vertices[first_block * BLOCK_SIZE..].chunks(BLOCK_SIZE))
        {
            *maximum = chunk.iter().map(|&v| self.removal_degree[v]).max().unwrap();
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
    fn extreme_sets() {
        let mut rng = Xorshift::default();
        for n in 1..=9 {
            let pairs: Vec<_> = (0..n).flat_map(|u| (0..u).map(move |v| (u, v))).collect();
            let len = pairs.len()..=pairs.len();
            let exhaustive = (n <= 4)
                .then(|| exhaustive_sequences(0..=6u64, len.clone()))
                .into_iter()
                .flatten();
            let structured: Vec<_> = sequence_cases(&mut rng, 0..=7u64, len).collect();
            let trials = if n >= 5 { 160 } else { 0 };
            let sampled = (0..trials).map(|trial| {
                (0..pairs.len())
                    .map(|_| rng.rand(12).saturating_sub(trial % 10))
                    .collect()
            });
            for weights in exhaustive.chain(structured).chain(sampled) {
                let edges: Vec<_> = pairs
                    .iter()
                    .zip(weights)
                    .map(|(&(u, v), w)| (u, v, w))
                    .collect();
                let (parent, boundary) = extreme_set_tree(n, edges.iter().copied());
                let mut sets = vec![0usize; boundary.len()];
                for (v, s) in sets.iter_mut().take(n).enumerate() {
                    *s = 1 << v;
                }
                for (v, p) in parent.into_iter().enumerate() {
                    assert!(v < p);
                    sets[p] |= sets[v];
                }
                let cuts: Vec<u128> = (0..1usize << n)
                    .map(|s| {
                        edges
                            .iter()
                            .filter(|&&(u, v, _)| ((s >> u) ^ (s >> v)) & 1 != 0)
                            .map(|e| u128::from(e.2))
                            .sum()
                    })
                    .collect();
                for (&s, d) in sets.iter().zip(boundary) {
                    assert_eq!(d, cuts[s]);
                }
                let mut minimum = cuts.clone();
                minimum[0] = u128::MAX;
                let mut expected = Vec::new();
                for s in 1..cuts.len() {
                    let below = (0..n)
                        .filter(|&v| s >> v & 1 != 0)
                        .map(|v| minimum[s ^ (1 << v)])
                        .min()
                        .unwrap();
                    if cuts[s] < below || s + 1 == cuts.len() {
                        expected.push(s);
                    }
                    minimum[s] = cuts[s].min(below);
                }
                sets.sort_unstable();
                assert_eq!(sets, expected);
            }
        }
        for (groups, size, connected) in [
            (33, 3, false),
            (7, 4, true),
            (8, 4, true),
            (9, 4, true),
            (33, 4, true),
        ] {
            let n = groups * size;
            for interleaved in [false, true] {
                let vertex = |i, j| {
                    if interleaved {
                        i + j * groups
                    } else {
                        size * i + j
                    }
                };
                let mut edges = Vec::new();
                for i in 0..groups {
                    for j in 0..size {
                        for k in 0..j {
                            edges.push((vertex(i, j), vertex(i, k), 4));
                        }
                    }
                    if connected && i > 0 {
                        edges.push((vertex(i - 1, 0), vertex(i, 0), 1));
                    }
                }
                let (parent, boundary) = extreme_set_tree(n, edges);
                let root = n + groups;
                assert_eq!(parent.len(), root);
                assert_eq!(boundary[root], 0);
                for i in 0..groups {
                    let group = parent[vertex(i, 0)];
                    let external = if connected {
                        u128::from(i > 0) + u128::from(i + 1 < groups)
                    } else {
                        0
                    };
                    assert_eq!(parent[group], root);
                    assert_eq!(boundary[group], external);
                    for j in 0..size {
                        let v = vertex(i, j);
                        assert_eq!(parent[v], group);
                        assert_eq!(
                            boundary[v],
                            4 * (size as u128 - 1) + if j == 0 { external } else { 0 }
                        );
                    }
                }
                let mut unique: Vec<_> = (0..groups).map(|i| parent[vertex(i, 0)]).collect();
                unique.sort_unstable();
                assert!(unique.into_iter().eq(n..root));
            }
        }
    }
}
