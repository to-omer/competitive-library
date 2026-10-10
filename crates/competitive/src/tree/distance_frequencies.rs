use super::{Convolve998244353, ConvolveSteps, Graph, MInt, U64Convolve, UndirectedSparseGraph};

impl UndirectedSparseGraph {
    /// Counts ordered vertex pairs at each distance, including the n pairs at distance zero.
    /// The graph must be a tree. Uses O(n log n) time and O(n) auxiliary space.
    pub fn distance_frequencies(&self) -> Vec<u64> {
        let n = self.vertices_size();
        let mut table = vec![0u64; n];
        if n == 0 {
            return table;
        }
        table[0] = n as u64;

        let root = (0..n).max_by_key(|&u| self.neighbors(u).len()).unwrap();
        if self.neighbors(root).len() <= 2 {
            for (d, a) in table.iter_mut().enumerate().skip(1) {
                *a = 2 * (n - d) as u64;
            }
            return table;
        }
        // BFS numbering makes each vertex's children a contiguous interval.
        let mut order = Vec::with_capacity(n);
        let mut children = Vec::with_capacity(n + 1);
        order.push((root, n));
        children.push(1);
        for u in 0..n {
            let (v, parent) = order[u];
            order.extend(
                self.neighbors(v)
                    .filter(|a| a.to != parent)
                    .map(|a| (a.to, v)),
            );
            children.push(order.len());
        }
        drop(order);
        let mut height = vec![1usize; n];
        let mut heavy = vec![n; n];
        for u in (0..n).rev() {
            for v in children[u]..children[u + 1] {
                if height[u] < height[v] + 1 {
                    height[u] = height[v] + 1;
                    heavy[u] = v;
                }
            }
        }
        let mut end = 1;
        for a in table.iter_mut().take(height[0]).skip(1) {
            let next = children[end];
            *a = (next - end) as u64;
            end = next;
        }
        // Each vertex at depth >= d has exactly one ancestor at distance d.
        let mut sum = 0;
        for a in table[1..height[0]].iter_mut().rev() {
            sum += *a;
            *a = 2 * sum;
        }
        // Bound the direct algorithm's product count, aggregating leaf children first.
        let mut work = 0usize;
        if height[0] > 128 {
            for u in 0..n {
                let mut light = 0;
                let mut leaves = false;
                for (v, &h) in height
                    .iter()
                    .enumerate()
                    .take(children[u + 1])
                    .skip(children[u])
                {
                    if v != heavy[u] {
                        if h == 1 {
                            leaves = true;
                        } else {
                            light += h;
                        }
                    }
                }
                work = work
                    .saturating_add((height[u] - 1).saturating_mul(light + usize::from(leaves)));
                if work > n.saturating_mul(128) {
                    break;
                }
            }
        }
        let direct = work <= n.saturating_mul(128);
        let mut offset = vec![0; n];
        let mut heads = vec![0];
        let mut size = height[0];
        for u in 0..n {
            for v in children[u]..children[u + 1] {
                if v == heavy[u] {
                    offset[v] = offset[u] + 1;
                } else {
                    offset[v] = size;
                    size += height[v];
                    if !direct && height[v] > 1 {
                        heads.push(v);
                    }
                }
            }
        }
        if direct {
            distance_frequencies_direct(&children, &height, &heavy, &offset, &mut table);
            return table;
        }

        // Longest-child paths share suffixes; the disjoint head arrays contain n coefficients.
        let mut counts = vec![1u64; n];
        let mut branch_begin = vec![0; n + 1];
        for u in 0..n {
            let len = (children[u]..children[u + 1])
                .filter(|&v| v != heavy[u])
                .map(|v| height[v])
                .max()
                .unwrap_or(0);
            branch_begin[u + 1] = branch_begin[u] + len;
        }
        let mut branches = vec![0u64; branch_begin[n]];
        for u in (0..n).rev() {
            let branch = &mut branches[branch_begin[u]..branch_begin[u + 1]];
            if branch.is_empty() {
                continue;
            }
            let mut leaves = 0u64;
            let mut light = 0;
            let mut first = n;
            let mut second = n;
            for (v, &h) in height
                .iter()
                .enumerate()
                .take(children[u + 1])
                .skip(children[u])
            {
                if v == heavy[u] {
                    continue;
                }
                if h == 1 {
                    leaves += 1;
                } else {
                    light += 1;
                    if first == n {
                        first = v;
                    } else {
                        second = v;
                    }
                    for (a, &b) in branch.iter_mut().zip(&counts[offset[v]..offset[v] + h]) {
                        *a += b;
                    }
                }
            }
            for (a, &b) in table[2..].iter_mut().zip(branch.iter()) {
                *a += 2 * leaves * b;
            }
            table[2] += leaves * leaves.saturating_sub(1);
            if light == 2 {
                let [v, w] = [first, second];
                let product = distance_frequencies_convolve(
                    &counts[offset[v]..offset[v] + height[v]],
                    &counts[offset[w]..offset[w] + height[w]],
                );
                for (a, b) in table[2..].iter_mut().zip(product) {
                    *a += 2 * b;
                }
            } else if light > 2 {
                for (a, b) in table[2..]
                    .iter_mut()
                    .zip(distance_frequencies_convolve(branch, branch))
                {
                    *a = a.wrapping_add(b);
                }
                for v in children[u]..children[u + 1] {
                    if v == heavy[u] || height[v] == 1 {
                        continue;
                    }
                    let child = &counts[offset[v]..offset[v] + height[v]];
                    for (a, b) in table[2..]
                        .iter_mut()
                        .zip(distance_frequencies_convolve(child, child))
                    {
                        *a = a.wrapping_sub(b);
                    }
                }
            }
            branch[0] += leaves;
            for (d, &a) in branch.iter().enumerate() {
                counts[offset[u] + d + 1] += a;
            }
        }

        drop(counts);
        drop(offset);
        drop(children);
        let mut difference = vec![0u64; n + 1];
        for head in heads {
            let len = height[head];
            let mut path = Vec::new();
            let mut u = head;
            for i in 0..len {
                let branch = &branches[branch_begin[u]..branch_begin[u + 1]];
                if !branch.is_empty() {
                    // Ancestor pairs are already counted; add distances to later path vertices.
                    for (d, &a) in branch.iter().enumerate() {
                        difference[d + 2] = difference[d + 2].wrapping_add(2 * a);
                        difference[d + len - i + 1] =
                            difference[d + len - i + 1].wrapping_sub(2 * a);
                    }
                    path.push((i, branch));
                }
                u = heavy[u];
            }
            distance_frequencies_path(&path, &mut table);
        }
        let mut sum = 0u64;
        for (a, d) in table.iter_mut().zip(difference) {
            sum = sum.wrapping_add(d);
            *a = a.wrapping_add(sum);
        }
        table
    }
}

fn distance_frequencies_direct(
    children: &[usize],
    height: &[usize],
    heavy: &[usize],
    offset: &[usize],
    table: &mut [u64],
) {
    let n = table.len();
    let mut counts = vec![1u64; n];
    for u in (0..n).rev() {
        let mut leaves = 0u64;
        for v in children[u]..children[u + 1] {
            if v == heavy[u] {
                continue;
            }
            if height[v] == 1 {
                leaves += 1;
                continue;
            }
            for d in 0..height[v] {
                let b = 2 * counts[offset[v] + d];
                for (a, &c) in table[d + 2..]
                    .iter_mut()
                    .zip(&counts[offset[u] + 1..offset[u] + height[u]])
                {
                    *a += b * c;
                }
            }
            for d in 0..height[v] {
                counts[offset[u] + d + 1] += counts[offset[v] + d];
            }
        }
        if leaves != 0 {
            for (a, &b) in table[2..]
                .iter_mut()
                .zip(&counts[offset[u] + 1..offset[u] + height[u]])
            {
                *a += 2 * leaves * b;
            }
            table[2] += leaves * (leaves - 1);
            counts[offset[u] + 1] += leaves;
        }
    }
}

fn distance_frequencies_path(path: &[(usize, &[u64])], table: &mut [u64]) {
    if path.len() < 2 {
        return;
    }
    if path.len() <= 4 || path.iter().map(|(_, a)| a.len()).sum::<usize>() <= 64 {
        for (k, (i, a)) in path.iter().enumerate() {
            for (j, b) in &path[k + 1..] {
                for (value, product) in table[j - i + 2..]
                    .iter_mut()
                    .zip(distance_frequencies_convolve(a, b))
                {
                    *value += 2 * product;
                }
            }
        }
        return;
    }
    let first = path[0].0;
    let len = path.last().unwrap().0 - first + 1;
    let max_len = path.iter().map(|(_, a)| a.len()).max().unwrap();
    let mut forward = vec![0u64; len + max_len - 1];
    let mut backward = vec![0u64; len + max_len - 1];
    for (i, a) in path {
        for (d, &a) in a.iter().enumerate() {
            forward[i - first + d] += a;
            backward[len - 1 - (i - first) + d] += a;
        }
    }
    while forward.last() == Some(&0) {
        forward.pop();
    }
    while backward.last() == Some(&0) {
        backward.pop();
    }
    // P+ P- contains the desired pairs, self products, and reverse-direction pairs.
    let skip = usize::from(max_len == 1);
    for (a, b) in table[2 + skip..].iter_mut().zip(
        distance_frequencies_convolve(&forward, &backward)
            .into_iter()
            .skip(len - 1 + skip),
    ) {
        *a = a.wrapping_add(2 * b);
    }
    if max_len == 1 {
        return;
    }
    drop(forward);
    drop(backward);
    for (_, a) in path {
        if a.len() <= 9 {
            for (d, &x) in a.iter().enumerate() {
                for (b, &y) in table[d + 2..].iter_mut().zip(a.iter()) {
                    *b = b.wrapping_sub(2 * x * y);
                }
            }
        }
    }
    // Short profiles can only contribute reverse terms between nearby positions.
    if max_len <= 9 {
        for (k, (i, a)) in path.iter().enumerate() {
            for (j, b) in path[k + 1..]
                .iter()
                .take_while(|(j, _)| j - i < 2 * max_len - 1)
            {
                for (d, &x) in a.iter().enumerate() {
                    for (e, &y) in b.iter().enumerate().skip((j - i).saturating_sub(d)) {
                        let value = &mut table[d + e + 2 - (j - i)];
                        *value = value.wrapping_sub(2 * x * y);
                    }
                }
            }
        }
        return;
    }

    let mut order: Vec<_> = (0..path.len()).collect();
    order.sort_unstable_by_key(|&i| (path[i].1.len(), i));
    let mut rank = vec![0; path.len()];
    let mut begin = vec![0; path.len() + 1];
    for (k, &i) in order.iter().enumerate() {
        rank[i] = k;
    }
    for (k, (_, a)) in path.iter().enumerate() {
        begin[k + 1] = begin[k] + a.len();
    }
    let mut diagonals = None;
    for (k, &index) in order.iter().enumerate() {
        let (i, a) = path[index];
        let degree = a.len() - 1;
        if degree == 0 {
            continue;
        }
        let previous_degree = if k == 0 {
            0
        } else {
            path[order[k - 1]].1.len() - 1
        };
        // Only exponents -degree .. previous_degree-1 can survive multiplication by a.
        let mut correction = vec![0u64; degree + previous_degree];
        if previous_degree <= 8 {
            let radius = degree + previous_degree;
            let left = path.partition_point(|(j, _)| *j < i.saturating_sub(radius));
            for (j, &(position, b)) in path.iter().enumerate().skip(left) {
                if position > i + radius {
                    break;
                }
                if rank[j] >= k {
                    continue;
                }
                let distance = position.abs_diff(i);
                for (d, &value) in b.iter().enumerate().skip(distance.saturating_sub(degree)) {
                    correction[degree + d - distance] += value;
                }
            }
        } else {
            let diagonals = diagonals.get_or_insert_with(|| {
                [false, true]
                    .map(|reverse| DistanceFrequencyDiagonal::new(path, &begin, &rank, k, reverse))
            });
            for (d, value) in correction.iter_mut().enumerate() {
                let exponent = d as isize - degree as isize;
                *value = diagonals[0].query(i - first, exponent, begin[index])
                    + diagonals[1].query(len - 1 - (i - first), exponent, begin[index]);
            }
        }
        let nonzero_begin = correction
            .iter()
            .position(|&a| a != 0)
            .unwrap_or(correction.len());
        let nonzero_end = correction
            .iter()
            .rposition(|&a| a != 0)
            .map_or(nonzero_begin, |i| i + 1);
        let combined_begin = nonzero_begin.min(degree);
        // Fold a^2 into a * (correction + x^degree a) when the transform stays the same size.
        let fuse = a.len() > 9
            && nonzero_begin < nonzero_end
            && a.len().saturating_mul(nonzero_end - nonzero_begin) > 4096
            && (3 * degree + 1 - combined_begin).next_power_of_two()
                == (degree + nonzero_end - nonzero_begin).next_power_of_two();
        if a.len() > 9 && !fuse {
            for (b, c) in table[2..]
                .iter_mut()
                .zip(distance_frequencies_convolve(a, a))
            {
                *b = b.wrapping_sub(2 * c);
            }
        }
        if nonzero_begin < nonzero_end {
            let (product, shift) = if fuse {
                let mut combined = vec![0; 2 * degree + 1 - combined_begin];
                combined[nonzero_begin - combined_begin..nonzero_end - combined_begin]
                    .copy_from_slice(&correction[nonzero_begin..nonzero_end]);
                for (c, &value) in combined[degree - combined_begin..].iter_mut().zip(a) {
                    *c += value;
                }
                (distance_frequencies_convolve(a, &combined), combined_begin)
            } else {
                (
                    distance_frequencies_convolve(a, &correction[nonzero_begin..nonzero_end]),
                    nonzero_begin,
                )
            };
            for (b, c) in table[2 + shift.saturating_sub(degree)..]
                .iter_mut()
                .zip(product.into_iter().skip(degree.saturating_sub(shift)))
            {
                *b = b.wrapping_sub(2 * c);
            }
        }
        if k + 1 < order.len()
            && let Some(diagonals) = &mut diagonals
        {
            diagonals[0].insert(i - first, begin[index], a);
            diagonals[1].insert(len - 1 - (i - first), begin[index], a);
        }
    }
}

fn distance_frequencies_convolve(a: &[u64], b: &[u64]) -> Vec<u64> {
    if a.len().saturating_mul(b.len()) > 4096 && a.len().min(b.len()) > 16 {
        // These are nonnegative counts. The bound makes reduction modulo p exact.
        let stats = |values: &[u64]| {
            values.iter().try_fold((0u64, 0u64), |(sum, max), &value| {
                let sum = sum.checked_add(value)?;
                (sum < 998244353).then_some((sum, max.max(value)))
            })
        };
        if let Some((sum_a, max_a)) = stats(a)
            && let Some((sum_b, max_b)) = stats(b)
            && sum_a.saturating_mul(max_b).min(sum_b.saturating_mul(max_a)) < 998244353
        {
            let input = a.iter().copied().map(MInt::from).collect();
            // Raw square is limited to this modulus's 2^23-point transform.
            let product = if std::ptr::eq(a, b) && a.len() <= 1 << 22 {
                Convolve998244353::square(input, a.len() * 2 - 1)
            } else {
                Convolve998244353::convolve(input, b.iter().copied().map(MInt::from).collect())
            };
            return product
                .into_iter()
                .map(|value| value.inner() as u64)
                .collect();
        }
    }
    U64Convolve::convolve(a.to_vec(), b.to_vec())
}

// Fenwick groups share storage, while each update stays within its diagonal.
struct DistanceFrequencyDiagonal {
    begin: Vec<usize>,
    slot: Vec<usize>,
    total: Vec<u64>,
    bit: Vec<u64>,
}

impl DistanceFrequencyDiagonal {
    fn new(
        path: &[(usize, &[u64])],
        profile_begin: &[usize],
        rank: &[usize],
        processed: usize,
        reverse: bool,
    ) -> Self {
        let first = path[0].0;
        let last = path.last().unwrap().0;
        let max_len = path.iter().map(|(_, a)| a.len()).max().unwrap();
        let mut begin = vec![0; last - first + max_len + 1];
        for &(i, a) in path {
            let position = if reverse { last - i } else { i - first };
            for size in &mut begin[position + 1..position + a.len() + 1] {
                *size += 1;
            }
        }
        for i in 1..begin.len() {
            begin[i] += begin[i - 1];
        }
        let mut next = begin.clone();
        let mut slot = vec![0; *profile_begin.last().unwrap()];
        let mut values = vec![0; slot.len()];
        let mut total = vec![0; begin.len() - 1];
        for k in 0..path.len() {
            let k = if reverse { path.len() - 1 - k } else { k };
            let (i, a) = path[k];
            let position = if reverse { last - i } else { i - first };
            for (d, &value) in a.iter().enumerate() {
                let group = position + d;
                let index = next[group];
                next[group] += 1;
                slot[profile_begin[k] + d] = index;
                if rank[k] < processed {
                    values[index] = value;
                    total[group] += value;
                }
            }
        }
        for range in begin.windows(2) {
            let (base, size) = (range[0], range[1] - range[0]);
            for i in 1..=size {
                let j = i + (i & i.wrapping_neg());
                if j <= size {
                    let value = values[base + i - 1];
                    values[base + j - 1] += value;
                }
            }
        }
        Self {
            begin,
            slot,
            total,
            bit: values,
        }
    }

    fn query(&self, position: usize, exponent: isize, profile_begin: usize) -> u64 {
        let Some(group) = position.checked_add_signed(exponent) else {
            return 0;
        };
        if group >= self.total.len() {
            return 0;
        }
        // Nonpositive exponents cannot contain coefficients from the opposite side.
        if exponent <= 0 {
            self.total[group]
        } else {
            let base = self.begin[group];
            let mut i = self.slot[profile_begin + exponent as usize] - base;
            let mut sum = 0;
            while i != 0 {
                sum += self.bit[base + i - 1];
                i &= i - 1;
            }
            sum
        }
    }

    fn insert(&mut self, position: usize, profile_begin: usize, values: &[u64]) {
        for (d, &value) in values.iter().enumerate() {
            if value != 0 {
                self.total[position + d] += value;
                let base = self.begin[position + d];
                let size = self.begin[position + d + 1] - base;
                let mut i = self.slot[profile_begin + d] - base + 1;
                while i <= size {
                    self.bit[base + i - 1] += value;
                    i += i & i.wrapping_neg();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tools::{
            Xorshift,
            testutil::{exhaustive_sequences, sample_usize, structured_sequences},
        },
        tree::{MixedTree, PathTree, StarTree},
    };

    fn check(g: UndirectedSparseGraph) {
        let n = g.vertices_size();
        let result = g.distance_frequencies();
        let mut expected = vec![0u64; n];
        for u in 0..n {
            let mut depth = vec![n; n];
            let mut queue = vec![u];
            depth[u] = 0;
            for i in 0..n {
                let v = queue[i];
                expected[depth[v]] += 1;
                for a in g.neighbors(v) {
                    if depth[a.to] == n {
                        depth[a.to] = depth[v] + 1;
                        queue.push(a.to);
                    }
                }
            }
        }
        assert_eq!(result, expected, "{:?}", g.edges);
    }

    #[test]
    fn test_distance_frequencies() {
        let mut rng = Xorshift::default();
        // All labelled trees through eight vertices, via every Prüfer sequence.
        for n in 0usize..=8 {
            for sequence in exhaustive_sequences(0..n, n.saturating_sub(2)..=n.saturating_sub(2)) {
                let mut degree = vec![1; n];
                for &u in &sequence {
                    degree[u] += 1;
                }
                let mut edges = Vec::new();
                for u in sequence {
                    let v = (0..n).find(|&v| degree[v] == 1).unwrap();
                    edges.push((u, v));
                    degree[u] -= 1;
                    degree[v] -= 1;
                }
                let rest: Vec<_> = (0..n).filter(|&v| degree[v] == 1).collect();
                if rest.len() == 2 {
                    edges.push((rest[0], rest[1]));
                }
                check(UndirectedSparseGraph::from_edges(n, edges));
            }
        }
        for _ in 0..200 {
            check(rng.random(MixedTree(1usize..100)));
        }
        // These exceed the direct-work budget with two or more non-leaf light children.
        for arms in 3..=4 {
            for leaves in 0..=2 {
                let n = 1 + 256 * arms + leaves;
                let edges = (1..n)
                    .map(|u| {
                        let parent = if u > 256 * arms || (u - 1) % 256 == 0 {
                            0
                        } else {
                            u - 1
                        };
                        (parent, u)
                    })
                    .collect();
                check(UndirectedSparseGraph::from_edges(n, edges));
            }
        }
        for n in sample_usize(&mut rng, 16, 0..=1024, 32) {
            check(rng.random(PathTree(n)));
            check(rng.random(StarTree(n)));
            check(rng.random(MixedTree(n)));
            check(UndirectedSparseGraph::from_edges(
                n,
                (1..n).map(|u| ((u - 1) / 2, u)).collect(),
            ));
            for branch_len in sample_usize(&mut rng, 2, 1..=n.max(1), 4) {
                let stem = n / 2;
                let edges = (1..n)
                    .map(|u| {
                        let parent = if u < stem || (u - stem) % branch_len != 0 {
                            u - 1
                        } else {
                            (u - stem) / branch_len % stem.max(1)
                        };
                        (parent, u)
                    })
                    .collect();
                check(UndirectedSparseGraph::from_edges(n, edges));
            }
        }
    }

    #[test]
    fn test_distance_frequencies_path() {
        let mut rng = Xorshift::default();
        let check = |path: &[(usize, Vec<u64>)]| {
            let size = path.last().map_or(0, |(i, _)| *i)
                + 2 * path.iter().map(|(_, a)| a.len()).max().unwrap_or(0)
                + 2;
            let mut result = vec![0; size];
            let mut expected = vec![0; size];
            for (k, (i, a)) in path.iter().enumerate() {
                for (j, b) in &path[k + 1..] {
                    for (x, &a) in a.iter().enumerate() {
                        for (y, &b) in b.iter().enumerate() {
                            expected[j - i + x + y + 2] += 2 * a * b;
                        }
                    }
                }
            }
            let path: Vec<_> = path.iter().map(|(i, a)| (*i, a.as_slice())).collect();
            distance_frequencies_path(&path, &mut result);
            assert_eq!(result, expected, "{path:?}");
        };
        // All paths with <=4 profiles, lengths 1..=2, coefficients 0..=2,
        // first position 0..=2, and gaps 1..=2.
        for lengths in exhaustive_sequences(1usize..=2, 0..=4) {
            let total = lengths.iter().sum();
            let gaps = lengths.len().saturating_sub(1);
            for gaps in exhaustive_sequences(1usize..=2, gaps..=gaps) {
                for first in 0..=2 {
                    for values in exhaustive_sequences(0u64..=2, total..=total) {
                        let mut values = values.into_iter();
                        let mut position = first;
                        let path: Vec<_> = lengths
                            .iter()
                            .enumerate()
                            .map(|(i, &len)| {
                                if i != 0 {
                                    position += gaps[i - 1];
                                }
                                (position, values.by_ref().take(len).collect())
                            })
                            .collect();
                        check(&path);
                    }
                }
            }
        }

        // Vary every shape dimension beyond the exhaustive domain. The coefficient
        // budget keeps the independent quadratic oracle affordable for 1,000 trials.
        for count in sample_usize(&mut rng, 4, 0..=128, 1000) {
            let max_len = (1024 / count.max(1)).min(513);
            let mut position = rng.random(0usize..=1024);
            let path: Vec<_> = (0..count)
                .map(|_| {
                    position += rng.random(1..=max_len);
                    let len = rng.random(1..=max_len);
                    (position, rng.random_iter(0u64..=100).take(len).collect())
                })
                .collect();
            check(&path);
        }

        // Cross both direct-path cutoffs, including constant profiles in the fast path,
        // and the degree-8 correction cutoff. Shifted, uneven gaps exercise both directions.
        for count in (3..=5).chain(63..=65) {
            for len in 1..=10 {
                for first in 0..=2 {
                    let mut position = first;
                    let path: Vec<_> = (0..count)
                        .map(|i| {
                            if i != 0 {
                                position += i % 3 + 1;
                            }
                            (position, rng.random_iter(0u64..=100).take(len).collect())
                        })
                        .collect();
                    check(&path);
                }
            }
        }
        // Mixed short/long profiles in arbitrary, ascending and descending length order.
        for n in sample_usize(&mut rng, 10, 1..=513, 64) {
            let count = rng.random(5usize..=8);
            let mut lengths: Vec<_> = rng.random_iter(1usize..=n).take(count).collect();
            lengths[0] = n;
            lengths[1] = 1;
            lengths[2] = n.min(9);
            for order in 0..3 {
                match order {
                    1 => lengths.sort_unstable(),
                    2 => lengths.reverse(),
                    _ => {}
                }
                let mut position = rng.random(0usize..=2);
                let path: Vec<_> = lengths
                    .iter()
                    .map(|&len| {
                        position += rng.random(1..=n / 3 + 1);
                        (position, rng.random_iter(0u64..=100).take(len).collect())
                    })
                    .collect();
                check(&path);
            }
        }
    }

    #[test]
    fn test_distance_frequencies_convolve() {
        let mut rng = Xorshift::default();
        let check = |a: &[u64], b: &[u64]| {
            let mut expected = vec![0u64; (a.len() + b.len()).saturating_sub(1)];
            for (i, &x) in a.iter().enumerate() {
                for (j, &y) in b.iter().enumerate() {
                    expected[i + j] = expected[i + j].wrapping_add(x.wrapping_mul(y));
                }
            }
            let result = distance_frequencies_convolve(a, b);
            assert_eq!(result, expected, "{a:?}, {b:?}");
        };
        // Every pair of coefficient sequences in 0..=2 through length four.
        let small: Vec<_> = exhaustive_sequences(0u64..=2, 0..=4).collect();
        for a in &small {
            for b in &small {
                check(a, b);
            }
            check(a, a);
        }
        // Independent lengths and both small counts and full-width wrapping values.
        let lengths = sample_usize(&mut rng, 17, 0..=512, 1000)
            .into_iter()
            .chain(sample_usize(&mut rng, 0, 513..=2049, 16));
        for n in lengths {
            let m = rng.random(0usize..=n.max(512));
            let limit = if rng.gen_bool(0.5) {
                rng.random(1u64..=1000)
            } else {
                u64::MAX
            };
            let a: Vec<_> = rng.random_iter(0..=limit).take(n).collect();
            let b: Vec<_> = rng.random_iter(0..=limit).take(m).collect();
            check(&a, &b);
            check(&a, &a);
        }
        // Cross the min-length 16 and product 4096 cutoffs with nonuniform and sparse inputs.
        for n in sample_usize(&mut rng, 17, 1..=257, 16) {
            let inputs: Vec<_> = structured_sequences(&mut rng, 0u64..=2, [n]).collect();
            for a in &inputs {
                check(a, a);
            }
            let boundary = 4096 / n;
            for m in boundary - 1..=boundary + 1 {
                for a in &inputs {
                    let mut b: Vec<_> = rng.random_iter(0u64..=2).take(m).collect();
                    b[m - 1] = 2;
                    let bound = (a.iter().sum::<u64>() * b.iter().max().unwrap())
                        .min(b.iter().sum::<u64>() * a.iter().max().unwrap());
                    let scale = 998244353 / bound.max(1);
                    for scale in scale.saturating_sub(1)..=scale + 1 {
                        let b: Vec<_> = b.iter().map(|&value| value * scale).collect();
                        check(a, &b);
                        check(&b, a);
                    }
                }
                // A unit impulse makes the largest coefficient exactly p-1, p, or p+1.
                for value in 998244352..=998244354 {
                    let mut a = vec![0; n];
                    let mut b = vec![0; m];
                    a[rng.random(0..n)] = 1;
                    b[rng.random(0..m)] = value;
                    check(&a, &b);
                    check(&b, &a);
                }
            }
        }
    }
}
