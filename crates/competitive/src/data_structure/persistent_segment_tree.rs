use super::{Allocator, MemoryPool, Monoid, RangeBoundsExt};
use std::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ops::{Range, RangeBounds},
    ptr::NonNull,
    rc::Rc,
};

type NodePtr<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    child: [NodePtr<T>; 2],
    value: T,
}

impl<T> Node<T> {
    fn new(child: [NodePtr<T>; 2], value: T) -> Self {
        Self { child, value }
    }
}

struct PersistentSegmentTreeAllocator<T, A = MemoryPool<Node<T>>>
where
    A: Allocator<Node<T>>,
{
    // The allocator is append-only, and allocated nodes stay immutable afterwards.
    alloc: UnsafeCell<A>,
    _marker: PhantomData<fn() -> T>,
}

impl<T> PersistentSegmentTreeAllocator<T> {
    fn new() -> Self {
        Self {
            alloc: UnsafeCell::new(MemoryPool::new()),
            _marker: PhantomData,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            alloc: UnsafeCell::new(MemoryPool::with_capacity(capacity.max(1))),
            _marker: PhantomData,
        }
    }
}

impl<T, A> PersistentSegmentTreeAllocator<T, A>
where
    A: Allocator<Node<T>>,
{
    fn allocate(&self, node: Node<T>) -> NonNull<Node<T>> {
        unsafe { (&mut *self.alloc.get()).allocate(node) }
    }
}

pub struct PersistentSegmentTree<M>
where
    M: Monoid<T: PartialEq>,
{
    n: usize,
    root: NodePtr<M::T>,
    allocator: Rc<PersistentSegmentTreeAllocator<M::T>>,
}

impl<M> Clone for PersistentSegmentTree<M>
where
    M: Monoid<T: PartialEq>,
{
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            root: self.root,
            allocator: Rc::clone(&self.allocator),
        }
    }
}

impl<M> Debug for PersistentSegmentTree<M>
where
    M: Monoid<T: PartialEq + Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PersistentSegmentTree")
            .field("n", &self.n)
            .field("fold_all", &self.fold_all())
            .finish()
    }
}

impl<M> PersistentSegmentTree<M>
where
    M: Monoid<T: PartialEq>,
{
    pub fn new(n: usize) -> Self {
        Self {
            n,
            root: None,
            allocator: Rc::new(PersistentSegmentTreeAllocator::new()),
        }
    }

    pub fn from_vec(v: Vec<M::T>) -> Self {
        let n = v.len();
        let allocator = Rc::new(PersistentSegmentTreeAllocator::with_capacity(
            n.saturating_mul(2).max(1),
        ));
        let root = if n == 0 {
            None
        } else {
            Self::build_dfs(&allocator, 0, n, &v)
        };
        Self { n, root, allocator }
    }

    fn with_root(
        n: usize,
        root: NodePtr<M::T>,
        allocator: Rc<PersistentSegmentTreeAllocator<M::T>>,
    ) -> Self {
        Self { n, root, allocator }
    }

    fn build_dfs(
        allocator: &PersistentSegmentTreeAllocator<M::T>,
        l: usize,
        r: usize,
        values: &[M::T],
    ) -> NodePtr<M::T> {
        if r - l == 1 {
            return Self::leaf_node(allocator, values[l].clone());
        }
        let m = (l + r) / 2;
        let left = Self::build_dfs(allocator, l, m, values);
        let right = Self::build_dfs(allocator, m, r, values);
        Self::merge_nodes(allocator, left, right)
    }

    fn leaf_node(allocator: &PersistentSegmentTreeAllocator<M::T>, value: M::T) -> NodePtr<M::T> {
        if M::is_unit(&value) {
            None
        } else {
            Some(allocator.allocate(Node::new([None, None], value)))
        }
    }

    fn merge_nodes(
        allocator: &PersistentSegmentTreeAllocator<M::T>,
        left: NodePtr<M::T>,
        right: NodePtr<M::T>,
    ) -> NodePtr<M::T> {
        if left.is_none() && right.is_none() {
            None
        } else {
            let value = M::operate(&Self::node_value(left), &Self::node_value(right));
            Some(allocator.allocate(Node::new([left, right], value)))
        }
    }

    fn node_value(node: NodePtr<M::T>) -> M::T {
        node.map(|node| unsafe { node.as_ref().value.clone() })
            .unwrap_or_else(M::unit)
    }

    fn node_children(node: NodePtr<M::T>) -> [NodePtr<M::T>; 2] {
        node.map(|node| unsafe { node.as_ref().child })
            .unwrap_or([None, None])
    }

    fn point_get_dfs(node: NodePtr<M::T>, l: usize, r: usize, k: usize) -> M::T {
        let Some(node) = node else {
            return M::unit();
        };
        let node = unsafe { node.as_ref() };
        if r - l == 1 {
            node.value.clone()
        } else {
            let m = (l + r) / 2;
            if k < m {
                Self::point_get_dfs(node.child[0], l, m, k)
            } else {
                Self::point_get_dfs(node.child[1], m, r, k)
            }
        }
    }

    fn fold_dfs(node: NodePtr<M::T>, l: usize, r: usize, range: &Range<usize>) -> M::T {
        if range.end <= l || r <= range.start {
            return M::unit();
        }
        let Some(node) = node else {
            return M::unit();
        };
        let node = unsafe { node.as_ref() };
        if range.start <= l && r <= range.end {
            node.value.clone()
        } else {
            let m = (l + r) / 2;
            let left = Self::fold_dfs(node.child[0], l, m, range);
            let right = Self::fold_dfs(node.child[1], m, r, range);
            M::operate(&left, &right)
        }
    }

    fn set_dfs(
        allocator: &PersistentSegmentTreeAllocator<M::T>,
        node: NodePtr<M::T>,
        l: usize,
        r: usize,
        k: usize,
        value: &M::T,
    ) -> NodePtr<M::T> {
        if r - l == 1 {
            return Self::leaf_node(allocator, value.clone());
        }
        let m = (l + r) / 2;
        let mut child = Self::node_children(node);
        if k < m {
            child[0] = Self::set_dfs(allocator, child[0], l, m, k, value);
        } else {
            child[1] = Self::set_dfs(allocator, child[1], m, r, k, value);
        }
        Self::merge_nodes(allocator, child[0], child[1])
    }

    fn update_dfs(
        allocator: &PersistentSegmentTreeAllocator<M::T>,
        node: NodePtr<M::T>,
        l: usize,
        r: usize,
        k: usize,
        value: &M::T,
    ) -> NodePtr<M::T> {
        if r - l == 1 {
            return Self::leaf_node(allocator, M::operate(&Self::node_value(node), value));
        }
        let m = (l + r) / 2;
        let mut child = Self::node_children(node);
        if k < m {
            child[0] = Self::update_dfs(allocator, child[0], l, m, k, value);
        } else {
            child[1] = Self::update_dfs(allocator, child[1], m, r, k, value);
        }
        Self::merge_nodes(allocator, child[0], child[1])
    }

    pub fn set(&self, k: usize, value: M::T) -> Self {
        assert!(k < self.n);
        let root = Self::set_dfs(&self.allocator, self.root, 0, self.n, k, &value);
        Self::with_root(self.n, root, Rc::clone(&self.allocator))
    }

    pub fn update(&self, k: usize, value: M::T) -> Self {
        assert!(k < self.n);
        if M::is_unit(&value) {
            return self.clone();
        }
        let root = Self::update_dfs(&self.allocator, self.root, 0, self.n, k, &value);
        Self::with_root(self.n, root, Rc::clone(&self.allocator))
    }

    pub fn get(&self, k: usize) -> M::T {
        assert!(k < self.n);
        Self::point_get_dfs(self.root, 0, self.n, k)
    }

    pub fn fold<R>(&self, range: R) -> M::T
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        if range.is_empty() {
            M::unit()
        } else {
            Self::fold_dfs(self.root, 0, self.n, &range)
        }
    }

    pub fn fold_all(&self) -> M::T {
        Self::node_value(self.root)
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
        let len = rng.rand(4) as usize;
        (0..len).map(|_| rng.rand(SIGMA as u64) as u8).collect()
    }

    #[test]
    fn test_persistent_segment_tree_random_non_commutative() {
        let mut rng = Xorshift::default();
        let initial = (0..N).map(|_| rand_word(&mut rng)).collect::<Vec<_>>();
        let mut segs = vec![
            PersistentSegmentTree::<ConcatenateOperation<u8>>::new(N),
            PersistentSegmentTree::<ConcatenateOperation<u8>>::from_vec(initial.clone()),
        ];
        let mut arrs = vec![vec![Vec::new(); N], initial];

        for _ in 0..Q {
            let base = rng.rand(segs.len() as u64) as usize;
            let k = rng.rand(N as u64) as usize;
            let mut arr = arrs[base].clone();

            if rng.gen_bool(0.5) {
                let x = rand_word(&mut rng);
                arr[k] = x.clone();
                segs.push(segs[base].set(k, x));
            } else {
                let x = rand_word(&mut rng);
                arr[k].extend_from_slice(&x);
                segs.push(segs[base].update(k, x));
            }
            arrs.push(arr);

            let version = rng.rand(segs.len() as u64) as usize;
            let index = rng.rand(N as u64) as usize;
            let (l, r) = rng.random(Wes(N));
            let expected = arrs[version][l..r]
                .iter()
                .flat_map(|word| word.iter().copied())
                .collect::<Vec<_>>();
            let expected_all = arrs[version]
                .iter()
                .flat_map(|word| word.iter().copied())
                .collect::<Vec<_>>();

            assert_eq!(segs[version].get(index), arrs[version][index]);
            assert_eq!(segs[version].fold(l..r), expected);
            assert_eq!(segs[version].fold_all(), expected_all);
        }
    }
}
