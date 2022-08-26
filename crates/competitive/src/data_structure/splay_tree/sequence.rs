use super::{
    node::{marker, Node, NodeRange, NodeRef, Root, SplaySeeker, SplaySpec},
    Allocator, MemoryPool, MonoidAction,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{replace, ManuallyDrop},
    ops::{Bound, DerefMut, RangeBounds},
};

pub struct LazyAggElement<T>
where
    T: MonoidAction,
{
    key: T::Key,
    agg: T::Agg,
    lazy: T::Act,
    size: usize,
    rev: bool,
}

impl<T> Debug for LazyAggElement<T>
where
    T: MonoidAction,
    T::Key: Debug,
    T::Agg: Debug,
    T::Act: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyAggElement")
            .field("key", &self.key)
            .field("agg", &self.agg)
            .field("lazy", &self.lazy)
            .field("size", &self.size)
            .finish()
    }
}

pub struct LazyAggSplay<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> LazyAggSplay<T>
where
    T: MonoidAction,
{
    pub fn update_lazy(mut node: NodeRef<marker::DataMut<'_>, Self>, lazy: &T::Act) {
        T::act_operate_assign(&mut node.data_mut().lazy, lazy);
        node.data_mut().key = T::act_key(&node.data().key, lazy);
        if let Some(nxlazy) = T::act_agg(&node.data().agg, lazy) {
            node.data_mut().agg = nxlazy;
        } else {
            node = Self::propagate(node);
            Self::recalc(node);
        }
    }
    pub fn reverse(node: NodeRef<marker::DataMut<'_>, Self>) {
        node.reverse();
        T::toggle(&mut node.data_mut().agg);
        node.data_mut().rev ^= true;
    }
    fn propagate(node: NodeRef<marker::DataMut<'_>, Self>) -> NodeRef<marker::DataMut<'_>, Self> {
        let lazy = replace(&mut node.data_mut().lazy, T::act_unit());
        if let Some(left) = node.left() {
            Self::update_lazy(left, &lazy);
        }
        if let Some(right) = node.right() {
            Self::update_lazy(right, &lazy);
        }
        if replace(&mut node.data_mut().rev, false) {
            if let Some(left) = node.left() {
                Self::reverse(left);
            }
            if let Some(right) = node.right() {
                Self::reverse(right);
            }
        }
        node
    }
    fn recalc(node: NodeRef<marker::DataMut<'_>, Self>) -> NodeRef<marker::DataMut<'_>, Self> {
        let mut agg = T::single_agg(&node.data().key);
        let mut size = 1;
        if let Some(left) = node.left() {
            let data = left.data();
            agg = T::agg_operate(&data.agg, &agg);
            // agg = <T::AggMonoid as Magma>::operate(&data.agg, &agg);
            size += data.size;
        }
        if let Some(right) = node.right() {
            let data = right.data();
            agg = T::agg_operate(&agg, &data.agg);
            size += data.size;
        }
        let data = node.data_mut();
        data.agg = agg;
        data.size = size;
        node
    }
}
impl<T> SplaySpec for LazyAggSplay<T>
where
    T: MonoidAction,
{
    type T = LazyAggElement<T>;
    fn has_bottom_up() -> bool {
        true
    }
    fn top_down(node: NodeRef<marker::DataMut<'_>, Self>) {
        Self::propagate(node);
    }
    fn bottom_up(node: NodeRef<marker::DataMut<'_>, Self>) {
        Self::recalc(node);
    }
}

struct SeekBySize<T> {
    index: usize,
    _marker: PhantomData<fn() -> T>,
}
impl<T> SeekBySize<T> {
    fn new(index: usize) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }
}
impl<T> SplaySeeker for SeekBySize<T>
where
    T: MonoidAction,
{
    type S = LazyAggSplay<T>;
    fn splay_seek(&mut self, node: NodeRef<marker::Immut<'_>, Self::S>) -> Ordering {
        let lsize = node.left().map(|l| l.data().size).unwrap_or_default();
        let ord = self.index.cmp(&lsize);
        if matches!(ord, Ordering::Greater) {
            self.index -= lsize + 1;
        }
        ord
    }
}

pub struct SplaySequence<T, A = MemoryPool<Node<LazyAggElement<T>>>>
where
    T: MonoidAction,
    A: Allocator<Node<LazyAggElement<T>>>,
{
    root: Root<LazyAggSplay<T>>,
    length: usize,
    alloc: ManuallyDrop<A>,
}

impl<T, A> Debug for SplaySequence<T, A>
where
    T: MonoidAction,
    T::Key: Debug,
    T::Agg: Debug,
    T::Act: Debug,
    A: Allocator<Node<LazyAggElement<T>>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplayMap")
            .field("root", &self.root)
            .field("length", &self.length)
            .finish()
    }
}

impl<T, A> Drop for SplaySequence<T, A>
where
    T: MonoidAction,
    A: Allocator<Node<LazyAggElement<T>>>,
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

impl<T, A> Default for SplaySequence<T, A>
where
    T: MonoidAction,
    A: Allocator<Node<LazyAggElement<T>>> + Default,
{
    fn default() -> Self {
        Self {
            root: Root::default(),
            length: 0,
            alloc: Default::default(),
        }
    }
}

impl<T> SplaySequence<T>
where
    T: MonoidAction,
{
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
impl<T, A> SplaySequence<T, A>
where
    T: MonoidAction,
    A: Allocator<Node<LazyAggElement<T>>>,
{
    fn range<R>(&mut self, range: R) -> NodeRange<'_, LazyAggSplay<T>>
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
        self.root.range(start, end)
    }
    pub fn update<R>(&mut self, range: R, x: T::Act)
    where
        R: RangeBounds<usize>,
    {
        if let Some(root) = self.range(range).root().root_data_mut() {
            LazyAggSplay::<T>::update_lazy(root, &x);
        }
    }
    pub fn fold<R>(&mut self, range: R) -> T::Agg
    where
        R: RangeBounds<usize>,
    {
        if let Some(root) = self.range(range).root().root() {
            root.data().agg.clone()
        } else {
            T::agg_unit()
        }
    }
    pub fn reverse<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        if let Some(root) = self.range(range).root().root_data_mut() {
            LazyAggSplay::<T>::reverse(root);
        }
    }
    pub fn get(&mut self, index: usize) -> Option<&T::Key> {
        self.root.splay_by(SeekBySize::new(index))?;
        self.root.root().map(|root| &root.data().key)
    }
    pub fn modify<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&T::Key) -> T::Key,
    {
        self.root.splay_by(SeekBySize::new(index)).unwrap();
        let data = self.root.root_data_mut().unwrap().data_mut();
        data.key = f(&data.key);
        LazyAggSplay::<T>::bottom_up(self.root.root_data_mut().unwrap());
    }
    pub fn insert(&mut self, index: usize, x: T::Key) {
        assert!(index <= self.length);
        self.root.splay_by(SeekBySize::new(index));
        let agg = T::single_agg(&x);
        unsafe {
            let node = NodeRef::from_data(
                LazyAggElement {
                    key: x,
                    agg,
                    lazy: T::act_unit(),
                    size: 1,
                    rev: false,
                },
                self.alloc.deref_mut(),
            );
            if index == self.length {
                self.root.insert_right(node);
            } else {
                self.root.insert_left(node);
            }
        }
        self.length += 1;
    }
    pub fn remove(&mut self, index: usize) -> Option<T::Key> {
        if index >= self.length {
            return None;
        }
        self.root.splay_by(SeekBySize::new(index));
        self.length -= 1;
        let node = self.root.take_root().unwrap().into_dying();
        unsafe { Some(node.into_data(self.alloc.deref_mut()).key) }
    }
}
