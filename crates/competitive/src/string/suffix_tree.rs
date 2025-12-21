use super::SuffixArray;
use std::ops::Range;

#[derive(Debug)]
pub struct STNode {
    pub parent: usize,
    pub depth: usize,
    pub sa_range: Range<usize>,
    pub child_index: Range<usize>,
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

    fn reindex(&mut self) {
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

        for i in 0..self.nodes.len() {
            while new_index[i] != i {
                let j = new_index[i];
                self.nodes.swap(i, j);
                self.links.swap(i, j);
                new_index.swap(i, j);
            }
        }
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
        }
        while let Some(node) = stack.pop() {
            builder.nodes[node].sa_range.end = n + 1;
        }

        builder.reindex();
        let children = builder.build_children();
        let nodes = builder.nodes;

        Self {
            text,
            suffix_array: suffix_array.into_inner(),
            lcp_array,
            rank,
            nodes,
            children,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::BTreeSet;

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
}
