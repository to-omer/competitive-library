use super::{
    node::{marker, Node, NodeRange, NodeRef, Root, SplaySeeker, SplaySpec},
    Allocator, MemoryPool,
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::{replace, ManuallyDrop},
    ops::{Bound, DerefMut, RangeBounds},
};

struct SizedSplay<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> SplaySpec for SizedSplay<T> {
    type T = (T, usize);
    fn has_bottom_up() -> bool {
        true
    }
    fn bottom_up(node: NodeRef<marker::DataMut<'_>, Self>) {
        let l = node.left().map(|p| p.data().1).unwrap_or_default();
        let r = node.right().map(|p| p.data().1).unwrap_or_default();
        node.data_mut().1 = l + r + 1;
    }
}

struct SeekByKey<'a, K, V, Q>
where
    Q: ?Sized,
{
    key: &'a Q,
    _marker: PhantomData<fn() -> (K, V)>,
}
impl<'a, K, V, Q> SeekByKey<'a, K, V, Q>
where
    Q: ?Sized,
{
    fn new(key: &'a Q) -> Self {
        Self {
            key,
            _marker: PhantomData,
        }
    }
}
impl<'a, K, V, Q> SplaySeeker for SeekByKey<'a, K, V, Q>
where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
{
    type S = SizedSplay<(K, V)>;
    fn splay_seek(&mut self, node: NodeRef<marker::Immut<'_>, Self::S>) -> Ordering {
        self.key.cmp((node.data().0).0.borrow())
    }
}

struct SeekBySize<K, V> {
    index: usize,
    _marker: PhantomData<fn() -> (K, V)>,
}
impl<K, V> SeekBySize<K, V> {
    fn new(index: usize) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }
}
impl<K, V> SplaySeeker for SeekBySize<K, V> {
    type S = SizedSplay<(K, V)>;
    fn splay_seek(&mut self, node: NodeRef<marker::Immut<'_>, Self::S>) -> Ordering {
        let lsize = node.left().map(|l| l.data().1).unwrap_or_default();
        let ord = self.index.cmp(&lsize);
        if matches!(ord, Ordering::Greater) {
            self.index -= lsize + 1;
        }
        ord
    }
}

pub struct SplayMap<K, V, A = MemoryPool<Node<((K, V), usize)>>>
where
    A: Allocator<Node<((K, V), usize)>>,
{
    root: Root<SizedSplay<(K, V)>>,
    length: usize,
    alloc: ManuallyDrop<A>,
}

impl<K, V, A> Debug for SplayMap<K, V, A>
where
    K: Debug,
    V: Debug,
    A: Allocator<Node<((K, V), usize)>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplayMap")
            .field("root", &self.root)
            .field("length", &self.length)
            .finish()
    }
}

impl<K, V, A> Drop for SplayMap<K, V, A>
where
    A: Allocator<Node<((K, V), usize)>>,
{
    fn drop(&mut self) {
        unsafe {
            while let Some(node) = self.root.take_first() {
                self.alloc.deallocate(node.into_dying().into_inner());
            }
            ManuallyDrop::drop(&mut self.alloc);
        }
    }
}

impl<K, V, A> Default for SplayMap<K, V, A>
where
    A: Allocator<Node<((K, V), usize)>> + Default,
{
    fn default() -> Self {
        Self {
            root: Root::default(),
            length: 0,
            alloc: Default::default(),
        }
    }
}

impl<K, V> SplayMap<K, V> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            root: Root::default(),
            length: 0,
            alloc: ManuallyDrop::new(MemoryPool::with_capacity(capacity)),
        }
    }
}
impl<K, V, A> SplayMap<K, V, A>
where
    A: Allocator<Node<((K, V), usize)>>,
{
    pub fn get<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_key_value(key).map(|(_, v)| v)
    }
    fn splay_by_key<Q>(&mut self, key: &Q) -> Option<Ordering>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root.splay_by(SeekByKey::new(key))
    }
    pub fn get_key_value<Q>(&mut self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if !matches!(self.splay_by_key(key)?, Ordering::Equal) {
            return None;
        }
        self.root.root().map(|node| {
            let ((k, v), _) = node.data();
            (k, v)
        })
    }
    fn splay_at(&mut self, index: usize) -> Option<Ordering> {
        self.root.splay_by(SeekBySize::new(index))
    }
    pub fn get_key_value_at(&mut self, index: usize) -> Option<(&K, &V)> {
        if index >= self.length {
            return None;
        }
        self.splay_at(index);
        self.root.root().map(|node| {
            let ((k, v), _) = node.data();
            (k, v)
        })
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        let ord = self.splay_by_key(&key);
        self.length += (ord != Some(Ordering::Equal)) as usize;
        match ord {
            Some(Ordering::Equal) => {
                return Some(replace(
                    &mut (self.root.root_data_mut().unwrap().data_mut().0).1,
                    value,
                ))
            }
            Some(Ordering::Less) => unsafe {
                self.root.insert_left(NodeRef::from_data(
                    ((key, value), 1),
                    self.alloc.deref_mut(),
                ));
            },
            _ => unsafe {
                self.root.insert_right(NodeRef::from_data(
                    ((key, value), 1),
                    self.alloc.deref_mut(),
                ));
            },
        }
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
        self.length -= 1;
        let node = self.root.take_root().unwrap().into_dying();
        unsafe { Some((node.into_data(self.alloc.deref_mut()).0).1) }
    }
    pub fn remove_at(&mut self, index: usize) -> Option<(K, V)> {
        if index >= self.length {
            return None;
        }
        self.splay_at(index);
        self.length -= 1;
        let node = self.root.take_root().unwrap().into_dying();
        unsafe { Some(node.into_data(self.alloc.deref_mut()).0) }
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(&mut self) -> Iter<'_, K, V> {
        Iter {
            iter: NodeRange::new(&mut self.root),
        }
    }
    pub fn range<Q, R>(&mut self, range: R) -> Iter<'_, K, V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let start = match range.start_bound() {
            Bound::Included(key) => Bound::Included(SeekByKey::new(key)),
            Bound::Excluded(key) => Bound::Excluded(SeekByKey::new(key)),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match range.end_bound() {
            Bound::Included(key) => Bound::Included(SeekByKey::new(key)),
            Bound::Excluded(key) => Bound::Excluded(SeekByKey::new(key)),
            Bound::Unbounded => Bound::Unbounded,
        };
        Iter {
            iter: self.root.range(start, end),
        }
    }
    pub fn range_at<R>(&mut self, range: R) -> Iter<'_, K, V>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(&index) => Bound::Included(SeekBySize::new(index)),
            Bound::Excluded(&index) => Bound::Excluded(SeekBySize::new(index)),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match range.end_bound() {
            Bound::Included(&index) => Bound::Included(SeekBySize::new(index)),
            Bound::Excluded(&index) => Bound::Excluded(SeekBySize::new(index)),
            Bound::Unbounded => Bound::Unbounded,
        };
        Iter {
            iter: self.root.range(start, end),
        }
    }
}

#[derive(Debug)]
pub struct Iter<'a, K, V> {
    iter: NodeRange<'a, SizedSplay<(K, V)>>,
}
impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Clone,
    V: Clone,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_checked().map(|node| node.data().0.clone())
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
}
impl<K, V> FusedIterator for Iter<'_, K, V>
where
    K: Clone,
    V: Clone,
{
}
impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V>
where
    K: Clone,
    V: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back_checked()
            .map(|node| node.data().0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::{cell::RefCell, collections::BTreeMap, mem::swap};

    impl<K, V> SplayMap<K, V> {
        fn dump(&self) -> Vec<(K, V)>
        where
            K: Clone,
            V: Clone,
        {
            let mut arr = vec![];
            if let Some(root) = self.root.root() {
                root.traverse(&mut |node| {
                    arr.push(node.data().0.clone());
                })
            }
            arr
        }
        fn check_size(&self) -> bool {
            fn dfs<T>(node: NodeRef<marker::Immut<'_>, SizedSplay<T>>) -> Option<usize> {
                let mut size = 1usize;
                if let Some(node) = node.left() {
                    size += dfs(node)?;
                }
                if let Some(node) = node.right() {
                    size += dfs(node)?;
                }
                if size == node.data().1 {
                    Some(size)
                } else {
                    None
                }
            }
            if let Some(root) = self.root.root() {
                dfs(root) == Some(self.len())
            } else {
                true
            }
        }
    }

    #[test]
    fn test_insert_remove_get() {
        const Q: usize = 30_000;
        const A: u64 = 500;
        let mut stree = SplayMap::new();
        let mut btree = BTreeMap::new();
        let mut rng = Xorshift::default();
        for v in 1..=Q {
            let k = rng.rand(A);
            match rng.gen(0..3) {
                0 => assert_eq!(btree.remove(&k), stree.remove(&k)),
                1 => assert_eq!(btree.insert(k, v), stree.insert(k, v)),
                _ => assert_eq!(btree.get_key_value(&k), stree.get_key_value(&k)),
            }
            assert_eq!(btree.len(), stree.len());
            assert!(stree.check_size());
        }
    }

    #[test]
    fn test_at() {
        const Q: usize = 30_000;
        const A: u64 = 500;
        let mut stree = SplayMap::new();
        let mut btree = BTreeMap::new();
        let mut rng = Xorshift::default();
        for v in 1..=Q {
            let k = rng.rand(A);
            let i = rng.gen(0..=btree.len());
            match rng.gen(0..3) {
                0 => {
                    if let Some((&k, _)) = btree.iter().nth(i) {
                        assert_eq!(btree.remove(&k).map(|v| (k, v)), stree.remove_at(i));
                    }
                }
                1 => assert_eq!(btree.insert(k, v), stree.insert(k, v)),
                _ => assert_eq!(btree.iter().nth(i), stree.get_key_value_at(i)),
            }
            assert_eq!(btree.len(), stree.len());
            assert!(stree.check_size());
        }
    }

    #[test]
    fn test_iter() {
        const Q: usize = 3_000;
        const A: u64 = 100;
        let mut stree = SplayMap::new();
        let mut btree = BTreeMap::new();
        let mut rng = Xorshift::default();
        for v in 1..=Q {
            for v in v * 100..(v + 1) * 100 {
                let k = rng.rand(A);
                match rng.gen(0..2) {
                    0 => assert_eq!(btree.remove(&k), stree.remove(&k)),
                    _ => assert_eq!(btree.insert(k, v), stree.insert(k, v)),
                }
            }

            let b: Vec<_> = btree.iter().map(|(k, v)| (*k, *v)).collect();
            let a = stree.dump();
            assert_eq!(b, a);

            match rng.gen(0..3) {
                0 => {
                    let a: Vec<_> = if rng.gen(0..2) == 0 {
                        stree.iter().collect()
                    } else {
                        let mut a: Vec<_> = stree.iter().rev().collect();
                        a.reverse();
                        a
                    };
                    assert_eq!(b, a);
                }
                1 if !stree.is_empty() => {
                    let (mut l, mut r) = (rng.gen(0..=stree.len()), rng.gen(0..=stree.len()));
                    if l > r {
                        swap(&mut l, &mut r);
                    }
                    let l = match rng.gen(0..3) {
                        0 => Bound::Included(l),
                        1 => Bound::Excluded(l),
                        _ => Bound::Unbounded,
                    };
                    let r = match rng.gen(0..3) {
                        0 => Bound::Included(r),
                        1 => Bound::Excluded(r),
                        _ => Bound::Unbounded,
                    };
                    let a: Vec<_> = stree.range_at((l, r)).collect();
                    let lc = match l {
                        Bound::Included(l) => l,
                        Bound::Excluded(l) => l + 1,
                        Bound::Unbounded => 0,
                    };
                    let rc = match r {
                        Bound::Included(r) => r + 1,
                        Bound::Excluded(r) => r,
                        Bound::Unbounded => !0,
                    };
                    let b: Vec<_> = btree
                        .iter()
                        .take(rc)
                        .skip(lc)
                        .map(|(k, v)| (*k, *v))
                        .collect();
                    assert_eq!(b, a);
                }
                _ => {
                    let (mut l, mut r) = (rng.gen(0..=A), rng.gen(0..=A));
                    if l > r {
                        swap(&mut l, &mut r);
                    }
                    let l = match rng.gen(0..3) {
                        0 => Bound::Included(l),
                        1 => Bound::Excluded(l),
                        _ => Bound::Unbounded,
                    };
                    let r = match rng.gen(0..3) {
                        0 => Bound::Included(r),
                        1 if l == Bound::Excluded(r) => Bound::Excluded(r + 1),
                        1 => Bound::Excluded(r),
                        _ => Bound::Unbounded,
                    };
                    let b: Vec<_> = btree.range((l, r)).map(|(k, v)| (*k, *v)).collect();
                    let a: Vec<_> = stree.range((l, r)).collect();
                    assert_eq!(b, a);
                }
            }
            assert_eq!(btree.len(), stree.len());
            assert!(stree.check_size());

            let b: Vec<_> = btree.iter().map(|(k, v)| (*k, *v)).collect();
            let a = stree.dump();
            assert_eq!(b, a);
        }
    }

    #[test]
    fn test_drop() {
        #[derive(Debug)]
        struct CheckDrop<T>(T);
        thread_local! {
            static CNT: RefCell<usize> = const { RefCell::new(0) };
        }
        impl<T> Drop for CheckDrop<T> {
            fn drop(&mut self) {
                CNT.with(|cnt| *cnt.borrow_mut() += 1);
            }
        }
        const Q: usize = 3_000;
        const A: u64 = 500;
        let mut cnt = 0usize;
        let mut rng = Xorshift::default();
        for _ in 0..10 {
            {
                let mut stree = SplayMap::new();
                for v in 0..Q {
                    {
                        let k = rng.rand(A);
                        cnt += stree.remove(&k).is_some() as usize;
                        let k = rng.rand(A);
                        cnt += stree.insert(k, CheckDrop(v)).is_some() as usize;
                    }
                    assert_eq!(cnt, CNT.with(|cnt| *cnt.borrow()));
                }
                cnt += stree.len();
            }
            assert_eq!(cnt, CNT.with(|cnt| *cnt.borrow()));
        }
    }
}
