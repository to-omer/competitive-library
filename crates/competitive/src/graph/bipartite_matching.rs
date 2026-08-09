use std::{collections::VecDeque, mem::swap};

#[derive(Debug, Clone)]
pub struct BipartiteMatching {
    left_size: usize,
    right_size: usize,
    left_graph: Vec<Vec<usize>>,
    right_graph: Vec<Vec<usize>>,
    left_match: Vec<Option<usize>>,
    right_match: Vec<Option<usize>>,
    matching_size: usize,
}

impl BipartiteMatching {
    pub fn new(left_size: usize, right_size: usize) -> Self {
        Self {
            left_size,
            right_size,
            left_graph: vec![vec![]; left_size],
            right_graph: vec![vec![]; right_size],
            left_match: vec![None; left_size],
            right_match: vec![None; right_size],
            matching_size: 0,
        }
    }
    pub fn add_edge(&mut self, l: usize, r: usize) {
        assert!(l < self.left_size);
        assert!(r < self.right_size);
        self.left_graph[l].push(r);
        self.right_graph[r].push(l);
        self.matching_size = !0;
    }
    pub fn from_edges(left_size: usize, right_size: usize, lr: &[(usize, usize)]) -> Self {
        let mut left_deg = vec![0usize; left_size];
        let mut right_deg = vec![0usize; right_size];
        for &(l, r) in lr {
            left_deg[l] += 1;
            right_deg[r] += 1;
        }
        let mut left_graph: Vec<_> = left_deg.into_iter().map(Vec::with_capacity).collect();
        let mut right_graph: Vec<_> = right_deg.into_iter().map(Vec::with_capacity).collect();
        for &(l, r) in lr {
            assert!(l < left_size);
            assert!(r < right_size);
            left_graph[l].push(r);
            right_graph[r].push(l);
        }
        Self {
            left_size,
            right_size,
            left_graph,
            right_graph,
            left_match: vec![None; left_size],
            right_match: vec![None; right_size],
            matching_size: !0,
        }
    }
    pub fn hopcroft_karp(&mut self) {
        fn bfs(bm: &BipartiteMatching, deq: &mut VecDeque<usize>, level: &mut [usize]) {
            deq.clear();
            for (i, r) in bm.left_match.iter().enumerate() {
                if r.is_none() {
                    deq.push_back(i);
                    level[i] = 0;
                }
            }
            while let Some(l) = deq.pop_front() {
                for &r in &bm.left_graph[l] {
                    if let Some(nl) = bm.right_match[r]
                        && level[nl] == !0
                    {
                        deq.push_back(nl);
                        level[nl] = level[l] + 1;
                    }
                }
            }
        }
        fn dfs(
            bm: &mut BipartiteMatching,
            l: usize,
            level: &mut [usize],
            used: &mut [bool],
        ) -> bool {
            used[l] = true;
            for i in 0..bm.left_graph[l].len() {
                let r = bm.left_graph[l][i];
                if let Some(nl) = bm.right_match[r]
                    && (used[nl] || level[l] + 1 != level[nl] || !dfs(bm, nl, level, used))
                {
                    continue;
                }
                bm.right_match[r] = Some(l);
                bm.left_match[l] = Some(r);
                return true;
            }
            false
        }
        if self.matching_size != !0 {
            return;
        }
        self.matching_size = self.left_match.iter().filter(|r| r.is_some()).count();
        let mut used = vec![false; self.left_size];
        let mut level = vec![!0; self.left_size];
        let mut deq = VecDeque::with_capacity(self.left_size);
        loop {
            bfs(self, &mut deq, &mut level);
            let prev_size = self.matching_size;
            for l in 0..self.left_size {
                if self.left_match[l].is_none() {
                    self.matching_size += dfs(self, l, &mut level, &mut used) as usize;
                }
            }
            if self.matching_size == prev_size {
                break;
            }
            for item in level.iter_mut() {
                *item = !0;
            }
            for item in used.iter_mut() {
                *item = false;
            }
        }
    }
    pub fn kuhn_multi_start_bfs(&mut self) {
        if self.matching_size != !0 {
            return;
        }
        self.matching_size = self.left_match.iter().filter(|r| r.is_some()).count();
        let mut deq = VecDeque::with_capacity(self.left_size);
        let mut prev = vec![!0usize; self.left_size];
        let mut root = vec![!0usize; self.left_size];
        loop {
            let mut changed = false;
            for (l, r) in self.left_match.iter().enumerate() {
                if r.is_none() {
                    root[l] = l;
                    deq.push_back(l);
                }
            }
            while let Some(mut l) = deq.pop_front() {
                if self.left_match[root[l]].is_some() {
                    continue;
                }
                for mut r in self.left_graph[l].iter().cloned() {
                    if let Some(nl) = self.right_match[r] {
                        if prev[nl] == !0 {
                            prev[nl] = l;
                            root[nl] = root[l];
                            deq.push_back(nl);
                        }
                    } else {
                        loop {
                            self.right_match[r] = Some(l);
                            if let Some(nr) = &mut self.left_match[l] {
                                swap(nr, &mut r);
                                l = prev[l];
                            } else {
                                self.left_match[l] = Some(r);
                                break;
                            }
                        }
                        changed = true;
                        self.matching_size += 1;
                        break;
                    }
                }
            }
            if !changed {
                break;
            }
            for item in prev.iter_mut() {
                *item = !0;
            }
            for item in root.iter_mut() {
                *item = !0;
            }
        }
    }
    pub fn push_relabel(&mut self) {
        if self.matching_size != !0 {
            return;
        }
        let size = self.left_size + self.right_size;
        let mut level_left = vec![size; self.left_size];
        let mut level_right = vec![size; self.right_size];
        let mut bfs = VecDeque::with_capacity(self.left_size);
        let mut queue: VecDeque<_> = self
            .right_match
            .iter()
            .enumerate()
            .filter_map(|(right, left)| left.is_none().then_some(right))
            .collect();
        let mut iteration = 0;
        while let Some(right) = queue.pop_front() {
            if iteration == 0 {
                level_left.fill(size);
                level_right.fill(size);
                bfs.clear();
                for (left, right) in self.left_match.iter().enumerate() {
                    if right.is_none() {
                        level_left[left] = 0;
                        bfs.push_back(left);
                    }
                }
                while let Some(left) = bfs.pop_front() {
                    for &right in &self.left_graph[left] {
                        if level_right[right] > level_left[left] + 1 {
                            level_right[right] = level_left[left] + 1;
                            if let Some(next_left) = self.right_match[right] {
                                level_left[next_left] = level_right[right] + 1;
                                bfs.push_back(next_left);
                            }
                        }
                    }
                }
            }

            let mut selected = !0;
            let mut selected_level = size;
            for &left in &self.right_graph[right] {
                if level_left[left] < selected_level {
                    selected = left;
                    selected_level = level_left[left];
                }
            }
            if selected != !0 {
                level_right[right] = selected_level + 1;
                if let Some(previous_right) = self.left_match[selected].take() {
                    self.right_match[previous_right] = None;
                    queue.push_back(previous_right);
                }
                self.left_match[selected] = Some(right);
                self.right_match[right] = Some(selected);
                level_left[selected] += 2;
            }

            iteration += 1;
            if iteration == size {
                iteration = 0;
            }
        }
        self.matching_size = self
            .left_match
            .iter()
            .filter(|right| right.is_some())
            .count();
    }
    pub fn maximum_matching(&mut self) -> Vec<(usize, usize)> {
        self.push_relabel();
        self.left_match
            .iter()
            .enumerate()
            .filter_map(|(l, r)| r.map(|r| (l, r)))
            .collect()
    }
    pub fn minimum_edge_cover(&mut self) -> Vec<(usize, usize)> {
        self.push_relabel();
        let mut res = Vec::with_capacity(self.left_size + self.right_size - self.matching_size);
        let mut left_used: Vec<_> = self.left_match.iter().map(Option::is_some).collect();
        let mut right_used: Vec<_> = self.right_match.iter().map(Option::is_some).collect();
        for (l, lg) in self.left_graph.iter().enumerate() {
            if let Some(r) = self.left_match[l] {
                res.push((l, r));
            }
            for &r in lg {
                if !left_used[l] || !right_used[r] {
                    left_used[l] = true;
                    right_used[r] = true;
                    res.push((l, r));
                }
            }
        }
        res
    }
    fn reachable(&mut self) -> (Vec<bool>, Vec<bool>) {
        #[derive(Clone, Copy)]
        enum Either {
            Left(usize),
            Right(usize),
        }
        self.push_relabel();
        let mut left_used = vec![false; self.left_size];
        let mut right_used = vec![false; self.right_size];
        let mut deq = VecDeque::new();
        for (l, r) in self.left_match.iter().enumerate() {
            if r.is_none() {
                left_used[l] = true;
                deq.push_back(Either::Left(l));
            }
        }
        loop {
            match deq.pop_front() {
                Some(Either::Left(l)) => {
                    for &r in &self.left_graph[l] {
                        if self.left_match[l] != Some(r) && !right_used[r] {
                            right_used[r] = true;
                            deq.push_back(Either::Right(r));
                        }
                    }
                }
                Some(Either::Right(r)) => {
                    if let Some(l) = self.right_match[r]
                        && !left_used[l]
                    {
                        left_used[l] = true;
                        deq.push_back(Either::Left(l));
                    }
                }
                None => break,
            }
        }
        (left_used, right_used)
    }
    pub fn minimum_vertex_cover(&mut self) -> (Vec<usize>, Vec<usize>) {
        let (left_used, right_used) = self.reachable();
        (
            left_used
                .into_iter()
                .enumerate()
                .filter_map(|(l, b)| if !b { Some(l) } else { None })
                .collect(),
            right_used
                .into_iter()
                .enumerate()
                .filter_map(|(r, b)| if b { Some(r) } else { None })
                .collect(),
        )
    }
    pub fn maximum_independent_set(&mut self) -> (Vec<usize>, Vec<usize>) {
        let (left_used, right_used) = self.reachable();
        (
            left_used
                .into_iter()
                .enumerate()
                .filter_map(|(l, b)| if b { Some(l) } else { None })
                .collect(),
            right_used
                .into_iter()
                .enumerate()
                .filter_map(|(r, b)| if !b { Some(r) } else { None })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chmax, chmin, data_structure::UnionFind, rand, tools::Xorshift};

    fn gen_graph(n: usize, m: usize) -> Vec<(usize, usize)> {
        let mut rng = Xorshift::default();
        let mut uf = UnionFind::new(n + m);
        let mut lr = vec![];
        while uf.size(0) < n + m {
            rand!(rng, l: 0..n, r: 0..m);
            uf.unite(l, r + n);
            lr.push((l, r));
        }
        lr.sort_unstable();
        lr.dedup();
        lr
    }

    const Q: usize = 100;
    const N: usize = 8;
    const M: usize = 8;

    #[test]
    fn test_maximum_matching() {
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let n = rng.rand((N + 1) as u64) as usize;
            let m = rng.rand((M + 1) as u64) as usize;
            let mut lr = vec![];
            for l in 0..n {
                for r in 0..m {
                    if rng.rand(3) == 0 {
                        lr.push((l, r));
                    }
                }
            }

            let mut reachable = vec![false; 1 << m];
            reachable[0] = true;
            for l in 0..n {
                let mut next = reachable.clone();
                for (bits, &can_match) in reachable.iter().enumerate() {
                    if can_match {
                        for &(_, r) in lr.iter().filter(|&&(left, _)| left == l) {
                            next[bits | 1 << r] = true;
                        }
                    }
                }
                reachable = next;
            }
            let expected = reachable
                .iter()
                .enumerate()
                .filter_map(|(bits, &reachable)| reachable.then_some(bits.count_ones() as usize))
                .max()
                .unwrap();

            for incremental in [false, true] {
                let mut bm = if incremental {
                    let mut bm = BipartiteMatching::new(n, m);
                    for &(l, r) in &lr[..lr.len() / 2] {
                        bm.add_edge(l, r);
                    }
                    bm.maximum_matching();
                    for &(l, r) in &lr[lr.len() / 2..] {
                        bm.add_edge(l, r);
                    }
                    bm
                } else {
                    BipartiteMatching::from_edges(n, m, &lr)
                };
                let matching = bm.maximum_matching();
                assert_eq!(matching.len(), expected);
                let mut left_used = vec![false; n];
                let mut right_used = vec![false; m];
                for (l, r) in matching {
                    assert!(lr.contains(&(l, r)));
                    assert!(!left_used[l]);
                    assert!(!right_used[r]);
                    left_used[l] = true;
                    right_used[r] = true;
                }
            }
        }
    }

    #[test]
    fn test_minimum_edge_cover() {
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 4..=N, m: 4..=M);
            let lr = gen_graph(n, m);
            let mut dp = vec![vec![!0usize; 1 << m]; 1 << n];
            dp[0][0] = 0;
            for bitl in 0usize..1 << n {
                for bitr in 0usize..1 << m {
                    if dp[bitl][bitr] == !0 {
                        continue;
                    }
                    for &(l, r) in &lr {
                        chmin!(dp[bitl | (1 << l)][bitr | (1 << r)], dp[bitl][bitr] + 1);
                    }
                }
            }
            let cover = BipartiteMatching::from_edges(n, m, &lr).minimum_edge_cover();
            assert_eq!(*dp.last().unwrap().last().unwrap(), cover.len());
            let mut left_used = vec![false; n];
            let mut right_used = vec![false; m];
            for (l, r) in cover {
                left_used[l] = true;
                right_used[r] = true;
            }
            assert!(left_used.iter().all(|&b| b));
            assert!(right_used.iter().all(|&b| b));

            let mut bm = BipartiteMatching::new(n, m);
            for &(l, r) in &lr {
                bm.add_edge(l, r);
            }
            bm.hopcroft_karp();
            let cover = bm.minimum_edge_cover();
            assert_eq!(*dp.last().unwrap().last().unwrap(), cover.len());
            let mut left_used = vec![false; n];
            let mut right_used = vec![false; m];
            for (l, r) in cover {
                left_used[l] = true;
                right_used[r] = true;
            }
            assert!(left_used.iter().all(|&b| b));
            assert!(right_used.iter().all(|&b| b));
        }
    }

    #[test]
    fn test_minimum_vertex_cover() {
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 4..=N, m: 4..=M);
            let lr = gen_graph(n, m);
            let mut ans = !0usize;
            for bitl in 0usize..1 << n {
                for bitr in 0usize..1 << m {
                    if lr
                        .iter()
                        .all(|&(l, r)| bitl & (1 << l) != 0 || bitr & (1 << r) != 0)
                    {
                        chmin!(ans, (bitl.count_ones() + bitr.count_ones()) as usize);
                    }
                }
            }
            let set = BipartiteMatching::from_edges(n, m, &lr).minimum_vertex_cover();
            assert_eq!(ans, set.0.len() + set.1.len());
            for &(l, r) in &lr {
                assert!(set.0.contains(&l) || set.1.contains(&r));
            }

            let mut bm = BipartiteMatching::new(n, m);
            for &(l, r) in &lr {
                bm.add_edge(l, r);
            }
            bm.hopcroft_karp();
            let set = bm.minimum_vertex_cover();
            assert_eq!(ans, set.0.len() + set.1.len());
            for &(l, r) in &lr {
                assert!(set.0.contains(&l) || set.1.contains(&r));
            }
        }
    }

    #[test]
    fn test_maximum_independent_set() {
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 4..=N, m: 4..=M);
            let lr = gen_graph(n, m);
            let mut ans = 0usize;
            for bitl in 0usize..1 << n {
                for bitr in 0usize..1 << m {
                    if lr
                        .iter()
                        .all(|&(l, r)| bitl & (1 << l) == 0 || bitr & (1 << r) == 0)
                    {
                        chmax!(ans, (bitl.count_ones() + bitr.count_ones()) as usize);
                    }
                }
            }
            let set = BipartiteMatching::from_edges(n, m, &lr).maximum_independent_set();
            assert_eq!(ans, set.0.len() + set.1.len());
            for &(l, r) in &lr {
                assert!(!set.0.contains(&l) || !set.1.contains(&r));
            }

            let mut bm = BipartiteMatching::new(n, m);
            for &(l, r) in &lr {
                bm.add_edge(l, r);
            }
            bm.hopcroft_karp();
            let set = bm.maximum_independent_set();
            assert_eq!(ans, set.0.len() + set.1.len());
            for &(l, r) in &lr {
                assert!(!set.0.contains(&l) || !set.1.contains(&r));
            }
        }
    }
}
