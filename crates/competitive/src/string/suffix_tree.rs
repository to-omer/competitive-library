use super::SuffixArray;
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Delimited<T> {
    Separator(usize),
    Value(T),
}

#[derive(Debug)]
pub struct STNode {
    pub parent: usize,
    pub depth: usize,
    pub sa_range: Range<usize>,
    pub child_index: Range<usize>,
    pub head: usize,
    pub pos: usize,
}

pub struct SuffixTree<T>
where
    T: Ord,
{
    text: Vec<T>,
    suffix_array: Vec<usize>,
    lcp_array: Vec<usize>,
    rank: Vec<usize>,
    nodes: Vec<STNode>,
    children: Vec<usize>,
    leafs: Vec<usize>,
    inv: Vec<usize>,
}

struct Link {
    first_child: usize,
    last_child: usize,
    next_sibling: usize,
    prev_sibling: usize,
}

struct SuffixTreeBuilder {
    nodes: Vec<STNode>,
    links: Vec<Link>,
}

impl SuffixTreeBuilder {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            links: Vec::with_capacity(capacity),
        }
    }

    fn push_node(&mut self, depth: usize, start: usize) -> usize {
        let node_id = self.nodes.len();
        self.nodes.push(STNode {
            parent: !0,
            depth,
            sa_range: start..!0,
            child_index: 0..0,
            head: !0,
            pos: !0,
        });
        self.links.push(Link {
            first_child: !0,
            last_child: !0,
            next_sibling: !0,
            prev_sibling: !0,
        });
        node_id
    }

    fn attach_child(&mut self, parent: usize, child: usize) {
        self.nodes[child].parent = parent;
        let tail = self.links[parent].last_child;
        if tail == !0 {
            self.links[parent].first_child = child;
        } else {
            self.links[tail].next_sibling = child;
            self.links[child].prev_sibling = tail;
        }
        self.links[parent].last_child = child;
    }

    fn detach_last_child(&mut self, parent: usize, child: usize) {
        let prev = self.links[child].prev_sibling;
        if prev == !0 {
            self.links[parent].first_child = !0;
        } else {
            self.links[prev].next_sibling = !0;
        }
        self.links[parent].last_child = prev;
        self.links[child].prev_sibling = !0;
        self.links[child].next_sibling = !0;
    }

    fn reindex(&mut self, leafs: &mut [usize]) {
        let mut new_index = vec![!0; self.nodes.len()];
        let mut stack = vec![0];
        let mut index = 0;
        while let Some(node_id) = stack.pop() {
            new_index[node_id] = index;
            index += 1;
            let mut child = self.links[node_id].last_child;
            while child != !0 {
                stack.push(child);
                child = self.links[child].prev_sibling;
            }
        }

        for node in &mut self.nodes {
            if node.parent != !0 {
                node.parent = new_index[node.parent];
            }
        }
        for link in &mut self.links {
            if link.first_child != !0 {
                link.first_child = new_index[link.first_child];
            }
            if link.last_child != !0 {
                link.last_child = new_index[link.last_child];
            }
            if link.next_sibling != !0 {
                link.next_sibling = new_index[link.next_sibling];
            }
            if link.prev_sibling != !0 {
                link.prev_sibling = new_index[link.prev_sibling];
            }
        }
        for leaf in leafs {
            *leaf = new_index[*leaf];
        }

        for i in 0..self.nodes.len() {
            while new_index[i] != i {
                let j = new_index[i];
                self.nodes.swap(i, j);
                self.links.swap(i, j);
                new_index.swap(i, j);
            }
        }
    }

    fn build_aux(&mut self, children: &[usize]) -> Vec<usize> {
        let n = self.nodes.len();
        let mut sub_size = vec![1usize; n];
        let mut heavy = vec![!0usize; n];
        for node in (0..n).rev() {
            let mut max_size = 0usize;
            for &child in &children[self.nodes[node].child_index.clone()] {
                sub_size[node] += sub_size[child];
                if sub_size[child] > max_size {
                    max_size = sub_size[child];
                    heavy[node] = child;
                }
            }
        }

        let mut inv = vec![0usize; n];
        let mut cur = 0usize;
        let mut stack = vec![(0usize, 0usize)];
        while let Some((v, h)) = stack.pop() {
            let mut u = v;
            while u != !0 {
                self.nodes[u].head = h;
                self.nodes[u].pos = cur;
                inv[cur] = u;
                cur += 1;
                for &child in &children[self.nodes[u].child_index.clone()] {
                    if child != heavy[u] {
                        stack.push((child, child));
                    }
                }
                u = heavy[u];
            }
        }
        inv
    }

    fn build_children(&mut self) -> Vec<usize> {
        let mut children = vec![!0; self.nodes.len() - 1];
        let mut index = 0;
        for node_id in 0..self.nodes.len() {
            let mut child = self.links[node_id].first_child;
            self.nodes[node_id].child_index.start = index;
            while child != !0 {
                children[index] = child;
                index += 1;
                child = self.links[child].next_sibling;
            }
            self.nodes[node_id].child_index.end = index;
        }
        children
    }
}

impl<T> SuffixTree<T>
where
    T: Ord,
{
    pub fn new(text: Vec<T>) -> Self {
        let n = text.len();
        let suffix_array = SuffixArray::new(&text);
        let (lcp_array, rank) = suffix_array.lcp_array_with_rank(&text);
        let mut builder = SuffixTreeBuilder::with_capacity(n * 2);
        builder.push_node(0, 0);
        let mut leafs = vec![!0; n + 1];
        leafs[0] = 0;
        let mut stack = Vec::with_capacity(n);
        stack.push(0);
        for i in 1..=n {
            let lcp = lcp_array[i - 1];
            let mut last_popped = !0;
            while builder.nodes[stack.last().cloned().unwrap()].depth > lcp {
                last_popped = stack.pop().unwrap();
                builder.nodes[last_popped].sa_range.end = i;
            }

            // internal node
            if builder.nodes[stack.last().cloned().unwrap()].depth < lcp {
                let parent = stack.last().cloned().unwrap();
                let internal = builder.nodes.len();
                builder.push_node(lcp, builder.nodes[last_popped].sa_range.start);
                builder.detach_last_child(parent, last_popped);
                builder.attach_child(parent, internal);
                builder.attach_child(internal, last_popped);
                stack.push(internal);
            }

            // leaf node
            let parent = stack.last().cloned().unwrap();
            let leaf = builder.nodes.len();
            builder.push_node(n - suffix_array[i], i);
            builder.attach_child(parent, leaf);
            stack.push(leaf);
            leafs[i] = leaf;
        }
        while let Some(node) = stack.pop() {
            builder.nodes[node].sa_range.end = n + 1;
        }

        builder.reindex(&mut leafs[1..]);
        let children = builder.build_children();
        let inv = builder.build_aux(&children);
        let nodes = builder.nodes;
        let suffix_array = suffix_array.into_inner();

        Self {
            text,
            suffix_array,
            lcp_array,
            rank,
            nodes,
            children,
            leafs,
            inv,
        }
    }

    pub fn text(&self) -> &[T] {
        &self.text
    }

    pub fn suffix_array(&self) -> &[usize] {
        &self.suffix_array
    }

    pub fn lcp_array(&self) -> &[usize] {
        &self.lcp_array
    }

    pub fn rank(&self) -> &[usize] {
        &self.rank
    }

    pub fn node_size(&self) -> usize {
        self.nodes.len()
    }

    pub fn node(&self, node_id: usize) -> &STNode {
        &self.nodes[node_id]
    }

    pub fn children(&self, node_id: usize) -> &[usize] {
        &self.children[self.nodes[node_id].child_index.clone()]
    }

    pub fn depth_range(&self, node_id: usize) -> Range<usize> {
        let node = &self.nodes[node_id];
        if node.parent == !0 {
            0..0
        } else {
            self.nodes[node.parent].depth + 1..node.depth + 1
        }
    }

    pub fn kth_substrings(&self) -> KthSubstrings<'_, T> {
        KthSubstrings::new(self)
    }

    fn node_covering_depth(&self, leaf: usize, depth: usize) -> usize {
        debug_assert!(depth > 0);
        debug_assert!(depth <= self.nodes[leaf].depth);
        let mut v = leaf;
        loop {
            let h = self.nodes[v].head;
            let parent = self.nodes[h].parent;
            let parent_depth = if parent == !0 {
                0
            } else {
                self.nodes[parent].depth
            };
            if parent_depth < depth {
                let mut l = self.nodes[h].pos;
                let mut r = self.nodes[v].pos;
                while l < r {
                    let m = (l + r) >> 1;
                    let node_id = self.inv[m];
                    if self.nodes[node_id].depth < depth {
                        l = m + 1;
                    } else {
                        r = m;
                    }
                }
                return self.inv[l];
            }
            v = parent;
        }
    }
}

pub struct KthSubstrings<'a, T>
where
    T: Ord,
{
    tree: &'a SuffixTree<T>,
    prefix_distinct: Vec<u64>,
    prefix_total: Vec<u64>,
}

impl<'a, T> KthSubstrings<'a, T>
where
    T: Ord,
{
    fn new(tree: &'a SuffixTree<T>) -> Self {
        let n = tree.nodes.len();
        let mut prefix_distinct = vec![0u64; n];
        let mut prefix_total = vec![0u64; n];
        for i in 1..n {
            let node = &tree.nodes[i];
            let parent_depth = tree.nodes[node.parent].depth;
            let distinct = (node.depth - parent_depth) as u64;
            let total = distinct * node.sa_range.len() as u64;
            prefix_distinct[i] = prefix_distinct[i - 1] + distinct;
            prefix_total[i] = prefix_total[i - 1] + total;
        }
        Self {
            tree,
            prefix_distinct,
            prefix_total,
        }
    }

    pub fn kth_distinct_substring(&self, k: u64) -> Option<Range<usize>> {
        let idx = self.prefix_distinct.partition_point(|&x| x <= k);
        if idx == self.prefix_distinct.len() {
            return None;
        }
        let node = &self.tree.nodes[idx];
        let offset = k - self.prefix_distinct[idx - 1];
        let len = self.tree.nodes[node.parent].depth + 1 + offset as usize;
        let start = self.tree.suffix_array[node.sa_range.start];
        Some(start..start + len)
    }

    pub fn kth_substring(&self, k: u64) -> Option<Range<usize>> {
        let idx = self.prefix_total.partition_point(|&x| x <= k);
        if idx == self.prefix_total.len() {
            return None;
        }
        let node = &self.tree.nodes[idx];
        let offset = k - self.prefix_total[idx - 1];
        let occ = node.sa_range.len() as u64;
        let len = self.tree.nodes[node.parent].depth + 1 + (offset / occ) as usize;
        let start = self.tree.suffix_array[node.sa_range.start + (offset % occ) as usize];
        Some(start..start + len)
    }

    pub fn index_of_distinct_substring(&self, range: Range<usize>) -> u64 {
        debug_assert!(range.start < range.end && range.end <= self.tree.text.len());
        let m = range.len();
        let leaf = self.tree.leafs[self.tree.rank[range.start]];
        let node = self.tree.node_covering_depth(leaf, m);
        let offset = m - self.tree.nodes[self.tree.nodes[node].parent].depth - 1;
        self.prefix_distinct[node - 1] + offset as u64
    }

    pub fn index_of_substring(&self, range: Range<usize>) -> u64 {
        debug_assert!(range.start < range.end && range.end <= self.tree.text.len());
        let m = range.len();
        let idx = self.tree.rank[range.start];
        let leaf = self.tree.leafs[idx];
        let node = self.tree.node_covering_depth(leaf, m);
        let offset = m - self.tree.nodes[self.tree.nodes[node].parent].depth - 1;
        let occ = self.tree.nodes[node].sa_range.len() as u64;
        let occ_idx = (idx - self.tree.nodes[node].sa_range.start) as u64;
        self.prefix_total[node - 1] + offset as u64 * occ + occ_idx
    }
}

pub struct MultipleSuffixTree<T>
where
    T: Ord + Clone,
{
    texts: Vec<Vec<T>>,
    offsets: Vec<usize>,
    tree: SuffixTree<Delimited<T>>,
    position_map: Vec<(usize, usize)>,
}

impl<T> MultipleSuffixTree<T>
where
    T: Ord + Clone,
{
    pub fn new(texts: Vec<Vec<T>>) -> Self {
        assert!(!texts.is_empty());
        let total_len: usize = texts.iter().map(|text| text.len() + 1).sum();
        let mut concat = Vec::with_capacity(total_len - 1);
        let mut offsets = Vec::with_capacity(texts.len());
        let mut position_map = Vec::with_capacity(total_len);
        for (i, text) in texts.iter().enumerate() {
            offsets.push(concat.len());
            for (pos, value) in text.iter().cloned().enumerate() {
                concat.push(Delimited::Value(value));
                position_map.push((i, pos));
            }
            if i + 1 < texts.len() {
                concat.push(Delimited::Separator(!i));
            }
            position_map.push((i, text.len()));
        }

        let mut tree = SuffixTree::new(concat);
        for node_id in 0..tree.node_size() {
            if tree.children(node_id).is_empty() {
                let node = tree.node(node_id);
                let (text_idx, pos) = position_map[tree.suffix_array()[node.sa_range.start]];
                tree.nodes[node_id].depth = texts[text_idx].len() - pos;
            }
        }

        Self {
            texts,
            offsets,
            tree,
            position_map,
        }
    }

    pub fn texts(&self) -> &[Vec<T>] {
        &self.texts
    }

    pub fn suffix_array(&self) -> &[usize] {
        self.tree.suffix_array()
    }

    pub fn lcp_array(&self) -> &[usize] {
        self.tree.lcp_array()
    }

    pub fn rank(&self) -> &[usize] {
        self.tree.rank()
    }

    pub fn node_size(&self) -> usize {
        self.tree.node_size()
    }

    pub fn node(&self, node_id: usize) -> &STNode {
        self.tree.node(node_id)
    }

    pub fn children(&self, node_id: usize) -> &[usize] {
        self.tree.children(node_id)
    }

    pub fn position_map(&self) -> &[(usize, usize)] {
        &self.position_map
    }

    pub fn depth_range(&self, node_id: usize) -> Range<usize> {
        let node = self.tree.node(node_id);
        if node.parent == !0 {
            return 0..0;
        }
        let parent_depth = self.tree.node(node.parent).depth;
        parent_depth + 1..node.depth + 1
    }

    pub fn kth_substrings(&self) -> MultipleKthSubstrings<'_, T> {
        MultipleKthSubstrings::new(self)
    }

    fn to_global_start(&self, text_idx: usize, pos: usize) -> usize {
        self.offsets[text_idx] + pos
    }
}

pub struct MultipleKthSubstrings<'a, T>
where
    T: Ord + Clone,
{
    tree: &'a MultipleSuffixTree<T>,
    prefix_distinct: Vec<u64>,
    prefix_total: Vec<u64>,
}

impl<'a, T> MultipleKthSubstrings<'a, T>
where
    T: Ord + Clone,
{
    fn new(tree: &'a MultipleSuffixTree<T>) -> Self {
        let n = tree.tree.nodes.len();
        let mut prefix_distinct = vec![0u64; n];
        let mut prefix_total = vec![0u64; n];
        for i in 1..n {
            let node = &tree.tree.nodes[i];
            let parent_depth = tree.tree.nodes[node.parent].depth;
            let distinct = (node.depth - parent_depth) as u64;
            let total = distinct * node.sa_range.len() as u64;
            prefix_distinct[i] = prefix_distinct[i - 1] + distinct;
            prefix_total[i] = prefix_total[i - 1] + total;
        }
        Self {
            tree,
            prefix_distinct,
            prefix_total,
        }
    }

    pub fn kth_distinct_substring(&self, k: u64) -> Option<(usize, Range<usize>)> {
        let idx = self.prefix_distinct.partition_point(|&x| x <= k);
        if idx == self.prefix_distinct.len() {
            return None;
        }
        let node = &self.tree.tree.nodes[idx];
        let parent_depth = self.tree.tree.nodes[node.parent].depth;
        let offset = k - self.prefix_distinct[idx - 1];
        let len = parent_depth + 1 + offset as usize;
        let start = self.tree.tree.suffix_array[node.sa_range.start];
        let (text_idx, pos) = self.tree.position_map[start];
        Some((text_idx, pos..pos + len))
    }

    pub fn kth_substring(&self, k: u64) -> Option<(usize, Range<usize>)> {
        let idx = self.prefix_total.partition_point(|&x| x <= k);
        if idx == self.prefix_total.len() {
            return None;
        }
        let node = &self.tree.tree.nodes[idx];
        let parent_depth = self.tree.tree.nodes[node.parent].depth;
        let offset = k - self.prefix_total[idx - 1];
        let occ = node.sa_range.len() as u64;
        let len_offset = (offset / occ) as usize;
        let occ_idx = (offset % occ) as usize;
        let len = parent_depth + 1 + len_offset;
        let start = self.tree.tree.suffix_array[node.sa_range.start + occ_idx];
        let (text_idx, pos) = self.tree.position_map[start];
        Some((text_idx, pos..pos + len))
    }

    pub fn index_of_distinct_substring(&self, (text_idx, range): (usize, Range<usize>)) -> u64 {
        debug_assert!(text_idx < self.tree.texts.len());
        debug_assert!(range.start < range.end && range.end <= self.tree.texts[text_idx].len());
        let m = range.len();
        let start = self.tree.to_global_start(text_idx, range.start);
        let sa_idx = self.tree.tree.rank[start];
        let leaf = self.tree.tree.leafs[sa_idx];
        let node = self.tree.tree.node_covering_depth(leaf, m);
        let parent_depth = self.tree.tree.nodes[self.tree.tree.nodes[node].parent].depth;
        let offset = m - (parent_depth + 1);
        self.prefix_distinct[node - 1] + offset as u64
    }

    pub fn index_of_substring(&self, (text_idx, range): (usize, Range<usize>)) -> u64 {
        debug_assert!(text_idx < self.tree.texts.len());
        debug_assert!(range.start < range.end && range.end <= self.tree.texts[text_idx].len());
        let m = range.len();
        let start = self.tree.to_global_start(text_idx, range.start);
        let sa_idx = self.tree.tree.rank[start];
        let leaf = self.tree.tree.leafs[sa_idx];
        let node = self.tree.tree.node_covering_depth(leaf, m);
        let parent_depth = self.tree.tree.nodes[self.tree.tree.nodes[node].parent].depth;
        let offset = m - (parent_depth + 1);
        let occ = self.tree.tree.nodes[node].sa_range.len() as u64;
        let occ_idx = (sa_idx - self.tree.tree.nodes[node].sa_range.start) as u64;
        self.prefix_total[node - 1] + offset as u64 * occ + occ_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn test_suffix_tree_substrings() {
        let mut rng = Xorshift::default();
        for _ in 0..500 {
            let n = rng.random(1usize..=100);
            let csize = rng.random(1usize..=10);
            let s: Vec<_> = rng.random_iter(0usize..csize).take(n).collect();
            let st = SuffixTree::new(s.clone());
            let mut substrings = vec![];
            assert_eq!(st.node(0).parent, !0);
            for node_id in 0..st.node_size() {
                let node = st.node(node_id);
                if node.parent != !0 {
                    let parent_depth = st.node(node.parent).depth;
                    assert!(parent_depth < node.depth);
                }
                for depth in st.depth_range(node_id) {
                    for sa_idx in node.sa_range.clone() {
                        let start = st.suffix_array[sa_idx];
                        substrings.push(start..start + depth);
                    }
                }
                for &child_id in st.children(node_id) {
                    assert_eq!(st.node(child_id).parent, node_id);
                }
            }
            assert!(substrings.iter().map(|r| &s[r.clone()]).is_sorted());
            let mut expected = vec![];
            for i in 0..n {
                for j in i + 1..=n {
                    expected.push(i..j);
                }
            }
            expected.sort_by_key(|r| (&s[r.clone()], r.start, r.end));
            substrings.sort_by_key(|r| (&s[r.clone()], r.start, r.end));
            assert_eq!(substrings, expected);
        }
    }

    #[test]
    fn test_suffix_tree_distinct_substrings() {
        let mut rng = Xorshift::default();
        for _ in 0..500 {
            let n = rng.random(1usize..=100);
            let csize = rng.random(1usize..=10);
            let s: Vec<_> = rng.random_iter(0usize..csize).take(n).collect();
            let st = SuffixTree::new(s.clone());
            let mut substrings = vec![];
            assert_eq!(st.node(0).parent, !0);
            for node_id in 0..st.node_size() {
                let node = st.node(node_id);
                if node.parent != !0 {
                    let parent_depth = st.node(node.parent).depth;
                    assert!(parent_depth < node.depth);
                }
                for depth in st.depth_range(node_id) {
                    let sa_idx = node.sa_range.start;
                    let start = st.suffix_array[sa_idx];
                    substrings.push(start..start + depth);
                }
                for &child_id in st.children(node_id) {
                    assert_eq!(st.node(child_id).parent, node_id);
                }
            }
            assert!(substrings.iter().map(|r| &s[r.clone()]).is_sorted());
            let mut expected = BTreeSet::new();
            for i in 0..n {
                for j in i + 1..=n {
                    expected.insert(&s[i..j]);
                }
            }
            assert_eq!(substrings.len(), expected.len());
            for (a, &b) in substrings.iter().zip(expected.iter()) {
                assert_eq!(&s[a.clone()], b);
            }
        }
    }

    #[test]
    fn test_suffix_tree_kth_distinct_substring() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(0usize..=60);
            let csize = rng.random(1usize..=10);
            let s: Vec<_> = rng.random_iter(0usize..csize).take(n).collect();
            let st = SuffixTree::new(s.clone());
            let kth = st.kth_substrings();

            let mut set = BTreeSet::new();
            for i in 0..n {
                for j in i + 1..=n {
                    set.insert(s[i..j].to_vec());
                }
            }
            let substrings: Vec<_> = set.into_iter().collect();
            for (k, expected) in substrings.iter().enumerate() {
                let range = kth.kth_distinct_substring(k as u64).unwrap();
                assert_eq!(&s[range.clone()], expected.as_slice());
            }
            assert_eq!(kth.kth_distinct_substring(substrings.len() as u64), None);
            let mut index_map = BTreeMap::new();
            for (k, expected) in substrings.iter().enumerate() {
                index_map.insert(expected.clone(), k as _);
            }
            for i in 0..n {
                for j in i + 1..=n {
                    let key = s[i..j].to_vec();
                    let expected = *index_map.get(&key).unwrap();
                    assert_eq!(kth.index_of_distinct_substring(i..j), expected);
                }
            }
        }
    }

    #[test]
    fn test_suffix_tree_kth_substring() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(0usize..=60);
            let csize = rng.random(1usize..=10);
            let s: Vec<_> = rng.random_iter(0usize..csize).take(n).collect();
            let st = SuffixTree::new(s.clone());
            let kth = st.kth_substrings();

            let mut substrings = Vec::new();
            for node_id in 1..st.node_size() {
                let node = st.node(node_id);
                for depth in st.depth_range(node_id) {
                    for sa_idx in node.sa_range.clone() {
                        let start = st.suffix_array[sa_idx];
                        substrings.push(start..start + depth);
                    }
                }
            }

            for (k, range) in substrings.iter().enumerate() {
                let got = kth.kth_substring(k as u64).unwrap();
                assert_eq!(got, range.clone());
                assert_eq!(kth.index_of_substring(range.clone()), k as _);
            }
            assert_eq!(kth.kth_substring(substrings.len() as u64), None);
        }
    }

    #[test]
    fn test_multiple_suffix_tree_substrings() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let k = rng.random(1usize..=6);
            let csize = rng.random(1usize..=10);
            let mut texts = Vec::with_capacity(k);
            for _ in 0..k {
                let n = rng.random(0usize..=40);
                let s: Vec<_> = rng.random_iter(0usize..csize).take(n).collect();
                texts.push(s);
            }
            let st = MultipleSuffixTree::new(texts.clone());

            let mut substrings = vec![];
            assert_eq!(st.node(0).parent, !0);
            for node_id in 0..st.node_size() {
                let node = st.node(node_id);
                if node.parent != !0 {
                    let parent_depth = st.node(node.parent).depth;
                    assert!(parent_depth <= node.depth);
                }
                for depth in st.depth_range(node_id) {
                    for sa_idx in node.sa_range.clone() {
                        let start = st.suffix_array()[sa_idx];
                        let (text_idx, pos) = st.position_map()[start];
                        assert!(pos + depth <= texts[text_idx].len());
                        substrings.push((text_idx, pos..pos + depth));
                    }
                }
                for &child_id in st.children(node_id) {
                    assert_eq!(st.node(child_id).parent, node_id);
                }
            }
            assert!(
                substrings
                    .iter()
                    .map(|(i, r)| &texts[*i][r.clone()])
                    .is_sorted()
            );
            let mut expected = vec![];
            for (i, text) in texts.iter().enumerate() {
                for l in 0..text.len() {
                    for r in l + 1..=text.len() {
                        expected.push((i, l..r));
                    }
                }
            }
            expected.sort_unstable_by_key(|(i, r)| (&texts[*i][r.clone()], *i, r.start, r.end));
            substrings.sort_unstable_by_key(|(i, r)| (&texts[*i][r.clone()], *i, r.start, r.end));
            assert_eq!(substrings, expected);
        }
    }

    #[test]
    fn test_multiple_suffix_tree_distinct_substrings() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let k = rng.random(1usize..=6);
            let csize = rng.random(1usize..=10);
            let mut texts = Vec::with_capacity(k);
            for _ in 0..k {
                let n = rng.random(0usize..=40);
                let s: Vec<_> = rng.random_iter(0usize..csize).take(n).collect();
                texts.push(s);
            }
            let st = MultipleSuffixTree::new(texts.clone());

            let mut substrings = vec![];
            assert_eq!(st.node(0).parent, !0);
            for node_id in 0..st.node_size() {
                let node = st.node(node_id);
                if node.parent != !0 {
                    let parent_depth = st.node(node.parent).depth;
                    assert!(parent_depth <= node.depth);
                }
                for depth in st.depth_range(node_id) {
                    let sa_idx = node.sa_range.start;
                    let start = st.suffix_array()[sa_idx];
                    let (text_idx, pos) = st.position_map()[start];
                    assert!(pos + depth <= texts[text_idx].len());
                    substrings.push((text_idx, pos..pos + depth));
                }
                for &child_id in st.children(node_id) {
                    assert_eq!(st.node(child_id).parent, node_id);
                }
            }
            assert!(
                substrings
                    .iter()
                    .map(|(i, r)| &texts[*i][r.clone()])
                    .is_sorted()
            );
            let mut expected = BTreeSet::new();
            for text in &texts {
                for i in 0..text.len() {
                    for j in i + 1..=text.len() {
                        expected.insert(text[i..j].to_vec());
                    }
                }
            }
            assert_eq!(substrings.len(), expected.len());
            for (a, b) in substrings.iter().zip(expected.iter()) {
                assert_eq!(&texts[a.0][a.1.clone()], b.as_slice());
            }
        }
    }

    #[test]
    fn test_multiple_suffix_tree_kth_distinct_substring() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let k = rng.random(1usize..=5);
            let csize = rng.random(1usize..=10);
            let mut texts = Vec::with_capacity(k);
            for _ in 0..k {
                let n = rng.random(0usize..=30);
                let s: Vec<_> = rng.random_iter(0usize..csize).take(n).collect();
                texts.push(s);
            }
            let st = MultipleSuffixTree::new(texts.clone());
            let kth = st.kth_substrings();

            let mut set = BTreeSet::new();
            for text in &texts {
                for i in 0..text.len() {
                    for j in i + 1..=text.len() {
                        set.insert(text[i..j].to_vec());
                    }
                }
            }
            let substrings: Vec<_> = set.into_iter().collect();
            for (idx, expected) in substrings.iter().enumerate() {
                let (text_idx, range) = kth.kth_distinct_substring(idx as u64).unwrap();
                assert_eq!(&texts[text_idx][range.clone()], expected.as_slice());
            }
            assert_eq!(kth.kth_distinct_substring(substrings.len() as u64), None);
            let mut index_map = BTreeMap::new();
            for (idx, expected) in substrings.iter().enumerate() {
                index_map.insert(expected.clone(), idx as _);
            }
            for (text_idx, text) in texts.iter().enumerate() {
                for i in 0..text.len() {
                    for j in i + 1..=text.len() {
                        let key = text[i..j].to_vec();
                        let expected = *index_map.get(&key).unwrap();
                        assert_eq!(kth.index_of_distinct_substring((text_idx, i..j)), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn test_multiple_suffix_tree_kth_substring() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let k = rng.random(1usize..=5);
            let csize = rng.random(1usize..=10);
            let mut texts = Vec::with_capacity(k);
            for _ in 0..k {
                let n = rng.random(0usize..=30);
                let s: Vec<_> = rng.random_iter(0usize..csize).take(n).collect();
                texts.push(s);
            }
            let st = MultipleSuffixTree::new(texts.clone());
            let kth = st.kth_substrings();

            let mut substrings = Vec::new();
            for node_id in 1..st.node_size() {
                let node = st.node(node_id);
                for depth in st.depth_range(node_id) {
                    for sa_idx in node.sa_range.clone() {
                        let start = st.suffix_array()[sa_idx];
                        let (text_idx, pos) = st.position_map()[start];
                        substrings.push((text_idx, pos..pos + depth));
                    }
                }
            }

            for (idx, (text_idx, range)) in substrings.iter().enumerate() {
                let got = kth.kth_substring(idx as u64).unwrap();
                assert_eq!(got, (*text_idx, range.clone()));
                assert_eq!(kth.index_of_substring((*text_idx, range.clone())), idx as _);
            }
            assert_eq!(kth.kth_substring(substrings.len() as u64), None);
        }
    }
}
