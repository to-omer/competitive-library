use super::{
    Allocator, LazyMapAct, LazyMapAgg, LazyMapMonoid, LazyMapMonoidWithKeyAct, MemoryPool,
    binary_search_tree::{
        BstDataAccess, BstDataMutRef, BstNode, BstRoot, BstSeeker, BstSpec, EqualSide,
        data::{self, LazyMapElement},
        node::WithNoParent,
        seeker::{SeekByAccCond, SeekByRaccCond, SeekBySize},
        split::Split3,
    },
    splay_operations,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{ManuallyDrop, replace},
    ops::{DerefMut, RangeBounds},
};

type ImplicitSplayTreeRoot<T> = BstRoot<ImplicitSplayTreeSpec<T>>;
type ImplicitSplayTreeNode<T> = BstNode<ImplicitSplayTreeData<T>>;

pub struct ImplicitSplayTreeSpec<T> {
    _marker: PhantomData<fn() -> T>,
}

pub struct ImplicitSplayTreeData<T>
where
    T: LazyMapMonoid,
{
    value: LazyMapElement<T>,
    size: usize,
    rev: bool,
}

impl<T> Debug for ImplicitSplayTreeData<T>
where
    T: LazyMapMonoid<Key: Debug>,
    LazyMapAgg<T>: Debug,
    LazyMapAct<T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImplicitSplayTreeData")
            .field("value", &self.value)
            .field("size", &self.size)
            .field("rev", &self.rev)
            .finish()
    }
}

impl<T> BstDataAccess<data::marker::Size> for ImplicitSplayTreeData<T>
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

impl<T> BstDataAccess<data::marker::LazyMap> for ImplicitSplayTreeData<T>
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

impl<T> ImplicitSplayTreeSpec<T>
where
    T: LazyMapMonoidWithKeyAct,
{
    fn update_act(mut node: BstDataMutRef<'_, Self>, act: &LazyMapAct<T>) {
        if T::is_act_unit(act) {
            return;
        }
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
        node.swap_children();
        let data = node.data_mut();
        T::toggle(&mut data.value.agg);
        data.rev ^= true;
    }
}

impl<T> BstSpec for ImplicitSplayTreeSpec<T>
where
    T: LazyMapMonoidWithKeyAct,
{
    type Parent = WithNoParent<Self::Data>;
    type Data = ImplicitSplayTreeData<T>;

    fn top_down(mut node: BstDataMutRef<'_, Self>) {
        if !T::is_act_unit(&node.reborrow().into_data().value.act) {
            let act = replace(&mut node.data_mut().value.act, T::act_unit());
            if let Ok(left) = node.reborrow_datamut().left().descend() {
                Self::update_act(left, &act);
            }
            if let Ok(right) = node.reborrow_datamut().right().descend() {
                Self::update_act(right, &act);
            }
        }
        if node.reborrow().into_data().rev {
            node.data_mut().rev = false;
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
        left: Option<ImplicitSplayTreeRoot<T>>,
        right: Option<ImplicitSplayTreeRoot<T>>,
    ) -> Option<ImplicitSplayTreeRoot<T>> {
        splay_operations::merge(left, right)
    }

    fn split<Seeker>(
        node: Option<ImplicitSplayTreeRoot<T>>,
        seeker: Seeker,
        equal_side: EqualSide,
    ) -> (
        Option<ImplicitSplayTreeRoot<T>>,
        Option<ImplicitSplayTreeRoot<T>>,
    )
    where
        Seeker: BstSeeker<Spec = Self>,
    {
        splay_operations::split(node, seeker, equal_side)
    }
}

pub struct ImplicitSplayTree<T, A = MemoryPool<ImplicitSplayTreeNode<T>>>
where
    T: LazyMapMonoidWithKeyAct,
    A: Allocator<ImplicitSplayTreeNode<T>>,
{
    root: Option<ImplicitSplayTreeRoot<T>>,
    length: usize,
    allocator: ManuallyDrop<A>,
    _marker: PhantomData<fn() -> T>,
}

impl<T, A> Default for ImplicitSplayTree<T, A>
where
    T: LazyMapMonoidWithKeyAct,
    A: Allocator<ImplicitSplayTreeNode<T>> + Default,
{
    fn default() -> Self {
        Self {
            root: None,
            length: 0,
            allocator: ManuallyDrop::new(A::default()),
            _marker: PhantomData,
        }
    }
}

impl<T, A> Drop for ImplicitSplayTree<T, A>
where
    T: LazyMapMonoidWithKeyAct,
    A: Allocator<ImplicitSplayTreeNode<T>>,
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

impl<T> ImplicitSplayTree<T>
where
    T: LazyMapMonoidWithKeyAct,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            root: None,
            length: 0,
            allocator: ManuallyDrop::new(MemoryPool::with_capacity(capacity)),
            _marker: PhantomData,
        }
    }
}

impl<T, A> ImplicitSplayTree<T, A>
where
    T: LazyMapMonoidWithKeyAct,
    A: Allocator<ImplicitSplayTreeNode<T>>,
{
    fn node(&mut self, key: T::Key) -> ImplicitSplayTreeRoot<T> {
        BstRoot::from_data(
            ImplicitSplayTreeData {
                value: LazyMapElement::from_key(key),
                size: 1,
                rev: false,
            },
            self.allocator.deref_mut(),
        )
    }

    #[inline]
    fn splay<Seeker>(&mut self, seeker: Seeker) -> Option<Ordering>
    where
        Seeker: BstSeeker<Spec = ImplicitSplayTreeSpec<T>>,
    {
        let (ordering, root) = splay_operations::splay(self.root.take()?, seeker);
        self.root = Some(root);
        Some(ordering)
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn update<R>(&mut self, range: R, act: LazyMapAct<T>)
    where
        R: RangeBounds<usize>,
    {
        let mut split = Split3::seek_by_size(&mut self.root, range);
        if let Some(root) = split.mid_datamut() {
            ImplicitSplayTreeSpec::update_act(root, &act);
        }
    }

    pub fn fold<R>(&mut self, range: R) -> LazyMapAgg<T>
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
            ImplicitSplayTreeSpec::reverse(root);
        }
    }

    pub fn get(&mut self, index: usize) -> Option<&T::Key> {
        if index >= self.length {
            return None;
        }
        self.splay(SeekBySize::new(index));
        Some(&self.root.as_ref()?.reborrow().into_data().value.key)
    }

    pub fn modify<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&T::Key) -> T::Key,
    {
        assert!(index < self.length);
        self.splay(SeekBySize::new(index));
        let mut root = self.root.as_mut().unwrap().borrow_datamut();
        ImplicitSplayTreeSpec::top_down(root.reborrow_datamut());
        {
            let data = root.data_mut();
            data.value.key = f(&data.value.key);
        }
        ImplicitSplayTreeSpec::bottom_up(root);
    }

    pub fn insert(&mut self, index: usize, key: T::Key) {
        assert!(index <= self.length);
        let mut node = self.node(key);
        if self.root.is_none() {
            self.root = Some(node);
        } else if index == self.length {
            self.splay(SeekBySize::new(index));
            unsafe { node.borrow_mut().left_mut().set(self.root.take().unwrap()) };
            ImplicitSplayTreeSpec::bottom_up(node.borrow_datamut());
            self.root = Some(node);
        } else {
            self.splay(SeekBySize::new(index));
            let mut root = self.root.take().unwrap();
            let left = unsafe { root.borrow_mut().left_mut().take() };
            if let Some(left) = left {
                unsafe { node.borrow_mut().left_mut().set(left) };
            }
            ImplicitSplayTreeSpec::bottom_up(root.borrow_datamut());
            unsafe { node.borrow_mut().right_mut().set(root) };
            ImplicitSplayTreeSpec::bottom_up(node.borrow_datamut());
            self.root = Some(node);
        }
        self.length += 1;
    }

    pub fn remove(&mut self, index: usize) -> Option<T::Key> {
        if index >= self.length {
            return None;
        }
        self.splay(SeekBySize::new(index));
        let mut node = self.root.take().unwrap();
        ImplicitSplayTreeSpec::top_down(node.borrow_datamut());
        let left = unsafe { node.borrow_mut().left_mut().take() };
        let right = unsafe { node.borrow_mut().right_mut().take() };
        self.root = ImplicitSplayTreeSpec::merge(left, right);
        self.length -= 1;
        let data = unsafe { node.into_dying().into_data(self.allocator.deref_mut()) };
        Some(data.value.key)
    }

    pub fn partition_point_acc<F>(&mut self, left: usize, mut pred: F) -> usize
    where
        F: FnMut(&LazyMapAgg<T>) -> bool,
    {
        let mut split3 = Split3::seek_by_size(&mut self.root, left..);
        let front_size = split3
            .left()
            .map(|node| node.into_data().size)
            .unwrap_or_default();
        let split = split3.split_mid(SeekByAccCond::new(|acc| !pred(acc)), EqualSide::Right);
        let index = split
            .left()
            .map(|node| node.into_data().size)
            .unwrap_or_default();
        front_size + index
    }

    pub fn rpartition_point_acc<F>(&mut self, right: usize, mut pred: F) -> usize
    where
        F: FnMut(&LazyMapAgg<T>) -> bool,
    {
        let mut split3 = Split3::seek_by_size(&mut self.root, ..right);
        let split = split3.split_mid(SeekByRaccCond::new(|acc| !pred(acc)), EqualSide::Left);
        split
            .left()
            .map(|node| node.into_data().size)
            .unwrap_or_default()
    }

    pub fn rotate_left(&mut self, mid: usize) {
        assert!(mid <= self.length);
        if mid == 0 || mid == self.length {
            return;
        }
        let (left, right) =
            ImplicitSplayTreeSpec::split(self.root.take(), SeekBySize::new(mid), EqualSide::Right);
        self.root = ImplicitSplayTreeSpec::merge(right, left);
    }

    pub fn rotate_right(&mut self, k: usize) {
        assert!(k <= self.length);
        self.rotate_left(self.length - k);
    }
}

impl<T, A> Extend<T::Key> for ImplicitSplayTree<T, A>
where
    T: LazyMapMonoidWithKeyAct,
    A: Allocator<ImplicitSplayTreeNode<T>>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T::Key>,
    {
        let nodes = iter
            .into_iter()
            .map(|key| self.node(key))
            .collect::<Vec<_>>();
        let len = nodes.len();
        let root = if len == 0 {
            None
        } else {
            let mut stack = Vec::with_capacity(64);
            stack.push((0, len, None::<(usize, usize)>, false));
            while let Some((start, end, parent, visited)) = stack.pop() {
                if start == end {
                    continue;
                }
                let mid = start + (end - start) / 2;
                if visited {
                    ImplicitSplayTreeSpec::bottom_up(
                        BstRoot::new(nodes[mid].node).borrow_datamut(),
                    );
                    continue;
                }
                if let Some((parent, direction)) = parent {
                    let mut parent = nodes[parent].node;
                    unsafe { parent.as_mut().child[direction] = Some(nodes[mid].node) };
                }
                stack.push((start, end, parent, true));
                stack.push((mid + 1, end, Some((mid, 1)), false));
                stack.push((start, mid, Some((mid, 0)), false));
            }
            Some(BstRoot::new(nodes[len / 2].node))
        };
        self.root = ImplicitSplayTreeSpec::merge(self.root.take(), root);
        self.length += len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        WithEmptySegment, Xorshift,
        testutil::{ConcatUpdate, sequence_cases},
    };

    #[test]
    fn test_implicit_splay_tree() {
        const MAX_LEN: usize = 1024;
        const MAX_VALUE: i64 = 1_000_000_000;
        let mut input_rng = Xorshift::default();
        let mut rng = Xorshift::default();
        for (case, mut arr) in
            sequence_cases(&mut input_rng, -MAX_VALUE..=MAX_VALUE, 0..=MAX_LEN).enumerate()
        {
            let initial_len = arr.len();
            let mut tree = ImplicitSplayTree::<ConcatUpdate>::new();
            tree.extend(arr.iter().copied());
            assert_eq!(tree.fold(..), arr, "case {case}");
            assert_eq!(tree.remove(arr.len()), None);
            for step in 0..initial_len / 2 + 10 {
                let n = arr.len();
                let kind = if step < 10 { step } else { rng.random(0..10) };
                let index = rng.random(0..=n);
                let (l, r) = rng.random(WithEmptySegment(n));
                let value = rng.random(-MAX_VALUE..=MAX_VALUE);
                match kind {
                    0 if n < MAX_LEN => {
                        tree.insert(index, value);
                        arr.insert(index, value);
                    }
                    1 => {
                        let index = index.min(n.saturating_sub(1));
                        let expected = (index < n).then(|| arr.remove(index));
                        assert_eq!(tree.remove(index), expected);
                    }
                    3 if n != 0 => {
                        let index = index.min(n - 1);
                        tree.modify(index, |_| value);
                        arr[index] = value;
                    }
                    4 => assert_eq!(tree.fold(l..r), arr[l..r]),
                    5 => {
                        tree.update(l..r, Some(value));
                        arr[l..r].fill(value);
                    }
                    6 => {
                        tree.reverse(l..r);
                        arr[l..r].reverse();
                    }
                    7 => {
                        tree.rotate_left(index);
                        arr.rotate_left(index);
                    }
                    8 => {
                        tree.rotate_right(index);
                        arr.rotate_right(index);
                    }
                    9 => {
                        let target = &arr[l..r];
                        assert_eq!(
                            tree.partition_point_acc(l, |agg| target.starts_with(agg)),
                            r
                        );
                        assert_eq!(tree.rpartition_point_acc(r, |agg| target.ends_with(agg)), l);
                        assert_eq!(
                            tree.partition_point_acc(l, |agg| agg.iter().all(|x| *x < value)),
                            arr[l..]
                                .iter()
                                .position(|x| *x >= value)
                                .map_or(n, |i| l + i),
                        );
                        assert_eq!(
                            tree.rpartition_point_acc(r, |agg| agg.iter().all(|x| *x < value)),
                            arr[..r]
                                .iter()
                                .rposition(|x| *x >= value)
                                .map_or(0, |i| i + 1),
                        );
                    }
                    _ => assert_eq!(tree.get(index), arr.get(index)),
                }
                assert_eq!((tree.len(), tree.is_empty()), (arr.len(), arr.is_empty()));
                assert_eq!(tree.fold(..), arr, "case {case}, step {step}");
            }
            while !arr.is_empty() {
                let index = match arr.len() % 3 {
                    0 => 0,
                    1 => arr.len() - 1,
                    _ if arr.len() > 2 => rng.random(1..arr.len() - 1),
                    _ => rng.random(0..arr.len()),
                };
                assert_eq!(tree.remove(index), Some(arr.remove(index)));
                assert_eq!(tree.fold(..), arr);
            }
            assert_eq!((tree.len(), tree.is_empty()), (0, true));
            assert_eq!(tree.get(0), None);
            assert_eq!(tree.remove(0), None);
            tree.update(.., Some(rng.random(-MAX_VALUE..=MAX_VALUE)));
            tree.reverse(..);
            tree.rotate_left(0);
            tree.rotate_right(0);
            assert_eq!(tree.fold(..), arr);
            assert_eq!(tree.partition_point_acc(0, |_| true), 0);
            assert_eq!(tree.rpartition_point_acc(0, |_| true), 0);
            for _ in 0..initial_len.max(2) {
                let index = match arr.len() % 3 {
                    0 => 0,
                    1 => arr.len(),
                    _ => rng.random(1..arr.len()),
                };
                let value = rng.random(-MAX_VALUE..=MAX_VALUE);
                tree.insert(index, value);
                arr.insert(index, value);
                assert_eq!(tree.fold(..), arr);
            }
            let first = rng.random(-MAX_VALUE..MAX_VALUE);
            let second = rng.random(first + 1..=MAX_VALUE);
            let split = rng.random(1..arr.len());
            let index = rng.random(0..arr.len());
            tree.modify(index, |_| first);
            arr[index] = first;
            assert_eq!(tree.get(index), arr.get(index));
            tree.update(.., Some(first));
            arr.fill(first);
            tree.update(split.., Some(second));
            arr[split..].fill(second);
            tree.reverse(..);
            arr.reverse();
            tree.update(.., None);
            assert_eq!(tree.fold(..), arr);
            let middle = arr.len() - split;
            for (l, r) in [
                (0, 0),
                (0, arr.len()),
                (middle, middle + 1),
                (middle - 1, middle + 1),
            ] {
                let target = &arr[l..r];
                assert_eq!(tree.fold(l..r), target);
                assert_eq!(
                    tree.partition_point_acc(l, |agg| target.starts_with(agg)),
                    r
                );
                assert_eq!(tree.rpartition_point_acc(r, |agg| target.ends_with(agg)), l);
            }
        }
    }
}
