use super::{
    Allocator, BoxAllocator, LazyMapMonoid, MonoidAct, Xorshift,
    binary_search_tree::{
        BstDataAccess, BstDataMutRef, BstImmutRef, BstNode, BstRoot, BstSeeker, BstSpec,
        data::{self, LazyMapElement, MonoidActElement},
        node::WithParent,
        seeker::SeekByKey,
        split::Split3,
    },
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{DerefMut, RangeBounds},
    ptr::NonNull,
};

type TreapRoot<M, L> = BstRoot<TreapSpec<M, L>>;
type TreapNode<M, L> = BstNode<TreapData<M, L>, WithParent<TreapData<M, L>>>;

struct TreapSpec<M, L> {
    _marker: PhantomData<(M, L)>,
}

pub struct TreapData<M, L>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
{
    priority: u64,
    key: MonoidActElement<M>,
    value: LazyMapElement<L>,
}

impl<M, L> Debug for TreapData<M, L>
where
    M: MonoidAct,
    M::Key: Ord + Debug,
    M::Act: Debug,
    L: LazyMapMonoid,
    L::Key: Debug,
    L::Agg: Debug,
    L::Act: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TreapData")
            .field("priority", &self.priority)
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<M, L> BstDataAccess<data::marker::Key> for TreapData<M, L>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
{
    type Value = M::Key;

    fn bst_data(&self) -> &Self::Value {
        &self.key.key
    }

    fn bst_data_mut(&mut self) -> &mut Self::Value {
        &mut self.key.key
    }
}

impl<M, L> BstDataAccess<data::marker::MonoidAct> for TreapData<M, L>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
{
    type Value = MonoidActElement<M>;

    fn bst_data(&self) -> &Self::Value {
        &self.key
    }

    fn bst_data_mut(&mut self) -> &mut Self::Value {
        &mut self.key
    }
}

impl<M, L> BstDataAccess<data::marker::LazyMap> for TreapData<M, L>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
{
    type Value = LazyMapElement<L>;

    fn bst_data(&self) -> &Self::Value {
        &self.value
    }

    fn bst_data_mut(&mut self) -> &mut Self::Value {
        &mut self.value
    }
}

impl<M, L> BstSpec for TreapSpec<M, L>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
{
    type Parent = WithParent<Self::Data>;
    type Data = TreapData<M, L>;

    fn top_down(mut node: BstDataMutRef<'_, Self>) {
        MonoidActElement::<M>::top_down(node.reborrow_datamut());
        LazyMapElement::<L>::top_down(node.reborrow_datamut());
    }

    fn bottom_up(mut node: BstDataMutRef<'_, Self>) {
        LazyMapElement::<L>::bottom_up(node.reborrow_datamut());
    }

    fn merge(
        left: Option<TreapRoot<M, L>>,
        right: Option<TreapRoot<M, L>>,
    ) -> Option<TreapRoot<M, L>> {
        match (left, right) {
            (None, None) => None,
            (None, Some(node)) | (Some(node), None) => Some(node),
            (Some(mut left), Some(mut right)) => unsafe {
                if left.reborrow().into_data().priority > right.reborrow().into_data().priority {
                    TreapSpec::top_down(left.borrow_datamut());
                    let lr = left.borrow_mut().right().take();
                    let lr = Self::merge(lr, Some(right)).unwrap_unchecked();
                    left.borrow_mut().right().set(lr);
                    TreapSpec::bottom_up(left.borrow_datamut());
                    Some(left)
                } else {
                    TreapSpec::top_down(right.borrow_datamut());
                    let rl = right.borrow_mut().left().take();
                    let rl = Self::merge(Some(left), rl).unwrap_unchecked();
                    right.borrow_mut().left().set(rl);
                    TreapSpec::bottom_up(right.borrow_datamut());
                    Some(right)
                }
            },
        }
    }

    fn split<Seeker>(
        node: Option<TreapRoot<M, L>>,
        mut seeker: Seeker,
        eq_left: bool,
    ) -> (Option<TreapRoot<M, L>>, Option<TreapRoot<M, L>>)
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

impl<M, L> TreapSpec<M, L>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
{
    pub fn merge_ordered(
        left: Option<TreapRoot<M, L>>,
        right: Option<TreapRoot<M, L>>,
    ) -> Option<TreapRoot<M, L>> {
        match (left, right) {
            (None, None) => None,
            (None, Some(node)) | (Some(node), None) => Some(node),
            (Some(mut left), Some(mut right)) => unsafe {
                if left.reborrow().into_data().priority > right.reborrow().into_data().priority {
                    Self::top_down(left.borrow_datamut());
                    let key = &left.reborrow().into_data().key.key;
                    let (rl, rr) = Self::split(Some(right), SeekByKey::new(key), false);
                    let ll = left.borrow_mut().left().take();
                    let lr = left.borrow_mut().right().take();
                    if let Some(l) = Self::merge_ordered(ll, rl) {
                        left.borrow_mut().left().set(l);
                    }
                    if let Some(r) = Self::merge_ordered(lr, rr) {
                        left.borrow_mut().right().set(r);
                    }
                    Self::bottom_up(left.borrow_datamut());
                    Some(left)
                } else {
                    Self::top_down(right.borrow_datamut());
                    let key = &right.reborrow().into_data().key.key;
                    let (ll, lr) = Self::split(Some(left), SeekByKey::new(key), false);
                    let rl = right.borrow_mut().left().take();
                    let rr = right.borrow_mut().right().take();
                    if let Some(l) = Self::merge_ordered(ll, rl) {
                        right.borrow_mut().left().set(l);
                    }
                    if let Some(r) = Self::merge_ordered(lr, rr) {
                        right.borrow_mut().right().set(r);
                    }
                    Self::bottom_up(right.borrow_datamut());
                    Some(right)
                }
            },
        }
    }
}

pub struct Treap<M, L, A = BoxAllocator<TreapNode<M, L>>>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
    A: Allocator<TreapNode<M, L>>,
{
    root: Option<TreapRoot<M, L>>,
    size: usize,
    nodes: Vec<Option<NonNull<TreapNode<M, L>>>>,
    rng: Xorshift,
    allocator: ManuallyDrop<A>,
    _marker: PhantomData<(M, L)>,
}

impl<M, L, A> Default for Treap<M, L, A>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
    A: Allocator<TreapNode<M, L>> + Default,
{
    fn default() -> Self {
        Self {
            root: None,
            size: 0,
            nodes: vec![],
            rng: Xorshift::new(),
            allocator: ManuallyDrop::new(A::default()),
            _marker: PhantomData,
        }
    }
}

impl<M, L, A> Drop for Treap<M, L, A>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
    A: Allocator<TreapNode<M, L>>,
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

impl<M, L> Treap<M, L>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<M, L, A> Treap<M, L, A>
where
    M: MonoidAct,
    M::Key: Ord,
    L: LazyMapMonoid,
    A: Allocator<TreapNode<M, L>>,
{
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        unsafe {
            if let Some(root) = self.root.take() {
                root.into_dying().drop_all(self.allocator.deref_mut());
            }
            self.size = 0;
            self.nodes.clear();
        }
    }

    pub fn kth_inserted(&mut self, k: usize) -> Option<(&M::Key, &L::Key)> {
        unsafe fn immut<M, L, A>(
            _treap: &mut Treap<M, L, A>,
            node: NonNull<TreapNode<M, L>>,
        ) -> BstImmutRef<'_, TreapSpec<M, L>>
        where
            M: MonoidAct,
            M::Key: Ord,
            L: LazyMapMonoid,
            A: Allocator<TreapNode<M, L>>,
        {
            unsafe { BstImmutRef::new_unchecked(node) }
        }
        let node = *self.nodes.get(k)?.as_ref()?;
        unsafe {
            WithParent::resolve_top_down::<TreapSpec<M, L>>(node);
            let node = immut(self, node);
            let data = node.into_data();
            Some((&data.key.key, &data.value.key))
        }
    }

    pub fn insert(&mut self, key: M::Key, value: L::Key) {
        let (left, right) = TreapSpec::split(self.root.take(), SeekByKey::new(&key), false);
        let data = TreapData {
            priority: self.rng.rand64(),
            key: MonoidActElement::from_key(key),
            value: LazyMapElement::from_key(value),
        };
        let node = BstRoot::from_data(data, self.allocator.deref_mut());
        self.nodes.push(Some(node.node));
        self.size += 1;
        self.root = TreapSpec::merge(TreapSpec::merge(left, Some(node)), right);
    }

    pub fn fold<Q, R>(&mut self, range: R) -> L::Agg
    where
        M::Key: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        if let Some(node) = Split3::seek_by_key(&mut self.root, range).mid() {
            node.reborrow().into_data().value.agg.clone()
        } else {
            L::agg_unit()
        }
    }

    pub fn update_key<Q, R>(&mut self, range: R, act: M::Act)
    where
        M::Key: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let mut split = Split3::seek_by_key(&mut self.root, range);
        if let Some(node) = split.mid_datamut() {
            MonoidActElement::<M>::update_act(node, &act);
        }
        split.manually_merge(TreapSpec::merge_ordered);
    }

    pub fn update_value<Q, R>(&mut self, range: R, act: L::Act)
    where
        M::Key: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        if let Some(node) = Split3::seek_by_key(&mut self.root, range).mid_datamut() {
            LazyMapElement::<L>::update_act(node, &act);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::{AdditiveOperation, FlattenAct, RangeSumRangeAdd};

    #[test]
    fn test_treap() {
        const A: i64 = 100;
        let mut rng = Xorshift::default();
        let mut treap = Treap::<FlattenAct<AdditiveOperation<i64>>, RangeSumRangeAdd<i64>>::new();
        let mut data = vec![];
        for _ in 0..10000 {
            let (l, r) = loop {
                let l = rng.random(-A..=A);
                let r = rng.random(-A..=A);
                if l <= r {
                    break (l, r);
                }
            };
            assert_eq!(data.len(), treap.len());
            assert_eq!(data.is_empty(), treap.is_empty());
            match rng.random(0..5) {
                0 => {
                    let key = rng.random(-A..=A);
                    let value = rng.random(-A..=A);
                    treap.insert(key, value);
                    data.push((key, value));
                }
                1 => {
                    let expected: i64 = data
                        .iter()
                        .filter(|(k, _)| (l..r).contains(k))
                        .map(|(_, v)| *v)
                        .sum();
                    let result = treap.fold(l..r).0;
                    assert_eq!(expected, result);
                }
                2 => {
                    let add = rng.random(-A..=A);
                    for (k, v) in data.iter_mut() {
                        if (l..r).contains(k) {
                            *v += add;
                        }
                    }
                    treap.update_value(l..r, add);
                }
                3 => {
                    let add = rng.random(-A..=A);
                    for (k, _) in data.iter_mut() {
                        if (l..r).contains(k) {
                            *k += add;
                        }
                    }
                    treap.update_key(l..r, add);
                }
                _ => {
                    if !data.is_empty() {
                        let k = rng.random(0..data.len());
                        let expected = data[k];
                        let result = treap.kth_inserted(k).unwrap();
                        assert_eq!(expected, (*result.0, *result.1));
                    }
                }
            }
        }
    }
}
