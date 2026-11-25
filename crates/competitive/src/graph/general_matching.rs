use std::mem::swap;

#[derive(Debug, Clone)]
pub struct GeneralMatching {
    size: usize,
    edges: Vec<(usize, usize)>,
    partners: Partners,
    matching_size: usize,
    status: Status,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Status {
    Dirty,
    Maximum,
    AtActiveBound,
}

#[derive(Debug, Clone)]
enum Partners {
    Saturated(Vec<u32>),
    Wide(Vec<usize>),
}

impl GeneralMatching {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            edges: Vec::new(),
            partners: Partners::Wide(Vec::new()),
            matching_size: 0,
            status: Status::Maximum,
        }
    }
    pub fn add_edge(&mut self, u: usize, v: usize) {
        assert!(u < self.size && v < self.size);
        if u != v && self.matching_size < self.size / 2 {
            self.edges.push((u, v));
            // Both endpoints were already counted in the non-isolated vertex bound.
            if self.status == Status::AtActiveBound
                && let Partners::Wide(mate) = &self.partners
                && mate[u] < self.size
                && mate[v] < self.size
            {
                return;
            }
            self.status = Status::Dirty;
        }
    }
    pub fn from_edges(size: usize, edges: &[(usize, usize)]) -> Self {
        let mut this = Self::new(size);
        (this.partners, this.matching_size) = Self::initial_matching(size, edges);
        if this.matching_size < size / 2 {
            this.edges
                .extend(edges.iter().copied().filter(|&(u, v)| u != v));
            if !this.edges.is_empty() {
                this.status = Status::Dirty;
            }
        }
        this
    }
    pub fn maximum_matching(&mut self) -> Vec<(usize, usize)> {
        if self.status == Status::Dirty {
            let mut status = Status::Maximum;
            if self.matching_size == 0 {
                (self.partners, self.matching_size) =
                    Self::initial_matching(self.size, &self.edges);
            }
            if let Partners::Wide(mate) = &mut self.partners
                && self.matching_size < self.size / 2
            {
                let (count, upper) = Self::complete(mate, self.matching_size, &self.edges);
                self.matching_size = count;
                if count == upper {
                    status = Status::AtActiveBound;
                }
            }
            self.status = status;
            if self.matching_size == self.size / 2 {
                self.edges = Vec::new();
            }
        }
        let collect = |(u, v): (usize, usize)| (u < v && v < self.size).then_some((u, v));
        let mut pairs = Vec::with_capacity(self.matching_size);
        match &self.partners {
            Partners::Saturated(mate) => {
                pairs.extend(
                    mate.iter()
                        .map(|&v| v as usize)
                        .enumerate()
                        .filter_map(collect),
                );
            }
            Partners::Wide(mate) => {
                pairs.extend(mate.iter().copied().enumerate().filter_map(collect))
            }
        }
        pairs
    }
    fn initial_matching(size: usize, edges: &[(usize, usize)]) -> (Partners, usize) {
        fn greedy<T>(
            size: usize,
            edges: &[(usize, usize)],
            encode: impl Fn(usize) -> T,
            decode: impl Fn(T) -> usize,
        ) -> (Vec<T>, usize)
        where
            T: Copy + Eq,
        {
            let free = encode(size);
            let mut mate = vec![free; size];
            let mut count = 0;
            for &(u, v) in edges.iter().rev() {
                assert!(u < size && v < size);
                if u != v && mate[u] == free && mate[v] == free {
                    mate[u] = encode(v);
                    mate[v] = encode(u);
                    count += 1;
                }
            }
            if count < size / 2 {
                let mut neighbor = vec![size; size];
                for &(u, v) in edges.iter().rev() {
                    if u == v {
                        continue;
                    }
                    let (a, b) = if mate[u] == free {
                        (u, v)
                    } else if mate[v] == free {
                        (v, u)
                    } else {
                        continue;
                    };
                    // Maximality of the greedy matching guarantees that b has a partner.
                    let c = decode(mate[b]);
                    let d = neighbor[c];
                    if d != size && d != a && mate[d] == free {
                        mate[a] = encode(b);
                        mate[b] = encode(a);
                        mate[c] = encode(d);
                        mate[d] = encode(c);
                        count += 1;
                        if count == size / 2 {
                            break;
                        }
                    } else {
                        neighbor[b] = a;
                    }
                }
            }
            (mate, count)
        }

        if u32::try_from(size).is_ok() {
            let (mate, count) = greedy(size, edges, |v| v as u32, |v| v as usize);
            if count == size / 2 {
                (Partners::Saturated(mate), count)
            } else {
                (
                    Partners::Wide(mate.into_iter().map(|v| v as usize).collect()),
                    count,
                )
            }
        } else {
            let (mate, count) = greedy(size, edges, |v| v, |v| v);
            (Partners::Wide(mate), count)
        }
    }

    fn complete(mate: &mut [usize], mut count: usize, edges: &[(usize, usize)]) -> (usize, usize) {
        let size = mate.len();
        let mut degree = vec![0; size];
        let mut neighbors_xor = vec![0; size];
        for &(u, v) in edges {
            degree[u] += 1;
            degree[v] += 1;
            neighbors_xor[u] ^= v;
            neighbors_xor[v] ^= u;
        }
        let upper = degree.iter().filter(|&&d| d != 0).count() / 2;
        if count == upper {
            return (count, upper);
        }
        if !degree.contains(&1) && degree.contains(&2) {
            let mut candidate = vec![size; size];
            let mut candidate_size = 0;
            for &(u, v) in edges {
                if degree[u] != 2 || degree[v] != 2 || candidate[u] != size || candidate[v] != size
                {
                    continue;
                }
                // On a-u-v-b, prefer two disjoint end edges to the middle edge.
                let a = neighbors_xor[u] ^ v;
                let b = neighbors_xor[v] ^ u;
                if a != v && b != u && a != b && candidate[a] == size && candidate[b] == size {
                    candidate[a] = u;
                    candidate[u] = a;
                    candidate[b] = v;
                    candidate[v] = b;
                    candidate_size += 2;
                } else {
                    candidate[u] = v;
                    candidate[v] = u;
                    candidate_size += 1;
                }
            }
            if candidate_size > count {
                mate.copy_from_slice(&candidate);
                count = candidate_size;
                if count == upper {
                    return (count, upper);
                }
            }
        }
        for root in 0..size {
            let mut u = root;
            while degree[u] == 1 {
                let v = neighbors_xor[u];
                if mate[u] == size {
                    let old = mate[v];
                    // A partner already peeled from the graph fixes this matching edge.
                    if old == size || degree[old] != 0 {
                        if old == size {
                            count += 1;
                        } else {
                            mate[old] = size;
                        }
                        mate[u] = v;
                        mate[v] = u;
                    }
                }
                degree[u] = 0;
                degree[v] -= 1;
                neighbors_xor[v] ^= u;
                u = v;
            }
        }
        let mut free = (0..size)
            .filter(|&v| mate[v] == size && degree[v] != 0)
            .count();
        if free < 2 {
            return (count, upper);
        }
        for &(u, v) in edges.iter().rev() {
            if mate[u] == size && mate[v] == size {
                mate[u] = v;
                mate[v] = u;
                count += 1;
                free -= 2;
                if free < 2 {
                    return (count, upper);
                }
            }
        }
        if Self::certifies_maximum(mate, &degree, edges) {
            return (count, upper);
        }

        let active: Vec<_> = (0..size)
            .map(|v| degree[v] != 0 && (mate[v] == size || degree[mate[v]] != 0))
            .collect();
        let mut start = vec![0; size + 1];
        for &(u, v) in edges {
            if active[u] && active[v] {
                start[u] += 1;
                start[v] += 1;
            }
        }
        for v in 0..size {
            start[v + 1] += start[v];
        }
        let mut adj = vec![0; start[size]];
        for &(u, v) in edges.iter().rev() {
            if active[u] && active[v] {
                start[u] -= 1;
                adj[start[u]] = v;
                start[v] -= 1;
                adj[start[v]] = u;
            }
        }
        let mut search = Search {
            size,
            start: &start,
            adj: &adj,
            mate,
            matching_size: count,
            parent: vec![size; size + 1],
            base: (0..=size).collect(),
            state: vec![Search::UNSEEN; size],
            seen: vec![0; size],
            queue: Vec::new(),
            timestamp: 0,
        };
        search.compute();
        (search.matching_size, upper)
    }

    fn certifies_maximum(mate: &[usize], degree: &[usize], edges: &[(usize, usize)]) -> bool {
        use std::cmp::Ordering::{Greater, Less};

        let size = mate.len();
        // Deleting the higher-degree end of each unequal pair removes one matched edge
        // per deleted vertex. The bound is tight if each remaining component has at most
        // one exposed vertex.
        let role: Vec<_> = (0..size)
            .map(|v| {
                if mate[v] == size {
                    Less
                } else {
                    degree[v].cmp(&degree[mate[v]])
                }
            })
            .collect();
        let mut base: Vec<_> = (0..size).collect();
        for &(u, v) in edges {
            if role[u] == Greater || role[v] == Greater || base[u] == base[v] {
                continue;
            }
            let u = find(&mut base, u);
            let v = find(&mut base, v);
            if u == v {
                continue;
            }
            // A component's root is exposed in the remaining matching if any vertex is.
            match (role[u], role[v]) {
                (Less, Less) => return false,
                (_, Less) => base[u] = v,
                _ => base[v] = u,
            }
        }
        true
    }
}

fn find(base: &mut [usize], mut v: usize) -> usize {
    while base[v] != v {
        let parent = base[v];
        base[v] = base[parent];
        v = base[v];
    }
    v
}

struct Search<'a> {
    size: usize,
    start: &'a [usize],
    adj: &'a [usize],
    mate: &'a mut [usize],
    matching_size: usize,
    parent: Vec<usize>,
    base: Vec<usize>,
    state: Vec<u8>,
    seen: Vec<usize>,
    queue: Vec<usize>,
    timestamp: usize,
}
impl Search<'_> {
    fn compute(&mut self) {
        for u in 0..self.size {
            if self.mate[u] != self.size {
                continue;
            }
            if let Some(&v) = self.adj[self.start[u]..self.start[u + 1]]
                .iter()
                .find(|&&v| self.mate[v] == self.size)
            {
                self.mate[u] = v;
                self.mate[v] = u;
                self.matching_size += 1;
            } else if let Some((&v, rest)) =
                self.adj[self.start[u]..self.start[u + 1]].split_first()
                && rest.iter().all(|&w| w == v)
            {
                let old = self.mate[v];
                self.mate[old] = self.size;
                self.mate[v] = u;
                self.mate[u] = v;
            }
        }
        let mut free = (0..self.size)
            .filter(|&v| self.mate[v] == self.size && self.start[v] != self.start[v + 1])
            .count();
        if free < 2 {
            return;
        }

        for v in 0..self.size {
            if self.mate[v] == self.size && self.start[v] != self.start[v + 1] {
                free -= 1;
                if self.augment_from(v) {
                    self.matching_size += 1;
                    free -= 1;
                }
                if free < 2 {
                    break;
                }
            }
        }
    }

    const UNSEEN: u8 = 0;
    const OUTER: u8 = 1;
    const INNER: u8 = 2;

    fn lca(&mut self, mut u: usize, mut v: usize) -> usize {
        self.timestamp += 1;
        u = find(&mut self.base, u);
        v = find(&mut self.base, v);
        loop {
            if u != self.size {
                if self.seen[u] == self.timestamp {
                    return u;
                }
                self.seen[u] = self.timestamp;
                u = find(&mut self.base, self.parent[self.mate[u]]);
            }
            swap(&mut u, &mut v);
        }
    }

    fn contract(&mut self, mut v: usize, mut child: usize, ancestor: usize) {
        while find(&mut self.base, v) != ancestor {
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

    fn augment(&mut self, mut u: usize, mut v: usize) {
        for &outer in &self.queue {
            self.base[outer] = outer;
            self.state[outer] = Self::UNSEEN;
            let inner = self.mate[outer];
            if inner != self.size {
                self.base[inner] = inner;
                self.state[inner] = Self::UNSEEN;
            }
        }
        while u != self.size {
            let next = self.mate[u];
            self.mate[u] = v;
            self.mate[v] = u;
            v = next;
            u = self.parent[v];
        }
    }

    fn augment_from(&mut self, root: usize) -> bool {
        self.queue.clear();
        self.state[root] = Self::OUTER;
        self.queue.push(root);
        let mut head = 0;
        while head < self.queue.len() {
            let u = self.queue[head];
            head += 1;
            for edge in self.start[u]..self.start[u + 1] {
                let v = self.adj[edge];
                if self.state[v] == Self::UNSEEN {
                    self.parent[v] = u;
                    if self.mate[v] == self.size {
                        self.augment(u, v);
                        return true;
                    }
                    self.state[v] = Self::INNER;
                    let v = self.mate[v];
                    self.state[v] = Self::OUTER;
                    self.queue.push(v);
                    if let Some(&end) = self.adj[self.start[v]..self.start[v + 1]]
                        .iter()
                        .find(|&&w| self.mate[w] == self.size && self.state[w] == Self::UNSEEN)
                    {
                        self.augment(v, end);
                        return true;
                    }
                } else if self.state[v] == Self::OUTER
                    && self.base[u] != self.base[v]
                    && find(&mut self.base, u) != find(&mut self.base, v)
                {
                    let ancestor = self.lca(u, v);
                    self.contract(u, v, ancestor);
                    self.contract(v, u, ancestor);
                }
            }
        }
        // Edges from this tree to unsearched vertices leave inner vertices.
        // An alternating path entering it cannot leave through another inner vertex.
        for &v in &self.queue {
            self.state[v] = Self::INNER;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        rand,
        tools::{Xorshift, testutil::exhaustive_sequences},
    };

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

    #[test]
    fn small_graphs() {
        for n in 0..=6 {
            let pairs: Vec<_> = (0..n).flat_map(|v| (0..v).map(move |u| (u, v))).collect();
            for present in exhaustive_sequences([false, true], pairs.len()..=pairs.len()) {
                let edges: Vec<_> = pairs
                    .iter()
                    .copied()
                    .zip(present)
                    .filter_map(|(e, p)| p.then_some(e))
                    .collect();
                let mut g = GeneralMatching::from_edges(n, &edges);
                for v in 0..n {
                    g.add_edge(v, v);
                }
                for &(u, v) in &edges {
                    g.add_edge(u, v);
                }
                assert_eq!(
                    g.maximum_matching().len(),
                    brute_maximum_matching(n, &edges)
                );
            }
        }
    }

    #[test]
    fn disjoint_cycles() {
        let mut rng = Xorshift::default();
        for lengths in exhaustive_sequences(3..=18, 1..=4) {
            let n = lengths.iter().sum();
            let mut edges = Vec::with_capacity(n);
            let mut start = 0;
            for &len in &lengths {
                edges.extend((start + 1..start + len).map(|v| (v - 1, v)));
                edges.push((start, start + len - 1));
                start += len;
            }
            rng.shuffle(&mut edges);
            let pairs = GeneralMatching::from_edges(n, &edges).maximum_matching();
            assert_eq!(pairs.len(), lengths.iter().map(|len| len / 2).sum());
            let mut used = vec![false; n];
            for (u, v) in pairs {
                assert!(edges.contains(&(u, v)) && !used[u] && !used[v]);
                used[u] = true;
                used[v] = true;
            }
        }
    }
}
