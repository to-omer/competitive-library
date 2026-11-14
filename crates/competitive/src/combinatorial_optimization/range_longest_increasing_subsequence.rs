use super::{WaveletMatrix, Zero};
use std::{cmp::Reverse, collections::BinaryHeap, ops::Add};

#[derive(Debug, Clone)]
pub struct RangeLongestIncreasingSubsequence {
    len: usize,
    positions: WaveletMatrix<usize>,
}

impl RangeLongestIncreasingSubsequence {
    pub fn new<T>(sequence: &[T]) -> Self
    where
        T: Ord,
    {
        let mut order: Vec<_> = (0..sequence.len()).collect();
        // Equal values receive decreasing ranks in their original order.
        order.sort_unstable_by_key(|&i| (&sequence[i], Reverse(i)));
        let mut permutation = vec![0; sequence.len()];
        for (rank, i) in order.into_iter().enumerate() {
            permutation[i] = rank;
        }
        Self::from_permutation(&permutation)
    }

    pub fn from_permutation(permutation: &[usize]) -> Self {
        let len = permutation.len();
        assert!(len <= (u32::MAX / 2) as usize);
        let mut used = vec![false; len];
        for &value in permutation {
            assert!(!used[value]);
            used[value] = true;
        }
        drop(used);
        let positions = if len <= 1 << 19 {
            Self::seaweed_blocks(permutation)
        } else {
            Self::seaweed_doubling(permutation, 1 << 16)
        };
        Self {
            len,
            positions: WaveletMatrix::new(positions),
        }
    }

    pub fn longest_length(&self, left: usize, right: usize) -> usize {
        assert!(left <= right && right <= self.len);
        right - left - self.positions.rank_lessthan(right, left..self.len)
    }

    /// Answers follow the input order; intervals are half-open.
    pub fn longest_length_batch(
        &self,
        queries: impl IntoIterator<Item = (usize, usize)>,
    ) -> Vec<usize> {
        let queries = queries.into_iter();
        let mut result = Vec::with_capacity(queries.size_hint().0);
        let counts = self
            .positions
            .rank_lessthan_batch(queries.map(|(left, right)| {
                assert!(left <= right && right <= self.len);
                result.push(right - left);
                (right, left..self.len)
            }));
        for (length, count) in result.iter_mut().zip(counts) {
            *length -= count;
        }
        result
    }

    fn seaweed_doubling(permutation: &[usize], block_size: usize) -> Vec<usize> {
        let n = permutation.len();
        let mut indices = vec![0u32; n];
        let mut row = vec![0u32; n];
        let mut column = vec![0u32; n];
        let mut links = vec![0u32; n];
        let mut buffer = vec![0u32; n];
        let mut row_order = vec![0u32; n];
        let mut column_order = vec![0u32; n];
        let mut marked = vec![false; n];
        let mut local = Vec::with_capacity(block_size);
        let mut offsets: Vec<_> = (0..n).step_by(block_size).collect();
        for (i, &value) in permutation.iter().enumerate() {
            let at = &mut offsets[value / block_size];
            indices[*at] = i as u32;
            buffer[*at] = (value % block_size) as u32;
            *at += 1;
        }
        for start in (0..n).step_by(block_size) {
            let end = (start + block_size).min(n);
            local.clear();
            local.extend(buffer[start..end].iter().map(|&x| x as usize));
            for (i, j) in Self::seaweed_blocks(&local).into_iter().enumerate() {
                if j < end - start {
                    links[start + j] = (start + i + 1) as u32;
                }
            }
        }
        let mut width = block_size;
        while width < n {
            for mid in (width..n).step_by(2 * width) {
                let start = mid - width;
                let end = (mid + width).min(n);
                // Value-separated blocks with disjoint positions concatenate without crossings.
                if indices[mid - 1] < indices[mid] {
                    continue;
                }
                let (mut i, mut j) = (start, mid);
                for (position, entry) in buffer.iter_mut().enumerate().take(end).skip(start) {
                    let index = if j == end || (i < mid && indices[i] < indices[j]) {
                        let x = i;
                        i += 1;
                        x
                    } else {
                        let x = j;
                        j += 1;
                        x
                    };
                    row_order[index] = position as u32;
                    *entry = indices[index];
                }
                indices[start..end].copy_from_slice(&buffer[start..end]);
                column[start..end].fill(end as u32);
                for i in start..mid {
                    row[row_order[i] as usize] = if links[i] == 0 {
                        0
                    } else {
                        row_order[(links[i] - 1) as usize] + 1
                    };
                    column[row_order[i] as usize] = row_order[i];
                }
                for i in mid..end {
                    row[row_order[i] as usize] = row_order[i] + 1;
                    if links[i] != 0 {
                        column[row_order[(links[i] - 1) as usize] as usize] = row_order[i];
                    }
                }
                for i in start..end {
                    row_order[i] = i as u32;
                    column_order[i] = i as u32;
                }
                let mut step = 1;
                while step < end - start {
                    for middle in (start + step..end).step_by(2 * step) {
                        let left = middle - step;
                        let right = (middle + step).min(end);
                        let (mut i, mut j) = (left, middle);
                        for entry in &mut buffer[left..right] {
                            let index = if j == right
                                || (i < middle
                                    && row[row_order[i] as usize] > row[row_order[j] as usize])
                            {
                                let x = i;
                                i += 1;
                                x
                            } else {
                                let x = j;
                                j += 1;
                                x
                            };
                            *entry = row_order[index];
                        }
                        row_order[left..right].copy_from_slice(&buffer[left..right]);
                        marked[left..middle].fill(true);
                        marked[middle..right].fill(false);
                        let mut x = left;
                        let (mut i, mut j) = (left, middle);
                        for entry in &mut buffer[left..right] {
                            if j == right
                                || (i < middle
                                    && column[column_order[i] as usize]
                                        < column[column_order[j] as usize])
                            {
                                if marked[column_order[i] as usize] {
                                    links[column_order[i] as usize] =
                                        column[column_order[i] as usize];
                                    marked[column_order[i] as usize] = false;
                                    *entry = column_order[i];
                                    i += 1;
                                } else {
                                    while !marked[row_order[x] as usize] {
                                        x += 1;
                                    }
                                    links[row_order[x] as usize] = column[column_order[i] as usize];
                                    marked[row_order[x] as usize] = false;
                                    *entry = row_order[x];
                                    i += 1;
                                }
                            } else if row[column_order[j] as usize] >= row[row_order[x] as usize] {
                                links[column_order[j] as usize] = column[column_order[j] as usize];
                                *entry = column_order[j];
                                j += 1;
                            } else {
                                marked[column_order[j] as usize] = true;
                                while !marked[row_order[x] as usize] {
                                    x += 1;
                                }
                                links[row_order[x] as usize] = column[column_order[j] as usize];
                                marked[row_order[x] as usize] = false;
                                *entry = row_order[x];
                                j += 1;
                            }
                        }
                        column_order[left..right].copy_from_slice(&buffer[left..right]);
                        column[left..right].copy_from_slice(&links[left..right]);
                    }
                    step *= 2;
                }
                links[start..end].fill(0);
                for i in start..end {
                    if column[i] < end as u32 {
                        links[column[i] as usize] = row[i];
                    }
                }
            }
            width *= 2;
        }
        drop((
            indices,
            row,
            column,
            buffer,
            row_order,
            column_order,
            marked,
        ));
        let mut positions = vec![n; n];
        for (i, x) in links.into_iter().enumerate() {
            if x != 0 {
                positions[(x - 1) as usize] = i;
            }
        }
        positions
    }

    fn seaweed_blocks(permutation: &[usize]) -> Vec<usize> {
        let n = permutation.len();
        let width = (n.isqrt() * 3 / 4).max(1);
        let blocks = n.div_ceil(width);
        let mut boundary = vec![0u32; n];
        let mut positions = vec![n; n];
        let mut heaps: Vec<BinaryHeap<u32>> = (0..blocks)
            .map(|_| BinaryHeap::with_capacity(width))
            .collect();
        let mut dirty = vec![false; blocks];
        let mut maxima = vec![0; blocks];
        let mut ends = vec![0; blocks];
        let mut pending = BinaryHeap::with_capacity(width);
        let mut active_blocks = 0;
        for (i, &value) in permutation.iter().enumerate() {
            let block = value / width;
            active_blocks = active_blocks.max(block + 1);
            let start = block * width;
            let end = ends[block].max(value + 1);
            let append_only = !dirty[block] && value >= ends[block];
            ends[block] = end;
            if dirty[block] {
                dirty[block] = false;
                pending.clear();
                // Odd heap entries are replacements not yet applied to the boundary.
                pending.extend(
                    heaps[block]
                        .iter()
                        .filter(|&&x| x & 1 != 0)
                        .map(|&x| Reverse(x >> 1)),
                );
                for entry in &mut boundary[start..end] {
                    if pending.peek().is_some_and(|smallest| *entry > smallest.0) {
                        std::mem::swap(entry, &mut pending.peek_mut().unwrap().0);
                    }
                }
            }
            boundary[value] = (i + 1) as u32;
            let mut displaced = 0;
            for entry in &mut boundary[value + 1..end] {
                if *entry > displaced {
                    std::mem::swap(entry, &mut displaced);
                }
            }
            for (offset, maximum) in maxima[block + 1..active_blocks].iter_mut().enumerate() {
                if *maximum > displaced {
                    let index = block + 1 + offset;
                    let mut largest = heaps[index].peek_mut().unwrap();
                    let next = *maximum;
                    *largest = displaced << 1 | 1;
                    displaced = next;
                    drop(largest);
                    *maximum = heaps[index].peek().copied().unwrap() >> 1;
                    dirty[index] = true;
                }
            }
            if displaced != 0 {
                positions[displaced as usize - 1] = i;
            }
            let heap = &mut heaps[block];
            if append_only {
                heap.push(((i + 1) as u32) << 1);
            } else {
                heap.clear();
                heap.extend(
                    boundary[start..end]
                        .iter()
                        .filter(|&&x| x != 0)
                        .map(|&x| x << 1),
                );
            }
            maxima[block] = heap.peek().copied().unwrap() >> 1;
        }
        positions
    }
}

/// Weights must be positive. Addition must be exact, associative, commutative,
/// and strictly preserve order, with `Zero::zero()` as its identity.
#[derive(Debug, Clone)]
pub struct WeightedRangeLongestIncreasingSubsequence<W> {
    len: usize,
    nodes: Vec<[(u32, u32); 2]>,
    subsequences: Vec<SubsequenceNode<W>>,
    roots: Vec<(u32, u32)>,
}

impl<W> WeightedRangeLongestIncreasingSubsequence<W> {
    pub fn new<T>(sequence: &[T], weights: &[W]) -> Self
    where
        T: Ord,
        W: Clone + Ord + Zero + Add<Output = W>,
    {
        let len = sequence.len();
        assert_eq!(len, weights.len());
        assert!(len < u32::MAX as usize);
        for weight in weights {
            assert!(weight > &W::zero());
        }
        let mut result = WeightedRangeLongestIncreasingSubsequenceBuilder {
            nodes: vec![vec![FunctionNode {
                left: (0, 0),
                right: (0, 0),
                first: W::zero(),
                last: W::zero(),
            }]],
            subsequences: vec![vec![SubsequenceNode {
                left: 0,
                right: 0,
                weight: W::zero(),
            }]],
            roots: Vec::with_capacity(len + 1),
        };
        let mut order: Vec<_> = (0..len).collect();
        // Later equal values rank lower, preserving strict increase.
        order.sort_unstable_by_key(|&i| (&sequence[i], Reverse(i)));
        let mut ranks = vec![0; len];
        for (rank, i) in order.into_iter().enumerate() {
            ranks[i] = rank;
        }
        let size = len.next_power_of_two();
        let mut outer = vec![(0, 0); 2 * size];
        let mut ends = vec![0; 2 * size];
        result.roots.push((0, 0));
        for (i, (&rank, weight)) in ranks.iter().zip(weights).enumerate() {
            // A later higher-value endpoint dominates every earlier lower-value endpoint.
            let mut position = size + rank;
            let mut predecessor = (0, 0);
            let mut end = 0;
            while position > 1 {
                if position & 1 != 0 && ends[position - 1] > end {
                    predecessor = result.merge(outer[position - 1], predecessor, len);
                    end = ends[position - 1];
                }
                position /= 2;
            }
            let suffix = result.push_subsequence(SubsequenceNode {
                left: 0,
                right: i as u32,
                weight: weight.clone(),
            });
            let mut position = size + rank;
            outer[position] = result.append_prefix(predecessor, len, i + 1, suffix);
            ends[position] = i + 1;
            while position > 1 {
                let right = position & 1 != 0;
                position /= 2;
                ends[position] = i + 1;
                outer[position] = if right {
                    outer[2 * position + 1]
                } else {
                    result.merge(outer[2 * position], outer[2 * position + 1], len)
                };
            }
            result.roots.push(outer[1]);
            if (i + 1).is_power_of_two() {
                result.compact(&mut outer);
            }
        }
        drop(outer);
        result.compact(&mut []);
        let mut nodes = Vec::with_capacity(result.nodes.iter().map(Vec::len).sum());
        for chunk in result.nodes {
            nodes.extend(chunk.into_iter().map(|node| [node.left, node.right]));
        }
        let mut subsequences = Vec::with_capacity(result.subsequences.iter().map(Vec::len).sum());
        for chunk in result.subsequences {
            subsequences.extend(chunk);
        }
        Self {
            len,
            nodes,
            subsequences,
            roots: result.roots,
        }
    }

    pub fn maximum_weight(&self, left: usize, right: usize) -> W
    where
        W: Clone + Zero + Add<Output = W>,
    {
        assert!(left <= right && right <= self.len);
        if left == right {
            return W::zero();
        }
        let (mut tree, mut len, mut position) = (self.roots[right], self.len, left);
        let mut weight = self.subsequences[tree.1 as usize].weight.clone();
        while tree.0 != 0 {
            let node = self.nodes[tree.0 as usize];
            let mid = len / 2;
            if position < mid {
                tree = node[0];
                len = mid;
            } else {
                tree = node[1];
                position -= mid;
                len -= mid;
            }
            weight = weight + self.subsequences[tree.1 as usize].weight.clone();
        }
        weight
    }

    /// Returns the original indices of one optimum in the half-open interval.
    pub fn maximum_weight_subsequence(&self, left: usize, right: usize) -> Vec<usize> {
        assert!(left <= right && right <= self.len);
        if left == right {
            return Vec::new();
        }
        let (mut tree, mut len, mut position) = (self.roots[right], self.len, left);
        let mut suffixes = Vec::new();
        if tree.1 != 0 {
            suffixes.push(tree.1);
        }
        while tree.0 != 0 {
            let node = self.nodes[tree.0 as usize];
            let mid = len / 2;
            if position < mid {
                tree = node[0];
                len = mid;
            } else {
                tree = node[1];
                position -= mid;
                len -= mid;
            }
            if tree.1 != 0 {
                suffixes.push(tree.1);
            }
        }
        let mut result = Vec::new();
        let mut stack = Vec::new();
        for suffix in suffixes.into_iter().rev() {
            stack.push(suffix);
            while let Some(index) = stack.pop() {
                let node = &self.subsequences[index as usize];
                if node.left == 0 {
                    result.push(node.right as usize);
                } else {
                    stack.push(node.right);
                    stack.push(node.left);
                }
            }
        }
        result
    }
}

struct WeightedRangeLongestIncreasingSubsequenceBuilder<W> {
    nodes: Vec<Vec<FunctionNode<W>>>,
    subsequences: Vec<Vec<SubsequenceNode<W>>>,
    roots: Vec<(u32, u32)>,
}

impl<W> WeightedRangeLongestIncreasingSubsequenceBuilder<W> {
    // References point to older nodes, so marking and relocation need only linear scans.
    fn compact(&mut self, outer: &mut [(u32, u32)])
    where
        W: Clone,
    {
        let node_len = (self.nodes.len() - 1) * CHUNK_SIZE + self.nodes.last().unwrap().len();
        let subsequence_len =
            (self.subsequences.len() - 1) * CHUNK_SIZE + self.subsequences.last().unwrap().len();
        let mut nodes = vec![0u32; node_len];
        let mut subsequences = vec![0u32; subsequence_len];
        for &(tree, suffix) in self.roots.iter().chain(outer.iter()) {
            nodes[tree as usize] = 1;
            subsequences[suffix as usize] = 1;
        }
        for index in (1..node_len).rev() {
            if nodes[index] == 0 {
                continue;
            }
            let node = self.node(index as u32);
            nodes[node.left.0 as usize] = 1;
            nodes[node.right.0 as usize] = 1;
            subsequences[node.left.1 as usize] = 1;
            subsequences[node.right.1 as usize] = 1;
        }
        for index in (1..subsequence_len).rev() {
            if subsequences[index] == 0 {
                continue;
            }
            let node = self.subsequence(index as u32);
            if node.left != 0 {
                subsequences[node.left as usize] = 1;
                subsequences[node.right as usize] = 1;
            }
        }
        nodes[0] = 0;
        subsequences[0] = 0;
        let mut count = 1;
        for index in 1..subsequence_len {
            if subsequences[index] == 0 {
                continue;
            }
            let mut node = self.subsequence(index as u32).clone();
            if node.left != 0 {
                node.left = subsequences[node.left as usize];
                node.right = subsequences[node.right as usize];
            }
            self.subsequences[count / CHUNK_SIZE][count % CHUNK_SIZE] = node;
            subsequences[index] = count as u32;
            count += 1;
        }
        self.subsequences.truncate(count.div_ceil(CHUNK_SIZE));
        self.subsequences
            .last_mut()
            .unwrap()
            .truncate((count - 1) % CHUNK_SIZE + 1);
        count = 1;
        for index in 1..node_len {
            if nodes[index] == 0 {
                continue;
            }
            let mut node = self.node(index as u32).clone();
            node.left = (
                nodes[node.left.0 as usize],
                subsequences[node.left.1 as usize],
            );
            node.right = (
                nodes[node.right.0 as usize],
                subsequences[node.right.1 as usize],
            );
            self.nodes[count / CHUNK_SIZE][count % CHUNK_SIZE] = node;
            nodes[index] = count as u32;
            count += 1;
        }
        self.nodes.truncate(count.div_ceil(CHUNK_SIZE));
        self.nodes
            .last_mut()
            .unwrap()
            .truncate((count - 1) % CHUNK_SIZE + 1);
        for tree in self.roots.iter_mut().chain(outer.iter_mut()) {
            *tree = (nodes[tree.0 as usize], subsequences[tree.1 as usize]);
        }
    }

    fn node(&self, index: u32) -> &FunctionNode<W> {
        &self.nodes[index as usize / CHUNK_SIZE][index as usize % CHUNK_SIZE]
    }

    fn subsequence(&self, index: u32) -> &SubsequenceNode<W> {
        &self.subsequences[index as usize / CHUNK_SIZE][index as usize % CHUNK_SIZE]
    }

    fn push_subsequence(&mut self, node: SubsequenceNode<W>) -> u32 {
        if self.subsequences.last().unwrap().len() == CHUNK_SIZE {
            self.subsequences.push(Vec::with_capacity(CHUNK_SIZE));
        }
        let index =
            (self.subsequences.len() - 1) * CHUNK_SIZE + self.subsequences.last().unwrap().len();
        let index = index.try_into().expect("too many subsequence nodes");
        self.subsequences.last_mut().unwrap().push(node);
        index
    }

    fn concatenate(&mut self, left: u32, right: u32) -> u32
    where
        W: Clone + Add<Output = W>,
    {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        self.push_subsequence(SubsequenceNode {
            left,
            right,
            weight: self.subsequence(left).weight.clone() + self.subsequence(right).weight.clone(),
        })
    }

    fn join(&mut self, left: (u32, u32), right: (u32, u32)) -> (u32, u32)
    where
        W: Clone + Add<Output = W>,
    {
        if left == right && left.0 == 0 {
            return left;
        }
        let first = self.node(left.0).first.clone() + self.subsequence(left.1).weight.clone();
        let last = self.node(right.0).last.clone() + self.subsequence(right.1).weight.clone();
        if self.nodes.last().unwrap().len() == CHUNK_SIZE {
            self.nodes.push(Vec::with_capacity(CHUNK_SIZE));
        }
        let index = (self.nodes.len() - 1) * CHUNK_SIZE + self.nodes.last().unwrap().len();
        let index = index.try_into().expect("too many function nodes");
        self.nodes.last_mut().unwrap().push(FunctionNode {
            left,
            right,
            first,
            last,
        });
        (index, 0)
    }

    fn append(&mut self, tree: (u32, u32), suffix: u32) -> (u32, u32)
    where
        W: Clone + Add<Output = W>,
    {
        (tree.0, self.concatenate(tree.1, suffix))
    }

    fn append_prefix(&mut self, tree: (u32, u32), len: usize, end: usize, suffix: u32) -> (u32, u32)
    where
        W: Clone + Add<Output = W>,
    {
        if end == 0 {
            return tree;
        }
        if len <= end {
            return self.append(tree, suffix);
        }
        let node = self.node(tree.0);
        let (left, right) = (node.left, node.right);
        let left = self.append(left, tree.1);
        let right = self.append(right, tree.1);
        let mid = len / 2;
        let left = self.append_prefix(left, mid, end, suffix);
        let right = self.append_prefix(right, len - mid, end.saturating_sub(mid), suffix);
        self.join(left, right)
    }

    // The high-value envelope wins strictly on a prefix; ties select the low side.
    fn merge(&mut self, low: (u32, u32), high: (u32, u32), len: usize) -> (u32, u32)
    where
        W: Clone + Ord + Add<Output = W>,
    {
        if high == (0, 0) || low == high {
            return low;
        }
        if low == (0, 0) {
            return high;
        }
        let low_node = self.node(low.0);
        let high_node = self.node(high.0);
        let low_weight = &self.subsequence(low.1).weight;
        let high_weight = &self.subsequence(high.1).weight;
        if high_node.last.clone() + high_weight.clone() > low_node.last.clone() + low_weight.clone()
        {
            return high;
        }
        if high_node.first.clone() + high_weight.clone()
            <= low_node.first.clone() + low_weight.clone()
        {
            return low;
        }
        let mid = len / 2;
        let high_wins = self.node(high_node.left.0).last.clone()
            + self.subsequence(high_node.left.1).weight.clone()
            + high_weight.clone()
            > self.node(low_node.left.0).last.clone()
                + self.subsequence(low_node.left.1).weight.clone()
                + low_weight.clone();
        let (low_left, low_right) = (low_node.left, low_node.right);
        let (high_left, high_right) = (high_node.left, high_node.right);
        if high_wins {
            let left = self.append(high_left, high.1);
            let low_right = self.append(low_right, low.1);
            let high_right = self.append(high_right, high.1);
            let right = self.merge(low_right, high_right, len - mid);
            self.join(left, right)
        } else {
            let low_left = self.append(low_left, low.1);
            let high_left = self.append(high_left, high.1);
            let left = self.merge(low_left, high_left, mid);
            let right = self.append(low_right, low.1);
            self.join(left, right)
        }
    }
}

// Fixed-size chunks avoid copying the persistent arenas when they grow.
const CHUNK_SIZE: usize = 1 << 12;

#[derive(Debug, Clone)]
struct FunctionNode<W> {
    left: (u32, u32),
    right: (u32, u32),
    first: W,
    last: W,
}

#[derive(Debug, Clone)]
struct SubsequenceNode<W> {
    left: u32,
    // An original index for a leaf; the right child otherwise.
    right: u32,
    weight: W,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        Xorshift,
        testutil::{exhaustive_sequences, sequence_cases},
    };

    #[test]
    fn test_range_longest_increasing_subsequence() {
        assert!(
            RangeLongestIncreasingSubsequence::from_permutation(&[])
                .longest_length_batch([])
                .is_empty()
        );
        let mut rng = Xorshift::default();
        for values in
            exhaustive_sequences(0..4, 0..=7).chain(sequence_cases(&mut rng, -256..=256, 0..=256))
        {
            let sequence = RangeLongestIncreasingSubsequence::new(&values);
            let mut queries = Vec::new();
            let mut answers = Vec::new();
            for left in 0..=values.len() {
                let mut ending = vec![0; values.len()];
                let mut expected = 0;
                assert_eq!(sequence.longest_length(left, left), 0);
                queries.push((left, left));
                answers.push(0);
                for right in left..values.len() {
                    ending[right] = 1;
                    for j in left..right {
                        if values[j] < values[right] {
                            ending[right] = ending[right].max(ending[j] + 1);
                        }
                    }
                    expected = expected.max(ending[right]);
                    queries.push((left, right + 1));
                    answers.push(expected);
                    assert_eq!(
                        sequence.longest_length(left, right + 1),
                        expected,
                        "values={values:?}, left={left}, right={right}"
                    );
                }
            }
            assert_eq!(sequence.longest_length_batch(queries), answers);
        }

        let n = (1 << 19) + 1;
        let split = 1 << 16;
        let mut permutation: Vec<_> = (0..n).collect();
        permutation.swap(split - 1, split);
        let sequence = RangeLongestIncreasingSubsequence::from_permutation(&permutation);
        let boundaries = [0, split - 1, split, split + 1, n - 1, n];
        let mut queries = Vec::new();
        let mut answers = Vec::new();
        for (i, &left) in boundaries.iter().enumerate() {
            for &right in &boundaries[i..] {
                let expected = right - left - usize::from(left < split && split < right);
                assert_eq!(sequence.longest_length(left, right), expected);
                queries.push((left, right));
                answers.push(expected);
            }
        }
        assert_eq!(sequence.longest_length_batch(queries), answers);
    }

    fn check_weighted(values: &[i32], weights: &[u64]) {
        let sequence = WeightedRangeLongestIncreasingSubsequence::new(values, weights);
        for left in 0..=values.len() {
            let mut ending = vec![0; values.len()];
            let mut expected = 0;
            for right in left..=values.len() {
                if right > left {
                    let i = right - 1;
                    ending[i] = weights[i];
                    for j in left..i {
                        if values[j] < values[i] {
                            ending[i] = ending[i].max(ending[j] + weights[i]);
                        }
                    }
                    expected = expected.max(ending[i]);
                }
                assert_eq!(sequence.maximum_weight(left, right), expected);
                let indices = sequence.maximum_weight_subsequence(left, right);
                assert!(indices.iter().all(|&i| (left..right).contains(&i)));
                assert!(
                    indices
                        .windows(2)
                        .all(|p| p[0] < p[1] && values[p[0]] < values[p[1]])
                );
                let weight: u64 = indices.iter().map(|&i| weights[i]).sum();
                assert_eq!(weight, expected);
            }
        }
    }

    #[test]
    fn test_weighted_range_longest_increasing_subsequence() {
        for encoded in exhaustive_sequences(0..9, 0..=5) {
            let values: Vec<_> = encoded.iter().map(|&x| x / 3).collect();
            let weights: Vec<_> = encoded.iter().map(|&x| (x % 3 + 1) as u64).collect();
            check_weighted(&values, &weights);
        }
        let mut rng = Xorshift::default();
        let cases: Vec<_> = sequence_cases(&mut rng, -192..=192, 0..=192).collect();
        for values in cases {
            let weights: Vec<_> = (0..values.len())
                .map(|_| rng.random(1..=1_000_000_000_000u64))
                .collect();
            check_weighted(&values, &weights);
        }
        for values in sequence_cases(&mut rng, -512..=512, 0..=512) {
            let unweighted = RangeLongestIncreasingSubsequence::new(&values);
            let weights = vec![1; values.len()];
            let weighted = WeightedRangeLongestIncreasingSubsequence::new(&values, &weights);
            for left in 0..=values.len() {
                for right in left..=values.len() {
                    assert_eq!(
                        weighted.maximum_weight(left, right),
                        unweighted.longest_length(left, right) as u64
                    );
                }
            }
        }
        for n in [8191, 8192, 8193] {
            let values: Vec<_> = (0..n).collect();
            let weights: Vec<_> = (1..=n as u64).collect();
            let sequence = WeightedRangeLongestIncreasingSubsequence::new(&values, &weights);
            let total = (n * (n + 1) / 2) as u64;
            for left in 0..=n {
                let prefix = (left * (left + 1) / 2) as u64;
                assert_eq!(sequence.maximum_weight(0, left), prefix);
                assert_eq!(sequence.maximum_weight(left, n), total - prefix);
                let right = (left + 17).min(n);
                assert_eq!(
                    sequence.maximum_weight_subsequence(left, right),
                    (left..right).collect::<Vec<_>>()
                );
            }
            assert_eq!(sequence.maximum_weight_subsequence(0, n), values);
        }
    }
}
