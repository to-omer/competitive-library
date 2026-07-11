use super::{
    Allocator, LazyMapMonoid, MemoryPool,
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
    T: LazyMapMonoid<Key: Debug, Agg: Debug, Act: Debug>,
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
    T: LazyMapMonoid,
{
    fn update_act(mut node: BstDataMutRef<'_, Self>, act: &T::Act) {
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
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
    A: Allocator<ImplicitSplayTreeNode<T>>,
{
    root: Option<ImplicitSplayTreeRoot<T>>,
    length: usize,
    allocator: ManuallyDrop<A>,
    _marker: PhantomData<fn() -> T>,
}

impl<T, A> Default for ImplicitSplayTree<T, A>
where
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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
    T: LazyMapMonoid,
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

    pub fn update<R>(&mut self, range: R, act: T::Act)
    where
        R: RangeBounds<usize>,
    {
        let mut split = Split3::seek_by_size(&mut self.root, range);
        if let Some(root) = split.mid_datamut() {
            ImplicitSplayTreeSpec::update_act(root, &act);
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
        let split = split3.split_mid(SeekByAccCond::new(f), EqualSide::Right);
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
        let split = split3.split_mid(SeekByRaccCond::new(f), EqualSide::Left);
        let left_size = split.left()?.into_data().size;
        Some(front_size + left_size - 1)
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
    T: LazyMapMonoid,
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
    use crate::{
        algebra::RangeMaxRangeUpdate,
        rand,
        tools::{NotEmptySegment, Xorshift},
    };

    #[test]
    fn test_implicit_splay_tree() {
        const N: usize = 1_000;
        const Q: usize = 20_000;
        const A: i64 = 1_000_000_000;

        let mut rng = Xorshift::default();
        rand!(rng, mut arr: [-A..A; N]);
        let mut tree = ImplicitSplayTree::<RangeMaxRangeUpdate<_>>::new();
        tree.extend(arr.iter().copied());
        for _ in 0..Q {
            assert_eq!(arr.len(), tree.len());
            rand!(rng, ty: 0..10, (l, r): NotEmptySegment(arr.len()));
            match ty {
                0 if arr.len() < N * 2 => {
                    rand!(rng, i: ..=arr.len(), x: -A..A);
                    tree.insert(i, x);
                    arr.insert(i, x);
                }
                1 if arr.len() > 1 => {
                    rand!(rng, i: ..arr.len());
                    assert_eq!(arr.remove(i), tree.remove(i).unwrap());
                }
                2 => assert_eq!(tree.fold(l..r), *arr[l..r].iter().max().unwrap()),
                3 => {
                    rand!(rng, x: -A..A);
                    tree.update(l..r, Some(x));
                    arr[l..r].fill(x);
                }
                4 => {
                    tree.reverse(l..r);
                    arr[l..r].reverse();
                }
                5 => {
                    rand!(rng, x: -A..A);
                    assert_eq!(
                        tree.position_acc(l..r, |&value| value >= x),
                        arr[l..r]
                            .iter()
                            .scan(i64::MIN, |acc, &value| {
                                *acc = (*acc).max(value);
                                Some(*acc)
                            })
                            .position(|value| value >= x)
                            .map(|index| l + index),
                    );
                }
                6 => {
                    rand!(rng, x: -A..A);
                    assert_eq!(
                        tree.rposition_acc(l..r, |&value| value >= x),
                        arr[l..r]
                            .iter()
                            .rev()
                            .scan(i64::MIN, |acc, &value| {
                                *acc = (*acc).max(value);
                                Some(*acc)
                            })
                            .position(|value| value >= x)
                            .map(|index| r - index - 1),
                    );
                }
                7 => {
                    rand!(rng, mid: ..=arr.len());
                    tree.rotate_left(mid);
                    arr.rotate_left(mid);
                }
                8 => {
                    rand!(rng, count: ..=arr.len());
                    tree.rotate_right(count);
                    arr.rotate_right(count);
                }
                _ => {
                    rand!(rng, index: ..arr.len(), value: -A..A);
                    tree.modify(index, |_| value);
                    arr[index] = value;
                }
            }
            assert_eq!(tree.get(tree.len()), None);
            rand!(rng, index: ..arr.len());
            assert_eq!(tree.get(index), arr.get(index));
        }
    }
}
