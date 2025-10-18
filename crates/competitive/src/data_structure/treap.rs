use super::{
    Allocator, BoxAllocator, LazyMapMonoid, MonoidAct, Xorshift,
    binary_search_tree::{
        BstDataAccess, BstDataMutRef, BstNode, BstNodeId, BstNodeIdManager, BstRoot, BstSeeker,
        BstSpec,
        data::{self, LazyMapElement, MonoidActElement},
        node::WithParent,
        seeker::{SeekByAccCond, SeekByKey, SeekByRaccCond},
        split::{Split, Split3},
    },
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{DerefMut, RangeBounds},
};

type TreapRoot<M, L> = BstRoot<TreapSpec<M, L>>;
type TreapNode<M, L> = BstNode<TreapData<M, L>, WithParent<TreapData<M, L>>>;

pub struct TreapSpec<M, L> {
    _marker: PhantomData<(M, L)>,
}

pub struct TreapData<M, L>
where
    M: MonoidAct<Key: Ord>,
    L: LazyMapMonoid,
{
    priority: u64,
    key: MonoidActElement<M>,
    value: LazyMapElement<L>,
}

impl<M, L> Debug for TreapData<M, L>
where
    M: MonoidAct<Key: Ord + Debug, Act: Debug>,
    L: LazyMapMonoid<Key: Debug, Agg: Debug, Act: Debug>,
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
    M: MonoidAct<Key: Ord>,
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
    M: MonoidAct<Key: Ord>,
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
    M: MonoidAct<Key: Ord>,
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
    M: MonoidAct<Key: Ord>,
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
    M: MonoidAct<Key: Ord>,
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
    M: MonoidAct<Key: Ord>,
    L: LazyMapMonoid,
    A: Allocator<TreapNode<M, L>>,
{
    root: Option<TreapRoot<M, L>>,
    node_id_manager: BstNodeIdManager<TreapSpec<M, L>>,
    rng: Xorshift,
    allocator: ManuallyDrop<A>,
    _marker: PhantomData<(M, L)>,
}

impl<M, L, A> Default for Treap<M, L, A>
where
    M: MonoidAct<Key: Ord>,
    L: LazyMapMonoid,
    A: Allocator<TreapNode<M, L>> + Default,
{
    fn default() -> Self {
        Self {
            root: None,
            node_id_manager: Default::default(),
            rng: Xorshift::new(),
            allocator: ManuallyDrop::new(A::default()),
            _marker: PhantomData,
        }
    }
}

impl<M, L, A> Drop for Treap<M, L, A>
where
    M: MonoidAct<Key: Ord>,
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
    M: MonoidAct<Key: Ord>,
    L: LazyMapMonoid,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<M, L, A> Treap<M, L, A>
where
    M: MonoidAct<Key: Ord>,
    L: LazyMapMonoid,
    A: Allocator<TreapNode<M, L>>,
{
    pub fn len(&self) -> usize {
        self.node_id_manager.len()
    }

    pub fn is_empty(&self) -> bool {
        self.node_id_manager.is_empty()
    }

    pub fn clear(&mut self) {
        unsafe {
            if let Some(root) = self.root.take() {
                root.into_dying().drop_all(self.allocator.deref_mut());
            }
            self.node_id_manager.clear();
        }
    }

    pub fn get(&mut self, node_id: BstNodeId<TreapSpec<M, L>>) -> Option<(&M::Key, &L::Key)> {
        if !self.node_id_manager.contains(&node_id) {
            return None;
        }
        unsafe {
            WithParent::resolve_top_down::<TreapSpec<M, L>>(
                node_id.reborrow_datamut(&mut self.root),
            );
            let data = node_id.reborrow(&self.root).into_data();
            Some((&data.key.key, &data.value.key))
        }
    }

    pub fn change(
        &mut self,
        node_id: BstNodeId<TreapSpec<M, L>>,
        f: impl FnOnce(&mut L::Key),
    ) -> bool {
        if !self.node_id_manager.contains(&node_id) {
            return false;
        }
        unsafe {
            WithParent::resolve_top_down::<TreapSpec<M, L>>(
                node_id.reborrow_datamut(&mut self.root),
            );
            let data = node_id.reborrow_datamut(&mut self.root).into_data_mut();
            f(&mut data.value.key);
            WithParent::resolve_bottom_up::<TreapSpec<M, L>>(
                node_id.reborrow_datamut(&mut self.root),
            );
        }
        true
    }

    pub fn change_key_value(
        &mut self,
        node_id: BstNodeId<TreapSpec<M, L>>,
        f: impl FnOnce(&mut M::Key, &mut L::Key),
    ) -> bool {
        if !self.node_id_manager.contains(&node_id) {
            return false;
        }
        unsafe {
            WithParent::resolve_top_down::<TreapSpec<M, L>>(
                node_id.reborrow_datamut(&mut self.root),
            );
            let mut node = if WithParent::is_root(node_id.reborrow(&self.root)) {
                WithParent::remove_root(&mut self.root).unwrap_unchecked()
            } else {
                WithParent::remove_not_root(node_id.reborrow_mut(&mut self.root))
            };
            let data = node.borrow_datamut().into_data_mut();
            f(&mut data.key.key, &mut data.value.key);
            self.root = TreapSpec::merge_ordered(self.root.take(), Some(node));
            true
        }
    }

    pub fn insert(&mut self, key: M::Key, value: L::Key) -> BstNodeId<TreapSpec<M, L>> {
        let (left, right) = TreapSpec::split(self.root.take(), SeekByKey::new(&key), false);
        let data = TreapData {
            priority: self.rng.rand64(),
            key: MonoidActElement::from_key(key),
            value: LazyMapElement::from_key(value),
        };
        let node = BstRoot::from_data(data, self.allocator.deref_mut());
        let node_id = self.node_id_manager.register(&node);
        self.root = TreapSpec::merge(TreapSpec::merge(left, Some(node)), right);
        node_id
    }

    pub fn remove(&mut self, node_id: BstNodeId<TreapSpec<M, L>>) -> Option<(M::Key, L::Key)> {
        if !self.node_id_manager.contains(&node_id) {
            return None;
        }
        unsafe {
            WithParent::resolve_top_down::<TreapSpec<M, L>>(
                node_id.reborrow_datamut(&mut self.root),
            );
            let node = if WithParent::is_root(node_id.reborrow(&self.root)) {
                WithParent::remove_root(&mut self.root).unwrap_unchecked()
            } else {
                WithParent::remove_not_root(node_id.reborrow_mut(&mut self.root))
            };
            self.node_id_manager.unregister(node_id);
            let data = node.into_dying().into_data(self.allocator.deref_mut());
            Some((data.key.key, data.value.key))
        }
    }

    pub fn range_by_key<Q, R>(&mut self, range: R) -> TreapSplit3<'_, M, L>
    where
        M: MonoidAct<Key: Borrow<Q>>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let split3 = Split3::seek_by_key(&mut self.root, range);
        TreapSplit3 {
            split3,
            key_updated: false,
        }
    }

    pub fn find_by_key<Q>(&mut self, key: &Q) -> Option<BstNodeId<TreapSpec<M, L>>>
    where
        M: MonoidAct<Key: Borrow<Q>>,
        Q: Ord + ?Sized,
    {
        let split = Split::new(
            &mut self.root,
            SeekByKey::<TreapSpec<M, L>, M::Key, Q>::new(key),
            false,
        );
        let node = split.right()?.leftmost()?;
        matches!(node.into_data().key.key.borrow().cmp(key), Ordering::Equal)
            .then(|| self.node_id_manager.registerd_node_id(node))
            .flatten()
    }

    pub fn find_by_acc_cond<F>(&mut self, f: F) -> Option<BstNodeId<TreapSpec<M, L>>>
    where
        F: FnMut(&L::Agg) -> bool,
    {
        let split = Split::new(
            &mut self.root,
            SeekByAccCond::<TreapSpec<M, L>, L, F>::new(f),
            false,
        );
        let node = split.right()?.leftmost()?;
        self.node_id_manager.registerd_node_id(node)
    }

    pub fn find_by_racc_cond<F>(&mut self, f: F) -> Option<BstNodeId<TreapSpec<M, L>>>
    where
        F: FnMut(&L::Agg) -> bool,
    {
        let split = Split::new(
            &mut self.root,
            SeekByRaccCond::<TreapSpec<M, L>, L, F>::new(f),
            true,
        );
        let node = split.left()?.rightmost()?;
        self.node_id_manager.registerd_node_id(node)
    }
}

pub struct TreapSplit3<'a, M, L>
where
    M: MonoidAct<Key: Ord>,
    L: LazyMapMonoid,
{
    split3: Split3<'a, TreapSpec<M, L>>,
    key_updated: bool,
}

impl<'a, M, L> TreapSplit3<'a, M, L>
where
    M: MonoidAct<Key: Ord>,
    L: LazyMapMonoid,
{
    pub fn fold(&self) -> L::Agg {
        if let Some(node) = self.split3.mid() {
            node.reborrow().into_data().value.agg.clone()
        } else {
            L::agg_unit()
        }
    }

    pub fn update_key(&mut self, act: M::Act) {
        if let Some(node) = self.split3.mid_datamut() {
            MonoidActElement::<M>::update_act(node, &act);
            self.key_updated = true;
        }
    }

    pub fn update_value(&mut self, act: L::Act) {
        if let Some(node) = self.split3.mid_datamut() {
            LazyMapElement::<L>::update_act(node, &act);
        }
    }
}

impl<'a, M, L> Drop for TreapSplit3<'a, M, L>
where
    M: MonoidAct<Key: Ord>,
    L: LazyMapMonoid,
{
    fn drop(&mut self) {
        if self.key_updated {
            self.split3.manually_merge(TreapSpec::merge_ordered);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::{
        AdditiveOperation, EmptyAct, FlattenAct, RangeMaxRangeAdd, RangeSumRangeAdd,
    };

    #[test]
    fn test_treap() {
        const A: i64 = 100;
        let mut rng = Xorshift::default();
        let mut treap = Treap::<FlattenAct<AdditiveOperation<i64>>, RangeMaxRangeAdd<i64>>::new();
        let mut node_ids = vec![];
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
            match rng.random(0..8) {
                0 => {
                    let key = rng.random(-A..=A);
                    let value = rng.random(-A..=A);
                    let k = data.partition_point(|(k, _)| *k < key);
                    data.insert(k, (key, value));
                    node_ids.insert(k, treap.insert(key, value));
                }
                1 => {
                    if !data.is_empty() {
                        let k = rng.random(0..data.len());
                        let expected = data.remove(k);
                        let result = treap.remove(node_ids.remove(k)).unwrap();
                        assert_eq!(expected, result);
                    }
                }
                2 => {
                    let expected: i64 = data
                        .iter()
                        .filter(|(k, _)| (l..r).contains(k))
                        .map(|(_, v)| *v)
                        .max()
                        .unwrap_or(i64::MIN);
                    let result = treap.range_by_key(l..r).fold();
                    assert_eq!(expected, result);
                }
                3 => {
                    let add = rng.random(-A..=A);
                    for (k, v) in data.iter_mut() {
                        if (l..r).contains(k) {
                            *v += add;
                        }
                    }
                    treap.range_by_key(l..r).update_value(add);
                }
                4 => {
                    let add = rng.random(-A..=A);
                    for (k, _) in data.iter_mut() {
                        if (l..r).contains(k) {
                            *k += add;
                        }
                    }
                    treap.range_by_key(l..r).update_key(add);
                }
                5 => {
                    if !data.is_empty() {
                        let k = rng.random(0..data.len());
                        let expected = data[k];
                        let result = treap.get(node_ids[k]).unwrap();
                        assert_eq!(expected, (*result.0, *result.1));
                    }
                }
                6 => {
                    if !data.is_empty() {
                        let k = rng.random(0..data.len());
                        let x = rng.random(-A..=A);
                        data[k].1 = x;
                        treap.change(node_ids[k], |value| *value = x);
                    }
                }
                _ => {
                    if !data.is_empty() {
                        let k = rng.random(0..data.len());
                        let nk = rng.random(-A..=A);
                        let nv = rng.random(-A..=A);
                        data[k].0 = nk;
                        data[k].1 = nv;
                        treap.change_key_value(node_ids[k], |key, value| {
                            *key = nk;
                            *value = nv;
                        });
                    }
                }
            }
        }

        let mut treap = Treap::<EmptyAct<i64>, RangeSumRangeAdd<i64>>::new();
        let mut node_ids = vec![];
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
            match rng.random(0..10) {
                0 => {
                    let key = rng.random(-A..=A);
                    let value = rng.random(1..=A);
                    let k = data.partition_point(|(k, _)| *k < key);
                    data.insert(k, (key, value));
                    node_ids.insert(k, treap.insert(key, value));
                }
                1 => {
                    if !data.is_empty() {
                        let k = rng.random(0..data.len());
                        let expected = data.remove(k);
                        let result = treap.remove(node_ids.remove(k)).unwrap();
                        assert_eq!(expected, result);
                    }
                }
                2 => {
                    let expected: i64 = data
                        .iter()
                        .filter(|(k, _)| (l..r).contains(k))
                        .map(|(_, v)| *v)
                        .sum();
                    let result = treap.range_by_key(l..r).fold().0;
                    assert_eq!(expected, result);
                }
                3 => {
                    let add = rng.random(1..=A);
                    for (k, v) in data.iter_mut() {
                        if (l..r).contains(k) {
                            *v += add;
                        }
                    }
                    treap.range_by_key(l..r).update_value(add);
                }
                5 => {
                    if !data.is_empty() {
                        let k = rng.random(0..data.len());
                        let expected = data[k];
                        let result = treap.get(node_ids[k]).unwrap();
                        assert_eq!(expected, (*result.0, *result.1));
                    }
                }
                6 => {
                    if !data.is_empty() {
                        let k = rng.random(0..data.len());
                        let x = rng.random(1..=A);
                        data[k].1 = x;
                        treap.change(node_ids[k], |value| *value = x);
                    }
                }
                7 => {
                    let key = rng.random(-A..=A);
                    let expected = data.iter().find(|(k, _)| *k == key).cloned();
                    let result = treap.find_by_key(&key).map(|id| treap.get(id).unwrap());
                    assert_eq!(expected, result.map(|(k, v)| (*k, *v)));
                }
                8 => {
                    let s = rng.random(0..=A);
                    let mut acc = 0;
                    let expected = data.iter().find_map(|(k, v)| {
                        acc += *v;
                        if acc >= s { Some((*k, *v)) } else { None }
                    });
                    let result = treap
                        .find_by_acc_cond(|agg| agg.0 >= s)
                        .map(|id| treap.get(id).unwrap());
                    assert_eq!(expected, result.map(|(k, v)| (*k, *v)));
                }
                _ => {
                    let s = rng.random(0..=A);
                    let mut acc = 0;
                    let expected = data.iter().rev().find_map(|(k, v)| {
                        acc += *v;
                        if acc >= s { Some((*k, *v)) } else { None }
                    });
                    let result = treap
                        .find_by_racc_cond(|agg| agg.0 >= s)
                        .map(|id| treap.get(id).unwrap());
                    assert_eq!(expected, result.map(|(k, v)| (*k, *v)));
                }
            }
        }
    }
}
