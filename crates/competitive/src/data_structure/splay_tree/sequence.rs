use super::{
    node::{marker, Node, NodeRange, NodeRef, Root, SplaySeeker, SplaySpec},
    Allocator, LazyMapMonoid, MemoryPool,
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
    T: LazyMapMonoid,
{
    key: T::Key,
    agg: T::Agg,
    lazy: T::Act,
    size: usize,
    rev: bool,
}

impl<T> Debug for LazyAggElement<T>
where
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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

struct SeekByAccCond<F, T>
where
    T: LazyMapMonoid,
{
    acc: T::Agg,
    f: F,
    _marker: PhantomData<fn() -> T>,
}
impl<F, T> SeekByAccCond<F, T>
where
    T: LazyMapMonoid,
{
    fn new(f: F) -> Self {
        Self {
            acc: T::agg_unit(),
            f,
            _marker: PhantomData,
        }
    }
}
impl<F, T> SplaySeeker for SeekByAccCond<F, T>
where
    F: FnMut(&T::Agg) -> bool,
    T: LazyMapMonoid,
{
    type S = LazyAggSplay<T>;
    fn splay_seek(&mut self, node: NodeRef<marker::Immut<'_>, Self::S>) -> Ordering {
        if let Some(lagg) = node.left().map(|l| &l.data().agg) {
            let nacc = T::agg_operate(&self.acc, lagg);
            if (self.f)(&nacc) {
                return Ordering::Less;
            }
            self.acc = nacc;
        };
        self.acc = T::agg_operate(&self.acc, &T::single_agg(&node.data().key));
        if (self.f)(&self.acc) {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

struct SeekByRaccCond<F, T>
where
    T: LazyMapMonoid,
{
    acc: T::Agg,
    f: F,
    _marker: PhantomData<fn() -> T>,
}
impl<F, T> SeekByRaccCond<F, T>
where
    T: LazyMapMonoid,
{
    fn new(f: F) -> Self {
        Self {
            acc: T::agg_unit(),
            f,
            _marker: PhantomData,
        }
    }
}
impl<F, T> SplaySeeker for SeekByRaccCond<F, T>
where
    F: FnMut(&T::Agg) -> bool,
    T: LazyMapMonoid,
{
    type S = LazyAggSplay<T>;
    fn splay_seek(&mut self, node: NodeRef<marker::Immut<'_>, Self::S>) -> Ordering {
        if let Some(lagg) = node.right().map(|r| &r.data().agg) {
            let nacc = T::agg_operate(lagg, &self.acc);
            if (self.f)(&nacc) {
                return Ordering::Greater;
            }
            self.acc = nacc;
        };
        self.acc = T::agg_operate(&T::single_agg(&node.data().key), &self.acc);
        if (self.f)(&self.acc) {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}

pub struct SplaySequence<T, A = MemoryPool<Node<LazyAggElement<T>>>>
where
    T: LazyMapMonoid,
    A: Allocator<Node<LazyAggElement<T>>>,
{
    root: Root<LazyAggSplay<T>>,
    length: usize,
    alloc: ManuallyDrop<A>,
}

impl<T, A> Debug for SplaySequence<T, A>
where
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}
impl<T, A> SplaySequence<T, A>
where
    T: LazyMapMonoid,
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
        if let Some(root) = self.range(range).root_mut().root_data_mut() {
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
        if let Some(root) = self.range(range).root_mut().root_data_mut() {
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
    pub fn position_acc<R, F>(&mut self, range: R, f: F) -> Option<usize>
    where
        R: RangeBounds<usize>,
        F: FnMut(&T::Agg) -> bool,
    {
        let mut range = self.range(range);
        let ord = range.root_mut().splay_by(SeekByAccCond::new(f));
        if !matches!(ord, Some(Ordering::Equal)) {
            return None;
        }
        let front_size = range.front().size();
        let left_size = range.root().left_size();
        Some(front_size + left_size)
    }
    pub fn rposition_acc<R, F>(&mut self, range: R, f: F) -> Option<usize>
    where
        R: RangeBounds<usize>,
        F: FnMut(&T::Agg) -> bool,
    {
        let mut range = self.range(range);
        let ord = range.root_mut().splay_by(SeekByRaccCond::new(f));
        if !matches!(ord, Some(Ordering::Equal)) {
            return None;
        }
        let front_size = range.front().size();
        let left_size = range.root().left_size();
        Some(front_size + left_size)
    }
    pub fn rotate_left(&mut self, mid: usize) {
        assert!(mid <= self.length);
        if mid != 0 || mid != self.length {
            self.range(mid..).drop_rotate_left()
        }
    }
    pub fn rotate_right(&mut self, k: usize) {
        assert!(k <= self.length);
        self.rotate_left(self.length - k);
    }
}

impl<T, A> Extend<T::Key> for SplaySequence<T, A>
where
    T: LazyMapMonoid,
    A: Allocator<Node<LazyAggElement<T>>>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T::Key>,
    {
        let nodes: Vec<_> = unsafe {
            iter.into_iter()
                .map(|key| {
                    let agg = T::single_agg(&key);
                    NodeRef::from_data(
                        LazyAggElement {
                            key,
                            agg,
                            lazy: T::act_unit(),
                            size: 1,
                            rev: false,
                        },
                        self.alloc.deref_mut(),
                    )
                })
                .collect()
        };
        self.length += nodes.len();
        let mut root = unsafe { Root::from_single_nodes(nodes) };
        self.root.append(&mut root);
    }
}

impl<T> Root<LazyAggSplay<T>>
where
    T: LazyMapMonoid,
{
    fn size(&self) -> usize {
        self.root().map(|root| root.data().size).unwrap_or_default()
    }
    fn left_size(&self) -> usize {
        self.root()
            .and_then(|root| root.left().map(|left| left.data().size))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::RangeMaxRangeUpdate,
        rand,
        tools::{NotEmptySegment, Xorshift},
    };

    #[test]
    fn test_splay_sequence() {
        const N: usize = 1_000;
        const Q: usize = 20_000;
        const A: i64 = 1_000_000_000;

        let mut rng = Xorshift::default();
        rand!(rng, mut arr: [-A..A; N]);
        let mut seq = SplaySequence::<RangeMaxRangeUpdate<_>>::new();
        seq.extend(arr.iter().cloned());
        for _ in 0..Q {
            assert_eq!(arr.len(), seq.len());
            rand!(rng, ty: 0..6, (l, r): NotEmptySegment(arr.len()));
            match ty {
                0 if arr.len() < N * 2 => {
                    rand!(rng, i: ..=arr.len(), x: -A..A);
                    seq.insert(i, x);
                    arr.insert(i, x);
                }
                1 if arr.len() > 1 => {
                    rand!(rng, i: ..arr.len());
                    assert_eq!(arr.remove(i), seq.remove(i).unwrap());
                }
                2 => {
                    let res = arr[l..r].iter().max().cloned().unwrap_or_default();
                    assert_eq!(seq.fold(l..r), res);
                }
                3 => {
                    rand!(rng, x: -A..A);
                    seq.update(l..r, Some(x));
                    arr[l..r].iter_mut().for_each(|a| *a = x);
                }
                4 => {
                    arr[l..r].reverse();
                    seq.reverse(l..r);
                }
                5 => {
                    rand!(rng, x: -A..A);
                    assert_eq!(
                        seq.position_acc(l..r, |&d| d >= x),
                        arr[l..r]
                            .iter()
                            .scan(i64::MIN, |acc, &a| {
                                *acc = a.max(*acc);
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map(|i| i + l),
                    );
                }
                6 => {
                    rand!(rng, x: -A..A);
                    assert_eq!(
                        seq.rposition_acc(l..r, |&d| d >= x),
                        arr[l..r]
                            .iter()
                            .rev()
                            .scan(i64::MIN, |acc, &a| {
                                *acc = a.max(*acc);
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map(|i| r - i - 1),
                    );
                }
                7 => {
                    rand!(rng, i: ..=arr.len());
                    seq.rotate_left(i);
                    arr.rotate_left(i);
                }
                8 => {
                    rand!(rng, i: ..=arr.len());
                    seq.rotate_right(i);
                    arr.rotate_right(i);
                }
                _ => {
                    rand!(rng, i: ..arr.len());
                    assert_eq!(arr.get(i), seq.get(i));
                }
            }
        }
    }
}
