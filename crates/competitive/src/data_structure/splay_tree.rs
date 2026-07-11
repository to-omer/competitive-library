use super::{
    Allocator, MemoryPool,
    binary_search_tree::{
        BstDataAccess, BstDataMutRef, BstNode, BstRoot, BstSeeker, BstSpec, data,
        node::WithNoParent,
        seeker::{SeekByKey, SeekBySize},
        split::Split3,
    },
    splay_operations,
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::{ManuallyDrop, replace},
    ops::{DerefMut, RangeBounds},
    ptr::NonNull,
};

type SplayTreeRoot<K, V> = BstRoot<SplayTreeSpec<K, V>>;
type SplayTreeNode<K, V> = BstNode<SplayTreeData<K, V>>;

pub struct SplayTreeSpec<K, V> {
    _marker: PhantomData<fn() -> (K, V)>,
}

pub struct SplayTreeData<K, V> {
    key: K,
    value: V,
    size: usize,
}

impl<K, V> Debug for SplayTreeData<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplayTreeData")
            .field("key", &self.key)
            .field("value", &self.value)
            .field("size", &self.size)
            .finish()
    }
}

impl<K, V> BstDataAccess<data::marker::Key> for SplayTreeData<K, V> {
    type Value = K;

    fn bst_data(&self) -> &Self::Value {
        &self.key
    }

    fn bst_data_mut(&mut self) -> &mut Self::Value {
        &mut self.key
    }
}

impl<K, V> BstDataAccess<data::marker::Size> for SplayTreeData<K, V> {
    type Value = usize;

    fn bst_data(&self) -> &Self::Value {
        &self.size
    }

    fn bst_data_mut(&mut self) -> &mut Self::Value {
        &mut self.size
    }
}

impl<K, V> BstSpec for SplayTreeSpec<K, V> {
    type Parent = WithNoParent<Self::Data>;
    type Data = SplayTreeData<K, V>;

    #[inline]
    fn bottom_up(mut node: BstDataMutRef<'_, Self>) {
        let left = node
            .reborrow()
            .left()
            .descend()
            .map(|node| node.into_data().size)
            .unwrap_or_default();
        let right = node
            .reborrow()
            .right()
            .descend()
            .map(|node| node.into_data().size)
            .unwrap_or_default();
        node.data_mut().size = left + right + 1;
    }

    #[inline]
    fn merge(
        left: Option<SplayTreeRoot<K, V>>,
        right: Option<SplayTreeRoot<K, V>>,
    ) -> Option<SplayTreeRoot<K, V>> {
        splay_operations::merge(left, right)
    }

    #[inline]
    fn split<Seeker>(
        node: Option<SplayTreeRoot<K, V>>,
        seeker: Seeker,
        eq_left: bool,
    ) -> (Option<SplayTreeRoot<K, V>>, Option<SplayTreeRoot<K, V>>)
    where
        Seeker: BstSeeker<Spec = Self>,
    {
        splay_operations::split(node, seeker, eq_left)
    }
}

pub struct SplayTree<K, V, A = MemoryPool<SplayTreeNode<K, V>>>
where
    A: Allocator<SplayTreeNode<K, V>>,
{
    root: Option<SplayTreeRoot<K, V>>,
    length: usize,
    allocator: ManuallyDrop<A>,
}

impl<K, V, A> Debug for SplayTree<K, V, A>
where
    K: Debug,
    V: Debug,
    A: Allocator<SplayTreeNode<K, V>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplayTree")
            .field("length", &self.length)
            .finish_non_exhaustive()
    }
}

impl<K, V, A> Default for SplayTree<K, V, A>
where
    A: Allocator<SplayTreeNode<K, V>> + Default,
{
    fn default() -> Self {
        Self {
            root: None,
            length: 0,
            allocator: ManuallyDrop::new(A::default()),
        }
    }
}

impl<K, V, A> Drop for SplayTree<K, V, A>
where
    A: Allocator<SplayTreeNode<K, V>>,
{
    fn drop(&mut self) {
        unsafe {
            if let Some(root) = self.root.take() {
                root.into_dying().drop_all(self.allocator.deref_mut());
            }
            ManuallyDrop::drop(&mut self.allocator);
        }
    }
}

impl<K, V> SplayTree<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            root: None,
            length: 0,
            allocator: ManuallyDrop::new(MemoryPool::with_capacity(capacity)),
        }
    }
}

impl<K, V, A> SplayTree<K, V, A>
where
    A: Allocator<SplayTreeNode<K, V>>,
{
    #[inline]
    fn splay<Seeker>(&mut self, seeker: Seeker) -> Option<Ordering>
    where
        Seeker: BstSeeker<Spec = SplayTreeSpec<K, V>>,
    {
        let (ordering, root) = splay_operations::splay(self.root.take()?, seeker);
        self.root = Some(root);
        Some(ordering)
    }

    fn splay_by_key<Q>(&mut self, key: &Q) -> Option<Ordering>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.splay(SeekByKey::new(key))
    }

    fn splay_by_size(&mut self, index: usize) -> Option<Ordering> {
        self.splay(SeekBySize::new(index))
    }

    pub fn get<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_key_value(key).map(|(_, value)| value)
    }

    pub fn get_key_value<Q>(&mut self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        matches!(self.splay_by_key(key)?, Ordering::Equal).then(|| {
            let data = self.root.as_ref().unwrap().reborrow().into_data();
            (&data.key, &data.value)
        })
    }

    pub fn get_key_value_at(&mut self, index: usize) -> Option<(&K, &V)> {
        if index >= self.length {
            return None;
        }
        self.splay_by_size(index);
        let data = self.root.as_ref()?.reborrow().into_data();
        Some((&data.key, &data.value))
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        let ordering = self.splay_by_key(&key);
        if matches!(ordering, Some(Ordering::Equal)) {
            return Some(replace(
                &mut self
                    .root
                    .as_mut()
                    .unwrap()
                    .borrow_datamut()
                    .data_mut()
                    .value,
                value,
            ));
        }
        let mut node = BstRoot::from_data(
            SplayTreeData {
                key,
                value,
                size: 1,
            },
            self.allocator.deref_mut(),
        );
        if let Some(mut root) = self.root.take() {
            match ordering.unwrap() {
                Ordering::Greater => {
                    let left = unsafe { root.borrow_mut().left_mut().take() };
                    if let Some(left) = left {
                        unsafe { node.borrow_mut().left_mut().set(left) };
                    }
                    SplayTreeSpec::bottom_up(root.borrow_datamut());
                    unsafe { node.borrow_mut().right_mut().set(root) };
                }
                Ordering::Less => {
                    let right = unsafe { root.borrow_mut().right_mut().take() };
                    if let Some(right) = right {
                        unsafe { node.borrow_mut().right_mut().set(right) };
                    }
                    SplayTreeSpec::bottom_up(root.borrow_datamut());
                    unsafe { node.borrow_mut().left_mut().set(root) };
                }
                Ordering::Equal => unreachable!(),
            }
            SplayTreeSpec::bottom_up(node.borrow_datamut());
        }
        self.root = Some(node);
        self.length += 1;
        None
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if !matches!(self.splay_by_key(key)?, Ordering::Equal) {
            return None;
        }
        Some(self.remove_root().1)
    }

    pub fn remove_at(&mut self, index: usize) -> Option<(K, V)> {
        if index >= self.length {
            return None;
        }
        self.splay_by_size(index);
        Some(self.remove_root())
    }

    fn remove_root(&mut self) -> (K, V) {
        let mut node = self.root.take().unwrap();
        let left = unsafe { node.borrow_mut().left_mut().take() };
        let right = unsafe { node.borrow_mut().right_mut().take() };
        self.root = SplayTreeSpec::merge(left, right);
        self.length -= 1;
        let data = unsafe { node.into_dying().into_data(self.allocator.deref_mut()) };
        (data.key, data.value)
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn iter(&mut self) -> Iter<'_, K, V> {
        Iter::new(Split3::seek_by_size(&mut self.root, ..))
    }

    pub fn range<Q, R>(&mut self, range: R) -> Iter<'_, K, V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        Iter::new(Split3::seek_by_key(&mut self.root, range))
    }

    pub fn range_at<R>(&mut self, range: R) -> Iter<'_, K, V>
    where
        R: RangeBounds<usize>,
    {
        Iter::new(Split3::seek_by_size(&mut self.root, range))
    }
}

pub struct Iter<'a, K, V> {
    split: Split3<'a, SplayTreeSpec<K, V>>,
    front: Vec<NonNull<SplayTreeNode<K, V>>>,
    back: Vec<NonNull<SplayTreeNode<K, V>>>,
    remaining: usize,
}

impl<K, V> Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Iter")
            .field("remaining", &self.remaining)
            .finish_non_exhaustive()
    }
}

impl<'a, K, V> Iter<'a, K, V> {
    fn new(split: Split3<'a, SplayTreeSpec<K, V>>) -> Self {
        let remaining = split
            .mid()
            .map(|node| node.into_data().size)
            .unwrap_or_default();
        let mut iter = Self {
            split,
            front: vec![],
            back: vec![],
            remaining,
        };
        if let Some(root) = iter.split.mid() {
            Self::push_left(root.node, &mut iter.front);
            Self::push_right(root.node, &mut iter.back);
        }
        iter
    }

    fn push_left(
        mut node: NonNull<SplayTreeNode<K, V>>,
        stack: &mut Vec<NonNull<SplayTreeNode<K, V>>>,
    ) {
        loop {
            stack.push(node);
            let Some(left) = (unsafe { node.as_ref().child[0] }) else {
                break;
            };
            node = left;
        }
    }

    fn push_right(
        mut node: NonNull<SplayTreeNode<K, V>>,
        stack: &mut Vec<NonNull<SplayTreeNode<K, V>>>,
    ) {
        loop {
            stack.push(node);
            let Some(right) = (unsafe { node.as_ref().child[1] }) else {
                break;
            };
            node = right;
        }
    }
}

impl<K, V> Iterator for Iter<'_, K, V>
where
    K: Clone,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.front.pop().unwrap();
        if let Some(right) = unsafe { node.as_ref().child[1] } {
            Self::push_left(right, &mut self.front);
        }
        self.remaining -= 1;
        let data = unsafe { &node.as_ref().data };
        Some((data.key.clone(), data.value.clone()))
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    fn max(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V>
where
    K: Clone,
    V: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.back.pop().unwrap();
        if let Some(left) = unsafe { node.as_ref().child[0] } {
            Self::push_right(left, &mut self.back);
        }
        self.remaining -= 1;
        let data = unsafe { &node.as_ref().data };
        Some((data.key.clone(), data.value.clone()))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V>
where
    K: Clone,
    V: Clone,
{
}

impl<K, V> FusedIterator for Iter<'_, K, V>
where
    K: Clone,
    V: Clone,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::{
        cell::RefCell,
        collections::{BTreeMap, VecDeque},
        ops::Bound,
    };

    #[test]
    fn test_splay_tree() {
        const Q: usize = 30_000;
        const A: u64 = 500;
        let mut tree = SplayTree::new();
        let mut map = BTreeMap::new();
        let mut rng = Xorshift::default();
        for key in 0..A {
            map.insert(key, key as usize);
            tree.insert(key, key as usize);
        }
        for value in 0..Q {
            let key = rng.rand(A);
            match rng.rand(5) {
                0 => assert_eq!(map.remove(&key), tree.remove(&key)),
                1 => assert_eq!(map.insert(key, value), tree.insert(key, value)),
                2 => assert_eq!(map.get_key_value(&key), tree.get_key_value(&key)),
                3 => {
                    let index = rng.rand((map.len() + 1) as u64) as usize;
                    assert_eq!(map.iter().nth(index), tree.get_key_value_at(index));
                }
                _ => {
                    let index = rng.rand((map.len() + 1) as u64) as usize;
                    let key = map.iter().nth(index).map(|(&key, _)| key);
                    assert_eq!(
                        key.and_then(|key| map.remove_entry(&key)),
                        tree.remove_at(index)
                    );
                }
            }
            assert_eq!(map.len(), tree.len());
            assert_eq!(map.is_empty(), tree.is_empty());
            let expected = map
                .iter()
                .map(|(&key, &value)| (key, value))
                .collect::<Vec<_>>();
            assert_eq!(tree.iter().collect::<Vec<_>>(), expected);

            let key_range = {
                let left = rng.rand(A + 1);
                let right = rng.rand(A + 1);
                let (left, right) = (left.min(right), left.max(right));
                let start = match rng.rand(3) {
                    0 => Bound::Included(left),
                    1 => Bound::Excluded(left),
                    _ => Bound::Unbounded,
                };
                let end = match rng.rand(3) {
                    0 => Bound::Included(right),
                    1 if start == Bound::Excluded(right) => Bound::Included(right),
                    1 => Bound::Excluded(right),
                    _ => Bound::Unbounded,
                };
                (start, end)
            };
            assert_eq!(
                tree.range(key_range).collect::<Vec<_>>(),
                map.range(key_range)
                    .map(|(&key, &value)| (key, value))
                    .collect::<Vec<_>>()
            );

            let index_range = {
                let left = rng.rand((map.len() + 1) as u64) as usize;
                let right = rng.rand((map.len() + 1) as u64) as usize;
                let (left, right) = (left.min(right), left.max(right));
                let start = match rng.rand(3) {
                    0 => Bound::Included(left),
                    1 => Bound::Excluded(left),
                    _ => Bound::Unbounded,
                };
                let end = match rng.rand(3) {
                    0 => Bound::Included(right),
                    1 if start == Bound::Excluded(right) => Bound::Included(right),
                    1 => Bound::Excluded(right),
                    _ => Bound::Unbounded,
                };
                (start, end)
            };
            let left = match index_range.0 {
                Bound::Included(index) => index,
                Bound::Excluded(index) => (index + 1).min(expected.len()),
                Bound::Unbounded => 0,
            };
            let right = match index_range.1 {
                Bound::Included(index) => (index + 1).min(expected.len()),
                Bound::Excluded(index) => index,
                Bound::Unbounded => expected.len(),
            };
            assert_eq!(
                tree.range_at(index_range).collect::<Vec<_>>(),
                expected[left..right].to_vec()
            );
            assert_eq!(tree.iter().last(), expected.last().copied());
            assert_eq!(tree.iter().min(), expected.first().copied());
            assert_eq!(tree.iter().max(), expected.last().copied());

            let mut iter = tree.iter();
            let mut expected = VecDeque::from(expected);
            while !expected.is_empty() {
                if rng.rand(2) == 0 {
                    assert_eq!(iter.next(), expected.pop_front());
                } else {
                    assert_eq!(iter.next_back(), expected.pop_back());
                }
            }
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }
    }

    #[test]
    fn test_drop() {
        #[derive(Debug)]
        struct CheckDrop;
        thread_local! {
            static COUNT: RefCell<usize> = const { RefCell::new(0) };
        }
        impl Drop for CheckDrop {
            fn drop(&mut self) {
                COUNT.with(|count| *count.borrow_mut() += 1);
            }
        }
        {
            let mut tree = SplayTree::new();
            for key in 0..100 {
                tree.insert(key, CheckDrop);
            }
            for key in 0..50 {
                tree.remove(&key);
            }
            assert_eq!(COUNT.with(|count| *count.borrow()), 50);
        }
        assert_eq!(COUNT.with(|count| *count.borrow()), 100);
    }
}
