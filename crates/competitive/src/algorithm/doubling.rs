use super::{Group, LevelAncestor, Monoid, UndirectedSparseGraph};
use std::collections::VecDeque;

pub struct Doubling<M>
where
    M: Monoid,
{
    size: usize,
    table: Vec<(usize, M::T)>,
}

impl<M> Doubling<M>
where
    M: Monoid,
{
    pub fn new(size: usize, f: impl Fn(usize) -> (usize, M::T)) -> Self {
        let mut table = Vec::with_capacity(size);
        for i in 0..size {
            table.push(f(i));
        }
        Self { size, table }
    }

    pub fn double(&mut self) {
        let base = self.table.len() - self.size;
        for i in 0..self.size {
            let &(to, ref val) = &self.table[base + i];
            if to != !0 {
                let &(to2, ref val2) = &self.table[base + to];
                self.table.push((to2, M::operate(val, val2)));
            } else {
                self.table.push((!0, M::unit()));
            }
        }
    }

    pub fn kth(&mut self, mut pos: usize, mut k: usize) -> (usize, M::T) {
        let mut x = M::unit();
        for chunk in self.table.chunks_exact(self.size) {
            if k & 1 == 1 {
                let &(to, ref val) = &chunk[pos];
                if to == !0 {
                    return (!0, M::unit());
                }
                x = M::operate(&x, val);
                pos = to;
            }
            k >>= 1;
            if k == 0 {
                break;
            }
        }
        while k > 0 {
            self.double();
            if k & 1 == 1 {
                let base = self.table.len() - self.size;
                let &(to, ref val) = &self.table[base + pos];
                if to == !0 {
                    return (!0, M::unit());
                }
                x = M::operate(&x, val);
                pos = to;
            }
            k >>= 1;
        }
        (pos, x)
    }

    /// queries: (pos, k)
    /// Return: (pos, acc)
    pub fn kth_multiple(
        &self,
        queries: impl IntoIterator<Item = (usize, usize)>,
    ) -> Vec<(usize, M::T)> {
        let (mut ks, mut results): (Vec<usize>, Vec<(usize, M::T)>) = queries
            .into_iter()
            .map(|(start, k)| (k, (start, M::unit())))
            .unzip();
        for chunk in self.table.chunks_exact(self.size) {
            for (i, k) in ks.iter_mut().enumerate() {
                if *k & 1 == 1 {
                    let &(to, ref val) = &chunk[results[i].0];
                    if to == !0 {
                        results[i] = (!0, M::unit());
                        *k = 0;
                    } else {
                        results[i].1 = M::operate(&results[i].1, val);
                        results[i].0 = to;
                    }
                }
                *k >>= 1;
            }
        }
        if ks.iter().any(|&k| k > 0) {
            let mut dp = self.table[self.table.len() - self.size..].to_vec();
            while ks.iter().any(|&k| k > 0) {
                let mut ndp = Vec::with_capacity(dp.len());
                for i in 0..self.size {
                    let &(to, ref val) = &dp[i];
                    if to != !0 {
                        let &(to2, ref val2) = &dp[to];
                        ndp.push((to2, M::operate(val, val2)));
                    } else {
                        ndp.push((!0, M::unit()));
                    }
                }
                dp = ndp;
                for (i, k) in ks.iter_mut().enumerate() {
                    if *k & 1 == 1 {
                        let &(to, ref val) = &dp[results[i].0];
                        if to == !0 {
                            results[i] = (!0, M::unit());
                            *k = 0;
                        } else {
                            results[i].1 = M::operate(&results[i].1, val);
                            results[i].0 = to;
                        }
                    }
                    *k >>= 1;
                }
            }
        }
        results
    }

    /// Return: (k, (pos, acc))
    pub fn find_last(
        &self,
        mut pos: usize,
        mut pred: impl FnMut(usize, &M::T) -> bool,
    ) -> (usize, (usize, M::T)) {
        let mut k = 0usize;
        let mut x = M::unit();
        assert!(pred(pos, &x));
        for (i, chunk) in self.table.chunks_exact(self.size).enumerate().rev() {
            let &(to, ref val) = &chunk[pos];
            let nx = M::operate(&x, val);
            if pred(to, &nx) {
                x = nx;
                pos = to;
                k |= 1 << i;
            }
        }
        (k, (pos, x))
    }

    /// Return: (k, (pos, acc))
    pub fn find_first(
        &self,
        pos: usize,
        mut pred: impl FnMut(usize, &M::T) -> bool,
    ) -> Option<(usize, (usize, M::T))> {
        let (mut k, (mut pos, mut x)) = self.find_last(pos, |k, x| !pred(k, x));
        k += 1;
        M::operate_assign(&mut x, &self.table[pos].1);
        pos = self.table[pos].0;
        if pred(pos, &x) {
            Some((k, (pos, x)))
        } else {
            None
        }
    }
}

pub struct FunctionalGraphDoubling<M>
where
    M: Group,
{
    depth_to_cycle: Vec<usize>,
    cycle_entry: Vec<usize>,
    cycle_id: Vec<usize>,
    cycle_pos: Vec<usize>,
    cycles: Vec<Vec<usize>>,
    cycle_prefix: Vec<Vec<M::T>>,
    prefix_up: Vec<M::T>,
    la: LevelAncestor,
}

impl<M> FunctionalGraphDoubling<M>
where
    M: Group,
{
    pub fn new(size: usize, f: impl Fn(usize) -> (usize, M::T)) -> Self {
        let (next, value): (Vec<_>, Vec<_>) = (0..size).map(f).unzip();

        let mut indeg = vec![0usize; size];
        for &to in &next {
            indeg[to] += 1;
        }
        let mut in_cycle = vec![true; size];
        let mut deq = VecDeque::new();
        for (u, &deg) in indeg.iter().enumerate() {
            if deg == 0 {
                deq.push_back(u);
            }
        }
        while let Some(u) = deq.pop_front() {
            in_cycle[u] = false;
            indeg[next[u]] -= 1;
            if indeg[next[u]] == 0 {
                deq.push_back(next[u]);
            }
        }

        let mut cycle_id = vec![!0; size];
        let mut cycle_pos = vec![!0; size];
        let mut cycles = Vec::new();
        for i in 0..size {
            if in_cycle[i] && cycle_id[i] == !0 {
                let mut cycle = Vec::new();
                let mut u = i;
                loop {
                    cycle_id[u] = cycles.len();
                    cycle_pos[u] = cycle.len();
                    cycle.push(u);
                    u = next[u];
                    if u == i {
                        break;
                    }
                }
                cycles.push(cycle);
            }
        }

        let mut rev = vec![Vec::new(); size];
        for u in 0..size {
            rev[next[u]].push(u);
        }

        let mut depth_to_cycle = vec![0usize; size];
        let mut cycle_entry = vec![!0; size];
        let mut prefix_up = Vec::with_capacity(size);
        prefix_up.resize_with(size, M::unit);
        let mut q = VecDeque::new();
        for i in 0..size {
            if in_cycle[i] {
                cycle_entry[i] = i;
                prefix_up[i] = M::operate(&value[i], &M::unit());
                q.push_back(i);
            }
        }
        while let Some(u) = q.pop_front() {
            for &v in &rev[u] {
                if in_cycle[v] || cycle_entry[v] != !0 {
                    continue;
                }
                cycle_entry[v] = cycle_entry[u];
                depth_to_cycle[v] = depth_to_cycle[u] + 1;
                cycle_id[v] = cycle_id[u];
                prefix_up[v] = M::operate(&value[v], &prefix_up[u]);
                q.push_back(v);
            }
        }

        let mut cycle_prefix = Vec::with_capacity(cycles.len());
        for cycle in &cycles {
            let len = cycle.len();
            let mut pref = Vec::with_capacity(2 * len + 1);
            pref.push(M::unit());
            for i in 0..2 * len {
                let v = cycle[i % len];
                let next_val = M::operate(pref.last().unwrap(), &value[v]);
                pref.push(next_val);
            }
            cycle_prefix.push(pref);
        }

        let root = size;
        let mut edges = Vec::with_capacity(size);
        for u in 0..size {
            if in_cycle[u] {
                edges.push((u, root));
            } else {
                edges.push((u, next[u]));
            }
        }
        let graph = UndirectedSparseGraph::from_edges(size + 1, edges);
        let la = graph.level_ancestor(root);

        Self {
            depth_to_cycle,
            cycle_entry,
            cycle_id,
            cycle_pos,
            cycles,
            cycle_prefix,
            prefix_up,
            la,
        }
    }

    fn acc_to_ancestor(&self, u: usize, ancestor: usize) -> M::T {
        let inv = M::inverse(&self.prefix_up[ancestor]);
        M::operate(&self.prefix_up[u], &inv)
    }

    fn cycle_segment(&self, cycle_id: usize, start: usize, len: usize) -> M::T {
        if len == 0 {
            return M::unit();
        }
        let pref = &self.cycle_prefix[cycle_id];
        let inv = M::inverse(&pref[start]);
        M::operate(&inv, &pref[start + len])
    }

    fn cycle_acc_from(&self, entry: usize, steps: usize) -> M::T {
        if steps == 0 {
            return M::unit();
        }
        let cycle_id = self.cycle_id[entry];
        let start = self.cycle_pos[entry];
        let len = self.cycles[cycle_id].len();
        let q = steps / len;
        let r = steps % len;
        let rem = self.cycle_segment(cycle_id, start, r);
        if q == 0 {
            return rem;
        }
        let full = self.cycle_segment(cycle_id, start, len);
        let pow = M::pow(full, q);
        M::operate(&pow, &rem)
    }

    fn cycle_jump_from(&self, entry: usize, steps: usize) -> usize {
        if steps == 0 {
            return entry;
        }
        let cycle_id = self.cycle_id[entry];
        let start = self.cycle_pos[entry];
        let len = self.cycles[cycle_id].len();
        let idx = (start + steps % len) % len;
        self.cycles[cycle_id][idx]
    }

    pub fn kth(&self, u: usize, k: usize) -> (usize, M::T) {
        let depth = self.depth_to_cycle[u];
        if k <= depth {
            let ancestor = self.la.la(u, k).unwrap();
            let acc = self.acc_to_ancestor(u, ancestor);
            return (ancestor, acc);
        }
        let entry = self.cycle_entry[u];
        let acc_tree = self.acc_to_ancestor(u, entry);
        let steps = k - depth;
        let pos = self.cycle_jump_from(entry, steps);
        let acc_cycle = self.cycle_acc_from(entry, steps);
        let acc = M::operate(&acc_tree, &acc_cycle);
        (pos, acc)
    }

    /// queries: (pos, k)
    /// Return: (pos, acc)
    pub fn kth_multiple(
        &self,
        queries: impl IntoIterator<Item = (usize, usize)>,
    ) -> Vec<(usize, M::T)> {
        queries.into_iter().map(|(u, k)| self.kth(u, k)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, LinearOperation, Magma as _, Unital as _},
        num::{Zero as _, mint_basic::MInt998244353},
        tools::Xorshift,
    };

    #[test]
    fn test_kth() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(1usize..100);
            let to: Vec<_> = rng
                .random_iter(0..=n)
                .take(n)
                .map(|x| x.wrapping_sub(1))
                .collect();
            let w: Vec<MInt998244353> = rng.random_iter(..).take(n).collect();
            let mut doubling = Doubling::<AdditiveOperation<_>>::new(n, |i| (to[i], w[i]));
            let mut queries = vec![];
            let mut results = vec![];
            for s in 0..n {
                let mut pos = s;
                let mut x = MInt998244353::zero();
                for k in 0..100 {
                    if pos == !0 {
                        assert_eq!(doubling.kth(s, k), (pos, MInt998244353::zero()));
                        queries.push((s, k));
                        results.push((pos, MInt998244353::zero()));
                    } else {
                        assert_eq!(doubling.kth(s, k), (pos, x));
                        x += w[pos];
                        pos = to[pos];
                    }
                }
            }
            let doubling = Doubling::<AdditiveOperation<_>>::new(n, |i| (to[i], w[i]));
            assert_eq!(doubling.kth_multiple(queries), results);
        }
    }

    #[test]
    fn test_find() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(1usize..100);
            let to: Vec<_> = rng.random_iter(0..n).take(n).collect();
            let w: Vec<u64> = rng.random_iter(1..100).take(n).collect();
            let mut doubling = Doubling::<AdditiveOperation<_>>::new(n, |i| (to[i], w[i]));
            for _ in 0..10 {
                doubling.double();
            }
            for s in 0..n {
                let mut k = 0usize;
                let mut pos = s;
                let mut acc = 0u64;
                for x in 0u64..200 {
                    while acc + w[pos] <= x {
                        acc += w[pos];
                        pos = to[pos];
                        k += 1;
                    }
                    assert_eq!(doubling.find_last(s, |_, &v| v <= x), (k, (pos, acc)));
                    assert_eq!(
                        doubling.find_first(s, |_, &v| v > x),
                        Some((k + 1, (to[pos], acc + w[pos])))
                    );
                }
                assert_eq!(doubling.find_first(s, |_, &v| v > 1_000_000), None);
            }
        }
    }

    #[test]
    fn test_functional_graph_doubling_kth() {
        let mut rng = Xorshift::default();
        type M = LinearOperation<MInt998244353>;
        for _ in 0..200 {
            let n = rng.random(1usize..50);
            let to: Vec<_> = rng.random_iter(0..n).take(n).collect();
            let w: Vec<_> = rng
                .random_iter((1..MInt998244353::get_mod(), 0..MInt998244353::get_mod()))
                .take(n)
                .map(|(a, b)| (MInt998244353::new(a), MInt998244353::new(b)))
                .collect();
            let doubling = FunctionalGraphDoubling::<M>::new(n, |i| (to[i], w[i]));
            let mut queries = vec![];
            let mut results = vec![];
            for s in 0..n {
                let mut pos = s;
                let mut acc = M::unit();
                for k in 0..200 {
                    assert_eq!(doubling.kth(s, k), (pos, acc));
                    queries.push((s, k));
                    results.push((pos, acc));
                    acc = M::operate(&acc, &w[pos]);
                    pos = to[pos];
                }
            }
            assert_eq!(doubling.kth_multiple(queries), results);
        }
    }
}
