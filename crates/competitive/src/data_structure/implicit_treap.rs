use super::{
    Allocator, LazyMapMonoid, MemoryPool, Xorshift,
    binary_search_tree::{
        BstDataAccess, BstDataMutRef, BstNode, BstRoot, BstSeeker, BstSpec,
        data::{self, LazyMapElement},
        node::WithNoParent,
        seeker::{SeekByAccCond, SeekByRaccCond, SeekBySize},
        split::{Split, Split3},
    },
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{ManuallyDrop, replace},
    ops::{DerefMut, RangeBounds},
    ptr::NonNull,
};

type ImplicitTreapRoot<T> = BstRoot<ImplicitTreapSpec<T>>;
type ImplicitTreapNode<T> = BstNode<ImplicitTreapData<T>>;

pub struct ImplicitTreapSpec<T> {
    _marker: PhantomData<fn() -> T>,
}

pub struct ImplicitTreapData<T>
where
    T: LazyMapMonoid,
{
    priority: u64,
    value: LazyMapElement<T>,
    size: usize,
    rev: bool,
}

impl<T> Debug for ImplicitTreapData<T>
where
    T: LazyMapMonoid<Key: Debug, Agg: Debug, Act: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImplicitTreapData")
            .field("priority", &self.priority)
            .field("value", &self.value)
            .field("size", &self.size)
            .field("rev", &self.rev)
            .finish()
    }
}

impl<T> BstDataAccess<data::marker::Size> for ImplicitTreapData<T>
where
    T: LazyMapMonoid,
{
    type Value = usize;

    fn bst_data(&self) -> &Self::Value {
        &self.size
    }

    fn bst_data_mut(&mut self) -> &mut Self::Value {
        &mut self.size
    }
}

impl<T> BstDataAccess<data::marker::LazyMap> for ImplicitTreapData<T>
where
    T: LazyMapMonoid,
{
    type Value = LazyMapElement<T>;

    fn bst_data(&self) -> &Self::Value {
        &self.value
    }

    fn bst_data_mut(&mut self) -> &mut Self::Value {
        &mut self.value
    }
}

impl<T> ImplicitTreapSpec<T>
where
    T: LazyMapMonoid,
{
    fn update_act(mut node: BstDataMutRef<'_, Self>, act: &T::Act) {
        T::act_operate_assign(&mut node.data_mut().value.act, act);
        node.data_mut().value.key = T::act_key(&node.reborrow().into_data().value.key, act);
        if let Some(agg) = T::act_agg(&node.reborrow().into_data().value.agg, act) {
            node.data_mut().value.agg = agg;
        } else {
            Self::top_down(node.reborrow_datamut());
            Self::bottom_up(node);
        }
    }

    fn reverse(mut node: BstDataMutRef<'_, Self>) {
        unsafe {
            node.node.as_mut().child.swap(0, 1);
        }
        let data = node.data_mut();
        T::toggle(&mut data.value.agg);
        data.rev ^= true;
    }
}

impl<T> BstSpec for ImplicitTreapSpec<T>
where
    T: LazyMapMonoid,
{
    type Parent = WithNoParent<Self::Data>;
    type Data = ImplicitTreapData<T>;

    fn top_down(mut node: BstDataMutRef<'_, Self>) {
        let act = replace(&mut node.data_mut().value.act, T::act_unit());
        if let Ok(left) = node.reborrow_datamut().left().descend() {
            Self::update_act(left, &act);
        }
        if let Ok(right) = node.reborrow_datamut().right().descend() {
            Self::update_act(right, &act);
        }
        if replace(&mut node.data_mut().rev, false) {
            if let Ok(left) = node.reborrow_datamut().left().descend() {
                Self::reverse(left);
            }
            if let Ok(right) = node.reborrow_datamut().right().descend() {
                Self::reverse(right);
            }
        }
    }

    fn bottom_up(mut node: BstDataMutRef<'_, Self>) {
        let mut agg = T::single_agg(&node.reborrow().into_data().value.key);
        let mut size = 1;
        if let Ok(left) = node.reborrow().left().descend() {
            let data = left.into_data();
            agg = T::agg_operate(&data.value.agg, &agg);
            size += data.size;
        }
        if let Ok(right) = node.reborrow().right().descend() {
            let data = right.into_data();
            agg = T::agg_operate(&agg, &data.value.agg);
            size += data.size;
        }
        let data = node.data_mut();
        data.value.agg = agg;
        data.size = size;
    }

    fn merge(
        left: Option<ImplicitTreapRoot<T>>,
        right: Option<ImplicitTreapRoot<T>>,
    ) -> Option<ImplicitTreapRoot<T>> {
        match (left, right) {
            (None, None) => None,
            (None, Some(node)) | (Some(node), None) => Some(node),
            (Some(mut left), Some(mut right)) => unsafe {
                if left.reborrow().into_data().priority > right.reborrow().into_data().priority {
                    Self::top_down(left.borrow_datamut());
                    let lr = left.borrow_mut().right().take();
                    let lr = Self::merge(lr, Some(right)).unwrap_unchecked();
                    left.borrow_mut().right().set(lr);
                    Self::bottom_up(left.borrow_datamut());
                    Some(left)
                } else {
                    Self::top_down(right.borrow_datamut());
                    let rl = right.borrow_mut().left().take();
                    let rl = Self::merge(Some(left), rl).unwrap_unchecked();
                    right.borrow_mut().left().set(rl);
                    Self::bottom_up(right.borrow_datamut());
                    Some(right)
                }
            },
        }
    }

    fn split<Seeker>(
        node: Option<ImplicitTreapRoot<T>>,
        mut seeker: Seeker,
        eq_left: bool,
    ) -> (Option<ImplicitTreapRoot<T>>, Option<ImplicitTreapRoot<T>>)
    where
        Seeker: BstSeeker<Spec = Self>,
    {
        match node {
            None => (None, None),
            Some(mut node) => {
                Self::top_down(node.borrow_datamut());
                let seek_left = match seeker.bst_seek(node.reborrow()) {
                    Ordering::Less => false,
                    Ordering::Equal => !eq_left,
                    Ordering::Greater => true,
                };
                if seek_left {
                    unsafe {
                        let left = node.borrow_mut().left().take();
                        let (l, r) = Self::split(left, seeker, eq_left);
                        if let Some(r) = r {
                            node.borrow_mut().left().set(r);
                        }
                        Self::bottom_up(node.borrow_datamut());
                        (l, Some(node))
                    }
                } else {
                    unsafe {
                        let right = node.borrow_mut().right().take();
                        let (l, r) = Self::split(right, seeker, eq_left);
                        if let Some(l) = l {
                            node.borrow_mut().right().set(l);
                        }
                        Self::bottom_up(node.borrow_datamut());
                        (Some(node), r)
                    }
                }
            }
        }
    }
}

pub struct ImplicitTreap<T, A = MemoryPool<ImplicitTreapNode<T>>>
where
    T: LazyMapMonoid,
    A: Allocator<ImplicitTreapNode<T>>,
{
    root: Option<ImplicitTreapRoot<T>>,
    length: usize,
    rng: Xorshift,
    allocator: ManuallyDrop<A>,
    _marker: PhantomData<fn() -> T>,
}

impl<T, A> Default for ImplicitTreap<T, A>
where
    T: LazyMapMonoid,
    A: Allocator<ImplicitTreapNode<T>> + Default,
{
    fn default() -> Self {
        Self {
            root: None,
            length: 0,
            rng: Xorshift::new(),
            allocator: ManuallyDrop::new(A::default()),
            _marker: PhantomData,
        }
    }
}

impl<T, A> Drop for ImplicitTreap<T, A>
where
    T: LazyMapMonoid,
    A: Allocator<ImplicitTreapNode<T>>,
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

impl<T> ImplicitTreap<T>
where
    T: LazyMapMonoid,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            root: None,
            length: 0,
            rng: Xorshift::new(),
            allocator: ManuallyDrop::new(MemoryPool::with_capacity(capacity)),
            _marker: PhantomData,
        }
    }
}

impl<T, A> ImplicitTreap<T, A>
where
    T: LazyMapMonoid,
    A: Allocator<ImplicitTreapNode<T>>,
{
    fn node(&mut self, key: T::Key) -> ImplicitTreapRoot<T> {
        BstRoot::from_data(
            ImplicitTreapData {
                priority: self.rng.rand64(),
                value: LazyMapElement::from_key(key),
                size: 1,
                rev: false,
            },
            self.allocator.deref_mut(),
        )
    }

    fn build<I>(&mut self, iter: I) -> (Option<ImplicitTreapRoot<T>>, usize)
    where
        I: IntoIterator<Item = T::Key>,
    {
        let mut stack = vec![];
        let mut len = 0;
        for key in iter {
            let mut cur = self.node(key).node;
            let mut left = None;
            unsafe {
                while stack
                    .last()
                    .is_some_and(|node: &NonNull<ImplicitTreapNode<T>>| {
                        node.as_ref().data.priority < cur.as_ref().data.priority
                    })
                {
                    left = stack.pop();
                }
                cur.as_mut().child[0] = left;
                if let Some(parent) = stack.last_mut() {
                    parent.as_mut().child[1] = Some(cur);
                }
            }
            stack.push(cur);
            len += 1;
        }
        let root = stack.first().copied().map(BstRoot::new);
        if let Some(mut root) = root {
            Self::build_bottom_up(root.borrow_datamut());
            (Some(root), len)
        } else {
            (None, len)
        }
    }

    fn build_bottom_up(mut node: BstDataMutRef<'_, ImplicitTreapSpec<T>>) {
        if let Ok(left) = node.reborrow_datamut().left().descend() {
            Self::build_bottom_up(left);
        }
        if let Ok(right) = node.reborrow_datamut().right().descend() {
            Self::build_bottom_up(right);
        }
        ImplicitTreapSpec::<T>::bottom_up(node);
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn update<R>(&mut self, range: R, x: T::Act)
    where
        R: RangeBounds<usize>,
    {
        let mut split = Split3::seek_by_size(&mut self.root, range);
        if let Some(root) = split.mid_datamut() {
            ImplicitTreapSpec::<T>::update_act(root, &x);
        }
    }

    pub fn fold<R>(&mut self, range: R) -> T::Agg
    where
        R: RangeBounds<usize>,
    {
        let split = Split3::seek_by_size(&mut self.root, range);
        split
            .mid()
            .map(|node| node.into_data().value.agg.clone())
            .unwrap_or_else(T::agg_unit)
    }

    pub fn reverse<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        let mut split = Split3::seek_by_size(&mut self.root, range);
        if let Some(root) = split.mid_datamut() {
            ImplicitTreapSpec::<T>::reverse(root);
        }
    }

    pub fn get(&mut self, index: usize) -> Option<&T::Key> {
        if index >= self.length {
            return None;
        }
        let split = Split3::seek_by_size(&mut self.root, index..=index);
        let node = split.mid()?.node;
        drop(split);
        Some(unsafe { &(*node.as_ptr()).data.value.key })
    }

    pub fn modify<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&T::Key) -> T::Key,
    {
        assert!(index < self.length);
        let mut split = Split3::seek_by_size(&mut self.root, index..=index);
        let mut node = split.mid_datamut().unwrap();
        ImplicitTreapSpec::<T>::top_down(node.reborrow_datamut());
        {
            let data = node.data_mut();
            data.value.key = f(&data.value.key);
            data.value.agg = T::single_agg(&data.value.key);
        }
        ImplicitTreapSpec::<T>::bottom_up(node);
    }

    pub fn insert(&mut self, index: usize, x: T::Key) {
        assert!(index <= self.length);
        let node = self.node(x);
        if index == 0 {
            self.root = ImplicitTreapSpec::<T>::merge(Some(node), self.root.take());
        } else if index == self.length {
            self.root = ImplicitTreapSpec::<T>::merge(self.root.take(), Some(node));
        } else {
            let mut node = Some(node);
            let mut split = Split::new(&mut self.root, SeekBySize::new(index), false);
            split.manually_merge(|left, right| {
                ImplicitTreapSpec::<T>::merge(
                    ImplicitTreapSpec::<T>::merge(left, node.take()),
                    right,
                )
            });
        }
        self.length += 1;
    }

    pub fn remove(&mut self, index: usize) -> Option<T::Key> {
        if index >= self.length {
            return None;
        }
        let mid;
        if index == 0 {
            let (left, right) =
                ImplicitTreapSpec::<T>::split(self.root.take(), SeekBySize::new(1), false);
            mid = left;
            self.root = right;
        } else if index + 1 == self.length {
            let (left, right) =
                ImplicitTreapSpec::<T>::split(self.root.take(), SeekBySize::new(index), false);
            mid = right;
            self.root = left;
        } else {
            let (left, rest) =
                ImplicitTreapSpec::<T>::split(self.root.take(), SeekBySize::new(index), false);
            let (middle, right) = ImplicitTreapSpec::<T>::split(rest, SeekBySize::new(1), false);
            mid = middle;
            self.root = ImplicitTreapSpec::<T>::merge(left, right);
        }
        self.length -= 1;
        let mut node = mid.unwrap();
        ImplicitTreapSpec::<T>::top_down(node.borrow_datamut());
        let data = unsafe { node.into_dying().into_data(self.allocator.deref_mut()) };
        Some(data.value.key)
    }

    pub fn position_acc<R, F>(&mut self, range: R, f: F) -> Option<usize>
    where
        R: RangeBounds<usize>,
        F: FnMut(&T::Agg) -> bool,
    {
        let mut split3 = Split3::seek_by_size(&mut self.root, range);
        let front_size = split3
            .left()
            .map(|node| node.into_data().size)
            .unwrap_or_default();
        let split = split3.split_mid(SeekByAccCond::<ImplicitTreapSpec<T>, T, F>::new(f), false);
        split.right()?;
        let index = split
            .left()
            .map(|node| node.into_data().size)
            .unwrap_or_default();
        Some(front_size + index)
    }

    pub fn rposition_acc<R, F>(&mut self, range: R, f: F) -> Option<usize>
    where
        R: RangeBounds<usize>,
        F: FnMut(&T::Agg) -> bool,
    {
        let mut split3 = Split3::seek_by_size(&mut self.root, range);
        let front_size = split3
            .left()
            .map(|node| node.into_data().size)
            .unwrap_or_default();
        let split = split3.split_mid(SeekByRaccCond::<ImplicitTreapSpec<T>, T, F>::new(f), true);
        let left_size = split.left()?.into_data().size;
        Some(front_size + left_size - 1)
    }

    pub fn rotate_left(&mut self, mid: usize) {
        assert!(mid <= self.length);
        if mid == 0 || mid == self.length {
            return;
        }
        let (left, right) =
            ImplicitTreapSpec::<T>::split(self.root.take(), SeekBySize::new(mid), false);
        self.root = ImplicitTreapSpec::<T>::merge(right, left);
    }

    pub fn rotate_right(&mut self, k: usize) {
        assert!(k <= self.length);
        self.rotate_left(self.length - k);
    }
}

impl<T, A> Extend<T::Key> for ImplicitTreap<T, A>
where
    T: LazyMapMonoid,
    A: Allocator<ImplicitTreapNode<T>>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T::Key>,
    {
        let (root, len) = self.build(iter);
        self.root = ImplicitTreapSpec::<T>::merge(self.root.take(), root);
        self.length += len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{RangeChminChmaxAdd, RangeSumRangeChminChmaxAdd},
        num::Saturating,
        tools::{NotEmptySegment, Xorshift},
    };

    #[test]
    fn test_implicit_treap_range_sum_chmin_chmax_add_random() {
        const N: usize = 1_000;
        const Q: usize = 20_000;
        const A: i64 = 1_000;

        let mut rng = Xorshift::default();
        let mut arr: Vec<_> = (0..N).map(|_| Saturating(rng.random(0..=A))).collect();
        let mut treap = ImplicitTreap::<RangeSumRangeChminChmaxAdd<_>>::with_capacity(N + Q);
        treap.extend(arr.iter().copied());

        assert_eq!(
            Saturating(0),
            ImplicitTreap::<RangeSumRangeChminChmaxAdd<Saturating<i64>>>::new()
                .fold(..)
                .sum
        );
        assert_eq!(None, treap.remove(N));
        assert_eq!(None, treap.get(N));

        for _ in 0..Q {
            assert_eq!(arr.len(), treap.len());
            assert_eq!(arr.is_empty(), treap.is_empty());
            match rng.random(0..10) {
                0 if arr.len() < N * 2 => {
                    let i = rng.random(0..=arr.len());
                    let x = Saturating(rng.random(0..=A));
                    treap.insert(i, x);
                    arr.insert(i, x);
                }
                1 if !arr.is_empty() => {
                    let i = rng.random(0..arr.len());
                    assert_eq!(arr.remove(i), treap.remove(i).unwrap());
                }
                2 if !arr.is_empty() => {
                    let (l, r) = rng.random(NotEmptySegment(arr.len()));
                    assert_eq!(
                        arr[l..r].iter().copied().sum::<Saturating<i64>>(),
                        treap.fold(l..r).sum
                    );
                }
                3 if !arr.is_empty() => {
                    let (l, r) = rng.random(NotEmptySegment(arr.len()));
                    match rng.random(0..3) {
                        0 => {
                            let x = Saturating(rng.random(0..=A));
                            treap.update(l..r, RangeChminChmaxAdd::chmin(x));
                            arr[l..r].iter_mut().for_each(|a| *a = (*a).min(x));
                        }
                        1 => {
                            let x = Saturating(rng.random(0..=A));
                            treap.update(l..r, RangeChminChmaxAdd::chmax(x));
                            arr[l..r].iter_mut().for_each(|a| *a = (*a).max(x));
                        }
                        _ => {
                            let x = Saturating(rng.random(0..=A));
                            treap.update(l..r, RangeChminChmaxAdd::add(x));
                            arr[l..r].iter_mut().for_each(|a| *a += x);
                        }
                    }
                }
                4 if !arr.is_empty() => {
                    let (l, r) = rng.random(NotEmptySegment(arr.len()));
                    treap.reverse(l..r);
                    arr[l..r].reverse();
                }
                5 if !arr.is_empty() => {
                    let (l, r) = rng.random(NotEmptySegment(arr.len()));
                    let sum = arr[l..r].iter().copied().sum::<Saturating<i64>>();
                    let x = Saturating(rng.random(0..=sum.0.saturating_add(A)));
                    assert_eq!(
                        arr[l..r]
                            .iter()
                            .scan(Saturating(0), |acc, &a| {
                                *acc += a;
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map(|i| i + l),
                        treap.position_acc(l..r, |acc| acc.sum >= x),
                    );
                }
                6 if !arr.is_empty() => {
                    let (l, r) = rng.random(NotEmptySegment(arr.len()));
                    let sum = arr[l..r].iter().copied().sum::<Saturating<i64>>();
                    let x = Saturating(rng.random(0..=sum.0.saturating_add(A)));
                    assert_eq!(
                        arr[l..r]
                            .iter()
                            .rev()
                            .scan(Saturating(0), |acc, &a| {
                                *acc += a;
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map(|i| r - i - 1),
                        treap.rposition_acc(l..r, |acc| acc.sum >= x),
                    );
                }
                7 => {
                    let i = rng.random(0..=arr.len());
                    treap.rotate_left(i);
                    arr.rotate_left(i);
                }
                8 => {
                    let i = rng.random(0..=arr.len());
                    treap.rotate_right(i);
                    arr.rotate_right(i);
                }
                _ if !arr.is_empty() => {
                    let i = rng.random(0..arr.len());
                    if rng.random(0..2) == 0 {
                        assert_eq!(arr.get(i), treap.get(i));
                    } else {
                        let x = Saturating(rng.random(0..=A));
                        treap.modify(i, |_| x);
                        arr[i] = x;
                    }
                }
                _ => {}
            }
        }
    }
}
