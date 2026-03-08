use super::{ConvolveSteps, U64Convolve, UndirectedSparseGraph};
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

    fn split_centroid(&self) -> CentroidSplit {
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
        let rsize = n - lsize - 1;
        let mut whole_parents = vec![usize::MAX; n];
        let mut whole_vs = vec![usize::MAX; n];
        let mut left_parents = vec![usize::MAX; lsize + 1];
        let mut left_vs = vec![usize::MAX; lsize + 1];
        let mut right_parents = vec![usize::MAX; rsize + 1];
        let mut right_vs = vec![usize::MAX; rsize + 1];
        for u in 0..n {
            let i = order[u];
            whole_vs[i] = vs[u];
            if side[u] != 1 {
                left_vs[i] = vs[u];
            }
            if side[u] != 0 {
                right_vs[if i == 0 { 0 } else { i - lsize }] = vs[u];
            }
        }
        for u in 1..n {
            let mut x = order[u];
            let mut y = order[parents[u]];
            if x > y {
                swap(&mut x, &mut y);
            }
            whole_parents[y] = x;
            if side[u] != 1 && side[parents[u]] != 1 {
                left_parents[y] = x;
            }
            if side[u] != 0 && side[parents[u]] != 0 {
                right_parents[if y == 0 { 0 } else { y - lsize }] =
                    if x == 0 { 0 } else { x - lsize };
            }
        }
        CentroidSplit {
            whole: RootedTree {
                parents: whole_parents,
                vs: whole_vs,
            },
            left: RootedTree {
                parents: left_parents,
                vs: left_vs,
            },
            right: RootedTree {
                parents: right_parents,
                vs: right_vs,
            },
            old_to_new: order,
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

    fn append_contour_components(
        &self,
        color: &[i8],
        comp_range: &mut Vec<usize>,
        vertex_info: &mut [Vec<ContourInfo>],
    ) {
        let n = self.len();
        let mut dist = vec![0usize; n];
        for i in 1..n {
            dist[i] = dist[self.parents[i]] + 1;
        }
        let mut comp = comp_range.len() - 1;
        for c1 in [0, 1] {
            let mut max_a = 0usize;
            let mut max_b = 0usize;
            let mut has_a = false;
            let mut has_b = false;
            for (v, &c2) in color.iter().enumerate() {
                if c2 == c1 {
                    has_a = true;
                    max_a = max_a.max(dist[v]);
                } else if c2 > c1 {
                    has_b = true;
                    max_b = max_b.max(dist[v]);
                }
            }
            if !has_a || !has_b {
                continue;
            }
            for (v, &c2) in color.iter().enumerate() {
                if c2 == c1 {
                    vertex_info[self.vs[v]].push(ContourInfo { comp, dep: dist[v] });
                }
            }
            comp_range.push(comp_range[comp] + max_a + 1);
            comp += 1;
            for (v, &c2) in color.iter().enumerate() {
                if c2 > c1 {
                    vertex_info[self.vs[v]].push(ContourInfo { comp, dep: dist[v] });
                }
            }
            comp_range.push(comp_range[comp] + max_b + 1);
            comp += 1;
        }
    }

    fn build_contour_query_range(
        self,
        real: Vec<bool>,
        comp_range: &mut Vec<usize>,
        vertex_info: &mut [Vec<ContourInfo>],
    ) {
        let n = self.len();
        if n <= 1 {
            return;
        }
        if n == 2 {
            if real[0] && real[1] {
                self.append_contour_components(&[0, 1], comp_range, vertex_info);
            }
            return;
        }
        let split = self.split_centroid();
        let mut whole_real = vec![false; n];
        for (u, &is_real) in real.iter().enumerate() {
            if is_real {
                whole_real[split.old_to_new[u]] = true;
            }
        }
        let mut color = vec![-1i8; n];
        for (i, &is_real) in whole_real.iter().enumerate().skip(1) {
            if is_real {
                color[i] = if i <= split.lsize { 0 } else { 1 };
            }
        }
        if whole_real[0] {
            color[0] = 2;
        }
        split
            .whole
            .append_contour_components(&color, comp_range, vertex_info);
        if whole_real[0] {
            whole_real[0] = false;
        }
        let mut right_real = Vec::with_capacity(split.rsize() + 1);
        right_real.push(whole_real[0]);
        right_real.extend_from_slice(&whole_real[split.lsize + 1..]);
        split.left.build_contour_query_range(
            whole_real[..=split.lsize].to_vec(),
            comp_range,
            vertex_info,
        );
        split
            .right
            .build_contour_query_range(right_real, comp_range, vertex_info);
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
            for a in graph.adjacencies(u) {
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
    old_to_new: Vec<usize>,
    lsize: usize,
}

impl CentroidSplit {
    fn rsize(&self) -> usize {
        self.whole.len() - self.lsize - 1
    }
}

#[derive(Debug, Clone, Copy)]
struct ContourInfo {
    comp: usize,
    dep: usize,
}

#[derive(Debug, Clone)]
pub struct ContourQueryRange {
    comp_range: Vec<usize>,
    info_indptr: Vec<usize>,
    infos: Vec<ContourInfo>,
}

impl ContourQueryRange {
    pub fn len(&self) -> usize {
        self.comp_range.last().copied().unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn for_each_index(&self, v: usize, mut f: impl FnMut(usize)) {
        for info in &self.infos[self.info_indptr[v]..self.info_indptr[v + 1]] {
            f(self.comp_range[info.comp] + info.dep);
        }
    }

    pub fn for_each_contour_range(
        &self,
        v: usize,
        l: usize,
        r: usize,
        mut f: impl FnMut(usize, usize),
    ) {
        for info in &self.infos[self.info_indptr[v]..self.info_indptr[v + 1]] {
            let comp = info.comp ^ 1;
            let start = self.comp_range[comp];
            let len = self.comp_range[comp + 1] - start;
            let lo = l.saturating_sub(info.dep).min(len);
            let hi = r.saturating_sub(info.dep).min(len);
            if lo < hi {
                f(start + lo, start + hi);
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
        if n <= 1 {
            return ContourQueryRange {
                comp_range: vec![0],
                info_indptr: vec![0; n + 1],
                infos: vec![],
            };
        }
        let mut comp_range = vec![0usize];
        let mut vertex_info = vec![vec![]; n];
        RootedTree::from(self).build_contour_query_range(
            vec![true; n],
            &mut comp_range,
            &mut vertex_info,
        );
        let mut info_indptr = vec![0usize; n + 1];
        for (v, infos) in vertex_info.iter().enumerate() {
            info_indptr[v + 1] = info_indptr[v] + infos.len();
        }
        let mut infos = Vec::with_capacity(info_indptr[n]);
        for entries in vertex_info {
            infos.extend(entries);
        }
        ContourQueryRange {
            comp_range,
            info_indptr,
            infos,
        }
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

    #[test]
    fn test_contour_query_range_counts() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let g = rng.random(MixedTree(1usize..80));
            let n = g.vertices_size();
            let cq = g.contour_query_range();
            let mut data = vec![0; cq.len()];
            for v in 0..n {
                cq.for_each_index(v, |i| data[i] += 1);
            }
            for _ in 0..200 {
                let v = rng.random(0..n);
                let l = rng.random(0..=n);
                let r = rng.random(l..=n + 1);
                let dist = g.tree_depth(v);
                let expected = dist
                    .iter()
                    .enumerate()
                    .filter(|&(u, &d)| u != v && l <= d as usize && (d as usize) < r)
                    .count();
                let mut actual = 0usize;
                cq.for_each_contour_range(v, l, r, |start, end| {
                    actual += data[start..end].iter().sum::<usize>();
                });
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn test_contour_query_range_single_vertex() {
        let mut rng = Xorshift::default();
        for _ in 0..80 {
            let g = rng.random(MixedTree(1usize..60));
            let n = g.vertices_size();
            let cq = g.contour_query_range();
            for _ in 0..120 {
                let u = rng.random(0..n);
                let v = rng.random(0..n);
                let l = rng.random(0..=n);
                let r = rng.random(l..=n + 1);
                let mut data = vec![0; cq.len()];
                cq.for_each_index(u, |i| data[i] += 1);
                let expected = usize::from({
                    let d = g.tree_depth(v)[u] as usize;
                    u != v && l <= d && d < r
                });
                let mut actual = 0usize;
                cq.for_each_contour_range(v, l, r, |start, end| {
                    actual += data[start..end].iter().sum::<usize>();
                });
                assert_eq!(actual, expected);
            }
        }
    }
}
