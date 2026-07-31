use super::{Allocator, MemoryPool, Monoid, RangeBoundsExt};
use std::{
    fmt::{self, Debug, Formatter},
    ops::{Range, RangeBounds},
    ptr::NonNull,
};

type NodePtr<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    children: [NodePtr<T>; 2],
    value: T,
}

impl<T> Node<T> {
    fn new(children: [NodePtr<T>; 2], value: T) -> Self {
        Self { children, value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct PersistentSegmentTreeVersion(usize);

impl PersistentSegmentTreeVersion {
    fn base() -> Self {
        Self(0)
    }

    fn new(version_id: usize) -> Self {
        Self(version_id)
    }

    fn index(self) -> usize {
        self.0
    }
}

pub struct PersistentSegmentTree<M>
where
    M: Monoid,
{
    len: usize,
    version_roots: Vec<NodePtr<M::T>>,
    allocator: MemoryPool<Node<M::T>>,
}

impl<M> Debug for PersistentSegmentTree<M>
where
    M: Monoid,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PersistentSegmentTree")
            .field("len", &self.len)
            .field("versions", &self.version_roots.len())
            .finish()
    }
}

impl<M> PersistentSegmentTree<M>
where
    M: Monoid,
{
    #[must_use]
    pub fn new(len: usize) -> Self {
        Self {
            len,
            version_roots: vec![None],
            allocator: MemoryPool::new(),
        }
    }

    pub fn base(&self) -> PersistentSegmentTreeVersion {
        PersistentSegmentTreeVersion::base()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn version_root(&self, version: PersistentSegmentTreeVersion) -> NodePtr<M::T> {
        *self
            .version_roots
            .get(version.index())
            .expect("invalid version")
    }

    fn push_version_root(&mut self, root: NodePtr<M::T>) -> PersistentSegmentTreeVersion {
        let version_id = self.version_roots.len();
        self.version_roots.push(root);
        PersistentSegmentTreeVersion::new(version_id)
    }

    fn allocate_node(&mut self, children: [NodePtr<M::T>; 2], value: M::T) -> NonNull<Node<M::T>> {
        self.allocator.allocate(Node::new(children, value))
    }

    fn build_dfs(&mut self, start: usize, end: usize, values: &[M::T]) -> NodePtr<M::T> {
        if end - start == 1 {
            return self.leaf_node(values[start].clone());
        }
        let mid = (start + end) / 2;
        let left = self.build_dfs(start, mid, values);
        let right = self.build_dfs(mid, end, values);
        self.merge_nodes(left, right)
    }

    fn leaf_node(&mut self, value: M::T) -> NodePtr<M::T> {
        Some(self.allocate_node([None, None], value))
    }

    fn merge_nodes(&mut self, left: NodePtr<M::T>, right: NodePtr<M::T>) -> NodePtr<M::T> {
        if left.is_none() && right.is_none() {
            None
        } else {
            let value = M::operate(&Self::subtree_value(left), &Self::subtree_value(right));
            Some(self.allocate_node([left, right], value))
        }
    }

    fn subtree_value(node: NodePtr<M::T>) -> M::T {
        node.map(|node| unsafe { node.as_ref().value.clone() })
            .unwrap_or_else(M::unit)
    }

    fn children(node: NodePtr<M::T>) -> [NodePtr<M::T>; 2] {
        node.map(|node| unsafe { node.as_ref().children })
            .unwrap_or([None, None])
    }

    fn point_get_dfs(node: NodePtr<M::T>, start: usize, end: usize, index: usize) -> M::T {
        let Some(node) = node else {
            return M::unit();
        };
        let node = unsafe { node.as_ref() };
        if end - start == 1 {
            node.value.clone()
        } else {
            let mid = (start + end) / 2;
            if index < mid {
                Self::point_get_dfs(node.children[0], start, mid, index)
            } else {
                Self::point_get_dfs(node.children[1], mid, end, index)
            }
        }
    }

    fn fold_dfs(node: NodePtr<M::T>, start: usize, end: usize, range: &Range<usize>) -> M::T {
        if range.end <= start || end <= range.start {
            return M::unit();
        }
        let Some(node) = node else {
            return M::unit();
        };
        let node = unsafe { node.as_ref() };
        if range.start <= start && end <= range.end {
            node.value.clone()
        } else {
            let mid = (start + end) / 2;
            if range.end <= mid {
                return Self::fold_dfs(node.children[0], start, mid, range);
            }
            if mid <= range.start {
                return Self::fold_dfs(node.children[1], mid, end, range);
            }
            let left = Self::fold_dfs(node.children[0], start, mid, range);
            let right = Self::fold_dfs(node.children[1], mid, end, range);
            M::operate(&left, &right)
        }
    }

    fn partition_point_dfs<P>(
        node: NodePtr<M::T>,
        start: usize,
        end: usize,
        left: usize,
        acc: &mut M::T,
        pred: &mut P,
    ) -> Option<usize>
    where
        P: FnMut(&M::T) -> bool,
    {
        if end <= left {
            return None;
        }
        if left <= start {
            let nacc = M::operate(acc, &Self::subtree_value(node));
            if pred(&nacc) {
                *acc = nacc;
                return None;
            }
            if end - start == 1 {
                return Some(start);
            }
        }
        let mid = (start + end) / 2;
        let [l, r] = Self::children(node);
        if let Some(pos) = Self::partition_point_dfs(l, start, mid, left, acc, pred) {
            Some(pos)
        } else {
            Self::partition_point_dfs(r, mid, end, left, acc, pred)
        }
    }

    fn rpartition_point_dfs<P>(
        node: NodePtr<M::T>,
        start: usize,
        end: usize,
        right: usize,
        acc: &mut M::T,
        pred: &mut P,
    ) -> Option<usize>
    where
        P: FnMut(&M::T) -> bool,
    {
        if right <= start {
            return None;
        }
        if end <= right {
            let nacc = M::operate(&Self::subtree_value(node), acc);
            if pred(&nacc) {
                *acc = nacc;
                return None;
            }
            if end - start == 1 {
                return Some(end);
            }
        }
        let mid = (start + end) / 2;
        let [l, r] = Self::children(node);
        if let Some(pos) = Self::rpartition_point_dfs(r, mid, end, right, acc, pred) {
            Some(pos)
        } else {
            Self::rpartition_point_dfs(l, start, mid, right, acc, pred)
        }
    }

    fn set_dfs(
        &mut self,
        node: NodePtr<M::T>,
        start: usize,
        end: usize,
        index: usize,
        value: &M::T,
    ) -> NodePtr<M::T> {
        if end - start == 1 {
            return self.leaf_node(value.clone());
        }
        let mid = (start + end) / 2;
        let mut children = Self::children(node);
        if index < mid {
            children[0] = self.set_dfs(children[0], start, mid, index, value);
        } else {
            children[1] = self.set_dfs(children[1], mid, end, index, value);
        }
        self.merge_nodes(children[0], children[1])
    }

    fn update_dfs(
        &mut self,
        node: NodePtr<M::T>,
        start: usize,
        end: usize,
        index: usize,
        value: &M::T,
    ) -> NodePtr<M::T> {
        if end - start == 1 {
            return self.leaf_node(M::operate(&Self::subtree_value(node), value));
        }
        let mid = (start + end) / 2;
        let mut children = Self::children(node);
        if index < mid {
            children[0] = self.update_dfs(children[0], start, mid, index, value);
        } else {
            children[1] = self.update_dfs(children[1], mid, end, index, value);
        }
        self.merge_nodes(children[0], children[1])
    }

    pub fn from_vec(&mut self, v: Vec<M::T>) -> PersistentSegmentTreeVersion {
        assert_eq!(v.len(), self.len);
        let root = if self.len == 0 {
            None
        } else {
            self.build_dfs(0, self.len, &v)
        };
        self.push_version_root(root)
    }

    pub fn set(
        &mut self,
        version: PersistentSegmentTreeVersion,
        index: usize,
        value: M::T,
    ) -> PersistentSegmentTreeVersion {
        assert!(index < self.len);
        let root = self.set_dfs(self.version_root(version), 0, self.len, index, &value);
        self.push_version_root(root)
    }

    pub fn update(
        &mut self,
        version: PersistentSegmentTreeVersion,
        index: usize,
        value: M::T,
    ) -> PersistentSegmentTreeVersion {
        assert!(index < self.len);
        let root = self.update_dfs(self.version_root(version), 0, self.len, index, &value);
        self.push_version_root(root)
    }

    #[must_use]
    pub fn get(&self, version: PersistentSegmentTreeVersion, index: usize) -> M::T {
        assert!(index < self.len);
        Self::point_get_dfs(self.version_root(version), 0, self.len, index)
    }

    #[must_use]
    pub fn fold<R>(&self, version: PersistentSegmentTreeVersion, range: R) -> M::T
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.len).expect("invalid range");
        if range.is_empty() {
            M::unit()
        } else {
            Self::fold_dfs(self.version_root(version), 0, self.len, &range)
        }
    }

    pub fn partition_point_acc<P>(
        &self,
        version: PersistentSegmentTreeVersion,
        left: usize,
        mut pred: P,
    ) -> (usize, M::T)
    where
        P: FnMut(&M::T) -> bool,
    {
        let root = self.version_root(version);
        let mut acc = M::unit();
        let pos = if self.len == 0 {
            None
        } else {
            Self::partition_point_dfs(root, 0, self.len, left, &mut acc, &mut pred)
        };
        (pos.unwrap_or(self.len), acc)
    }

    pub fn rpartition_point_acc<P>(
        &self,
        version: PersistentSegmentTreeVersion,
        right: usize,
        mut pred: P,
    ) -> (usize, M::T)
    where
        P: FnMut(&M::T) -> bool,
    {
        let root = self.version_root(version);
        let mut acc = M::unit();
        let pos = if self.len == 0 {
            None
        } else {
            Self::rpartition_point_dfs(root, 0, self.len, right, &mut acc, &mut pred)
        };
        (pos.unwrap_or(0), acc)
    }

    #[must_use]
    pub fn fold_all(&self, version: PersistentSegmentTreeVersion) -> M::T {
        Self::subtree_value(self.version_root(version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::ConcatenateOperation,
        tools::{WithEmptySegment as Wes, Xorshift},
    };

    const N: usize = 12;
    const Q: usize = 2_000;
    const SIGMA: u8 = 6;

    fn rand_word(rng: &mut Xorshift) -> Vec<u8> {
        let len = rng.random(0..4usize);
        (0..len).map(|_| rng.random(0..SIGMA)).collect()
    }

    #[test]
    fn test_persistent_segment_tree_random_non_commutative() {
        let mut rng = Xorshift::default();
        let mut segtree: PersistentSegmentTree<ConcatenateOperation<u8>> =
            PersistentSegmentTree::new(N);
        let initial: Vec<_> = (0..N).map(|_| rand_word(&mut rng)).collect();
        let mut versions = vec![segtree.base(), segtree.from_vec(initial.clone())];
        let mut states = vec![vec![Vec::new(); N], initial];

        for _ in 0..Q {
            let base_version = rng.random(0..versions.len());
            let index = rng.random(0..N);
            let mut state = states[base_version].clone();

            if rng.gen_bool(0.5) {
                let value = rand_word(&mut rng);
                state[index] = value.clone();
                versions.push(segtree.set(versions[base_version], index, value));
            } else {
                let value = rand_word(&mut rng);
                state[index].extend_from_slice(&value);
                versions.push(segtree.update(versions[base_version], index, value));
            }
            states.push(state);

            let version = rng.random(0..versions.len());
            let index = rng.random(0..N);
            let (start, end) = rng.random(Wes(N));
            let expected: Vec<_> = states[version][start..end]
                .iter()
                .flat_map(|word| word.iter().copied())
                .collect();
            let expected_all: Vec<_> = states[version]
                .iter()
                .flat_map(|word| word.iter().copied())
                .collect();

            assert_eq!(
                segtree.get(versions[version], index),
                states[version][index]
            );
            assert_eq!(segtree.fold(versions[version], start..end), expected);
            assert_eq!(segtree.fold_all(versions[version]), expected_all);

            let left = rng.random(0..=N);
            let limit = rng.random(1..=N * 4);
            let mut expected_acc = Vec::new();
            let mut expected_pos = left;
            while expected_pos < N {
                let mut nacc = expected_acc.clone();
                nacc.extend_from_slice(&states[version][expected_pos]);
                if nacc.len() < limit {
                    expected_acc = nacc;
                    expected_pos += 1;
                } else {
                    break;
                }
            }
            assert_eq!(
                segtree.partition_point_acc(versions[version], left, |acc| acc.len() < limit),
                (expected_pos, expected_acc)
            );

            let right = rng.random(0..=N);
            let limit = rng.random(1..=N * 4);
            let mut expected_acc = Vec::new();
            let mut expected_pos = right;
            while expected_pos > 0 {
                let mut nacc = states[version][expected_pos - 1].clone();
                nacc.extend_from_slice(&expected_acc);
                if nacc.len() < limit {
                    expected_acc = nacc;
                    expected_pos -= 1;
                } else {
                    break;
                }
            }
            assert_eq!(
                segtree.rpartition_point_acc(versions[version], right, |acc| acc.len() < limit),
                (expected_pos, expected_acc)
            );
        }
    }
}
