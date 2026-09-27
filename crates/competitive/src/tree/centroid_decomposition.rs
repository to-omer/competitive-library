use super::{Graph, UndirectedSparseGraph};
use std::mem::swap;

#[derive(Debug, Clone)]
struct RootedTree {
    parents: Vec<usize>,
    vs: Vec<usize>,
}

impl RootedTree {
    fn len(&self) -> usize {
        self.vs.len()
    }

    fn split_centroid(self) -> CentroidSplit {
        let n = self.len();
        assert!(n > 2);
        let parents = &self.parents;
        let vs = &self.vs;
        let mut size = vec![1; n];
        let mut c = usize::MAX;
        for i in (0..n).rev() {
            if size[i] >= n.div_ceil(2) {
                c = i;
                break;
            }
            size[parents[i]] += size[i];
        }
        let mut side = vec![u8::MAX; n];
        let mut order = vec![usize::MAX; n];
        order[c] = 0;
        let mut count = 1usize;
        let mut taken = 0usize;
        for u in 1..n {
            if parents[u] == c && taken + size[u] <= (n - 1) / 2 {
                taken += size[u];
                side[u] = 0;
                order[u] = count;
                count += 1;
            }
        }
        for u in 1..n {
            if side[parents[u]] == 0 {
                side[u] = 0;
                order[u] = count;
                count += 1;
            }
        }
        let lsize = count - 1;
        {
            let mut u = parents[c];
            while u != usize::MAX {
                side[u] = 1;
                order[u] = count;
                count += 1;
                u = parents[u];
            }
        }
        for u in 0..n {
            if u != c && side[u] == u8::MAX {
                side[u] = 1;
                order[u] = count;
                count += 1;
            }
        }
        assert_eq!(count, n);
        let mut whole_parents = vec![usize::MAX; n];
        let mut whole_vs = vec![usize::MAX; n];
        for u in 0..n {
            whole_vs[order[u]] = vs[u];
        }
        for u in 1..n {
            let mut x = order[u];
            let mut y = order[parents[u]];
            if x > y {
                swap(&mut x, &mut y);
            }
            whole_parents[y] = x;
        }
        let left = RootedTree {
            parents: whole_parents[..=lsize].to_vec(),
            vs: whole_vs[..=lsize].to_vec(),
        };
        let right = RootedTree {
            parents: std::iter::once(usize::MAX)
                .chain(
                    whole_parents[lsize + 1..]
                        .iter()
                        .map(|&p| if p == 0 { 0 } else { p - lsize }),
                )
                .collect(),
            vs: std::iter::once(whole_vs[0])
                .chain(whole_vs[lsize + 1..].iter().copied())
                .collect(),
        };
        CentroidSplit {
            whole: RootedTree {
                parents: whole_parents,
                vs: whole_vs,
            },
            left,
            right,
            lsize,
        }
    }

    fn centroid_decomposition(self, f: &mut impl FnMut(&[usize], &[usize], usize, usize)) {
        if self.len() <= 2 {
            return;
        }
        let split = self.split_centroid();
        f(
            &split.whole.parents,
            &split.whole.vs,
            split.lsize,
            split.rsize(),
        );
        split.left.centroid_decomposition(f);
        split.right.centroid_decomposition(f);
    }
}

impl From<&UndirectedSparseGraph> for RootedTree {
    fn from(graph: &UndirectedSparseGraph) -> Self {
        let n = graph.vertices_size();
        let mut vs = Vec::with_capacity(n);
        let mut parent = vec![usize::MAX; n];
        vs.push(0usize);
        for i in 0..n {
            let u = vs[i];
            for a in graph.neighbors(u) {
                if a.to != parent[u] {
                    vs.push(a.to);
                    parent[a.to] = u;
                }
            }
        }
        let mut new_idx = vec![0; n];
        for (i, &v) in vs.iter().enumerate() {
            new_idx[v] = i;
        }
        let mut parents = vec![usize::MAX; n];
        for v in 1..n {
            parents[new_idx[v]] = new_idx[parent[v]];
        }
        Self { parents, vs }
    }
}

#[derive(Debug)]
struct CentroidSplit {
    whole: RootedTree,
    left: RootedTree,
    right: RootedTree,
    lsize: usize,
}

impl CentroidSplit {
    fn rsize(&self) -> usize {
        self.whole.len() - self.lsize - 1
    }
}

#[derive(Debug, Clone, Copy)]
struct ContourInfo {
    comp: u32,
    dep: u32,
}

#[derive(Debug, Clone)]
pub struct ContourQueryRange {
    comp_range: Vec<usize>,
    info_indptr: Vec<usize>,
    infos: Vec<ContourInfo>,
    local_info: Vec<(usize, usize)>,
    local_offsets: Vec<usize>,
    local_masks: Vec<u32>,
}

impl ContourQueryRange {
    pub fn len(&self) -> usize {
        self.comp_range.last().copied().unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn component_sizes(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.comp_range.windows(2).map(|range| range[1] - range[0])
    }

    /// Calls `f(component, index)` for each position representing `v`.
    pub fn for_each_index(&self, v: usize, mut f: impl FnMut(usize, usize)) {
        for info in &self.infos[self.info_indptr[v]..self.info_indptr[v + 1]] {
            f(info.comp as usize, info.dep as usize);
        }
        let (comp, index) = self.local_info[v];
        if comp != usize::MAX {
            f(
                self.comp_range.len() - 1 - self.local_offsets.len() + comp,
                index,
            );
        }
    }

    /// Calls `f(component, start, end)` for disjoint ranges at distances in `l..r` from `v`.
    /// The ranges exclude `v` itself, even when `l == 0`.
    pub fn for_each_contour_range(
        &self,
        v: usize,
        l: usize,
        r: usize,
        mut f: impl FnMut(usize, usize, usize),
    ) {
        for info in &self.infos[self.info_indptr[v]..self.info_indptr[v + 1]] {
            let comp = (info.comp ^ 1) as usize;
            let start = self.comp_range[comp];
            let len = self.comp_range[comp + 1] - start;
            let lo = l.saturating_sub(info.dep as usize).min(len);
            let hi = r.saturating_sub(info.dep as usize).min(len);
            if lo < hi {
                f(comp, lo, hi);
            }
        }
        let (local, index) = self.local_info[v];
        if local != usize::MAX {
            let comp = self.comp_range.len() - 1 - self.local_offsets.len() + local;
            let len = self.comp_range[comp + 1] - self.comp_range[comp];
            let lo = l.max(1).min(len);
            let hi = r.min(len);
            if lo < hi {
                let offset = self.local_offsets[local] + index * (len + 1);
                let mut mask = self.local_masks[offset + hi] ^ self.local_masks[offset + lo];
                while mask != 0 {
                    let start = mask.trailing_zeros();
                    let end = start + (mask >> start).trailing_ones();
                    f(comp, start as usize, end as usize);
                    mask &= mask.wrapping_add(1 << start);
                }
            }
        }
    }
}

impl UndirectedSparseGraph {
    /// 1/3 centroid decomposition
    ///
    /// - f: (parents: &[usize], vs: &[usize], lsize: usize, rsize: usize)
    /// - 0: root, 1..=lsize: left subtree, lsize+1..=lsize+rsize: right subtree
    pub fn centroid_decomposition(&self, mut f: impl FnMut(&[usize], &[usize], usize, usize)) {
        if self.vertices_size() <= 1 {
            return;
        }
        RootedTree::from(self).centroid_decomposition(&mut f);
    }

    pub fn contour_query_range(&self) -> ContourQueryRange {
        let n = self.vertices_size();
        assert!(n <= u32::MAX as usize / 2);
        if n <= 1 {
            return ContourQueryRange {
                comp_range: vec![0],
                info_indptr: vec![0; n + 1],
                infos: vec![],
                local_info: vec![(usize::MAX, 0); n],
                local_offsets: vec![],
                local_masks: vec![],
            };
        }
        let (vertices, graph) = {
            let (vertices, parents) = self.tree_order(0);
            let mut indices = vec![0; n];
            for (i, &v) in vertices.iter().enumerate() {
                indices[v] = i;
            }
            let edges = vertices
                .iter()
                .enumerate()
                .skip(1)
                .map(|(i, &v)| (i, indices[parents[v]]))
                .collect();
            let graph = UndirectedSparseGraph::from_edges(n, edges);
            (vertices, graph)
        };
        let mut comp_range = vec![0usize];
        let mut vertex_info = Vec::with_capacity(n * (n.ilog2() as usize + 1));
        let mut info_indptr = vec![0usize; n + 1];
        let mut local_info = vec![(usize::MAX, 0); n];
        let mut local_offsets = Vec::new();
        let mut local_masks = Vec::new();
        let mut distances = Vec::new();
        let mut local_sizes = Vec::new();
        let mut removed = vec![false; n];
        let mut parents = vec![usize::MAX; n];
        let mut sizes = vec![0usize; n];
        let mut tasks = vec![0];
        let mut order = Vec::with_capacity(n);
        let mut entries = Vec::with_capacity(n);
        let mut boundaries = Vec::new();
        let mut groups = Vec::new();
        while let Some(root) = tasks.pop() {
            order.clear();
            order.push(root);
            parents[root] = usize::MAX;
            let mut i = 0;
            while i < order.len() {
                let v = order[i];
                sizes[v] = 1;
                for edge in graph.neighbors(v) {
                    if !removed[edge.to] && edge.to != parents[v] {
                        parents[edge.to] = v;
                        order.push(edge.to);
                    }
                }
                i += 1;
            }
            if order.len() <= 32 {
                let len = order.len();
                if len > 1 {
                    let comp = local_offsets.len();
                    let offset = local_masks.len();
                    local_offsets.push(offset);
                    local_sizes.push(len);
                    local_masks.resize(offset + len * (len + 1), 0u32);
                    distances.clear();
                    distances.resize(len * len, 0u8);
                    for (i, &v) in order.iter().enumerate() {
                        local_info[vertices[v]] = (comp, i);
                        sizes[v] = i;
                        local_masks[offset + i * (len + 1) + 1] = 1 << i;
                    }
                    for (i, &v) in order.iter().enumerate().skip(1) {
                        let parent = sizes[parents[v]];
                        for j in 0..i {
                            let distance = distances[parent * len + j] + 1;
                            distances[i * len + j] = distance;
                            distances[j * len + i] = distance;
                            local_masks[offset + i * (len + 1) + distance as usize + 1] |= 1 << j;
                            local_masks[offset + j * (len + 1) + distance as usize + 1] |= 1 << i;
                        }
                    }
                    for row in local_masks[offset..].chunks_exact_mut(len + 1) {
                        for d in 1..=len {
                            row[d] |= row[d - 1];
                        }
                    }
                }
                continue;
            }
            let mut centroid = root;
            for &v in order.iter().rev() {
                if sizes[v] >= order.len().div_ceil(2) {
                    centroid = v;
                    break;
                }
                sizes[parents[v]] += sizes[v];
            }
            removed[centroid] = true;
            entries.clear();
            entries.push((centroid, 0));
            boundaries.clear();
            boundaries.extend([0, 1]);
            for edge in graph.neighbors(centroid) {
                let v = edge.to;
                if removed[v] {
                    continue;
                }
                tasks.push(v);
                parents[v] = centroid;
                let mut i = entries.len();
                entries.push((v, 1));
                while i < entries.len() {
                    let (v, distance) = entries[i];
                    for edge in graph.neighbors(v) {
                        if !removed[edge.to] && edge.to != parents[v] {
                            parents[edge.to] = v;
                            entries.push((edge.to, distance + 1));
                        }
                    }
                    i += 1;
                }
                boundaries.push(entries.len());
            }
            groups.push((0, boundaries.len() - 1));
            while let Some((first, last)) = groups.pop() {
                if last - first < 2 {
                    continue;
                }
                let weight = boundaries[last] - boundaries[first];
                let target = boundaries[first] + weight.div_ceil(2);
                let mut middle =
                    first + 1 + boundaries[first + 1..last].partition_point(|&p| p < target);
                middle = middle.min(last - 1);
                if middle > first + 1 {
                    let x = boundaries[middle] - boundaries[first];
                    let y = boundaries[middle - 1] - boundaries[first];
                    if y.max(weight - y) < x.max(weight - x) {
                        middle -= 1;
                    }
                }
                for (l, r) in [(first, middle), (middle, last)] {
                    let comp = comp_range.len() - 1;
                    let mut max_distance = 0;
                    for &(v, dep) in &entries[boundaries[l]..boundaries[r]] {
                        vertex_info.push((
                            vertices[v] as u32,
                            ContourInfo {
                                comp: comp as u32,
                                dep: dep as u32,
                            },
                        ));
                        info_indptr[vertices[v] + 1] += 1;
                        max_distance = max_distance.max(dep);
                    }
                    comp_range.push(comp_range[comp] + max_distance + 1);
                }
                groups.extend([(middle, last), (first, middle)]);
            }
        }
        for len in local_sizes {
            comp_range.push(comp_range.last().unwrap() + len);
        }
        for v in 1..=n {
            info_indptr[v] += info_indptr[v - 1];
        }
        let mut infos = vec![ContourInfo { comp: 0, dep: 0 }; vertex_info.len()];
        let mut positions = info_indptr.clone();
        for (v, info) in vertex_info {
            let v = v as usize;
            infos[positions[v]] = info;
            positions[v] += 1;
        }
        ContourQueryRange {
            comp_range,
            info_indptr,
            infos,
            local_info,
            local_offsets,
            local_masks,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::UndirectedSparseGraph,
        tools::{Xorshift, testutil::exhaustive_sequences},
        tree::{MixedTree, PathTree, StarTree},
    };

    #[test]
    fn test_contour_query_range() {
        let mut rng = Xorshift::default();
        let mut graphs = vec![UndirectedSparseGraph::from_edges(0, vec![])];
        for n in 1..=5 {
            for parents in exhaustive_sequences(0..n, n - 1..=n - 1) {
                if parents.iter().enumerate().all(|(i, &p)| p <= i) {
                    graphs.push(UndirectedSparseGraph::from_edges(
                        n,
                        parents
                            .into_iter()
                            .enumerate()
                            .map(|(i, p)| (p, i + 1))
                            .collect(),
                    ));
                }
            }
        }
        for n in 1..=80 {
            graphs.extend([rng.random(PathTree(n)), rng.random(StarTree(n))]);
        }
        graphs.extend((0..200).map(|_| rng.random(MixedTree(1usize..80))));
        for graph in graphs {
            let n = graph.vertices_size();
            let query = graph.contour_query_range();
            let mut values = vec![0i64; n];
            let mut data: Vec<_> = query.component_sizes().map(|n| vec![0i64; n]).collect();
            assert_eq!(query.len(), data.iter().map(Vec::len).sum());
            assert_eq!(query.is_empty(), n <= 1);
            let updates: Vec<_> = if n <= 5 {
                (0..n)
                    .flat_map(|u| (-1..=1).map(move |delta| (u, delta)))
                    .collect()
            } else {
                (0..200)
                    .map(|_| (rng.random(0..n), rng.random(-100..=100i64)))
                    .collect()
            };
            for (u, delta) in updates {
                values[u] += delta;
                query.for_each_index(u, |c, i| data[c][i] += delta);
                let ranges: Vec<_> = if n <= 5 {
                    (0..n)
                        .flat_map(|v| {
                            (0..=n).flat_map(move |l| (l..=n + 1).map(move |r| (v, l, r)))
                        })
                        .collect()
                } else {
                    let v = rng.random(0..n);
                    let l = rng.random(0..=n);
                    vec![(v, l, rng.random(l..=n + 1))]
                };
                for (v, l, r) in ranges {
                    let distances = graph.tree_depth(v);
                    let expected: i64 = (0..n)
                        .filter(|&u| {
                            u != v && l <= distances[u] as usize && (distances[u] as usize) < r
                        })
                        .map(|u| values[u])
                        .sum();
                    let mut actual = 0;
                    query.for_each_contour_range(v, l, r, |c, start, end| {
                        actual += data[c][start..end].iter().sum::<i64>()
                    });
                    assert_eq!(actual, expected);
                }
            }
        }
    }
}
