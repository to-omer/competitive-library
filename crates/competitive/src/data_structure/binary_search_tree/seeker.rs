use super::{BstDataAccess, BstImmutRef, BstSpec, LazyMapMonoid, data, data::LazyMapElement};
use std::{borrow::Borrow, cmp::Ordering, marker::PhantomData};

pub trait BstSeeker {
    type Spec: BstSpec;

    fn bst_seek(&mut self, _node: BstImmutRef<'_, Self::Spec>) -> Ordering;
}

pub struct SeekLeft<Spec> {
    _marker: PhantomData<fn() -> Spec>,
}

impl<S> Default for SeekLeft<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Spec> BstSeeker for SeekLeft<Spec>
where
    Spec: BstSpec,
{
    type Spec = Spec;

    fn bst_seek(&mut self, node: BstImmutRef<'_, Self::Spec>) -> Ordering {
        if node.reborrow().right().descend().is_ok() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

pub struct SeekRight<Spec> {
    _marker: PhantomData<fn() -> Spec>,
}

impl<S> Default for SeekRight<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Spec> BstSeeker for SeekRight<Spec>
where
    Spec: BstSpec,
{
    type Spec = Spec;
    fn bst_seek(&mut self, node: BstImmutRef<'_, Self::Spec>) -> Ordering {
        if node.reborrow().left().descend().is_ok() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

pub struct SeekByKey<'a, Spec, K, Q>
where
    Q: ?Sized,
{
    key: &'a Q,
    _marker: PhantomData<fn() -> (Spec, K)>,
}

impl<'a, Spec, K, Q> SeekByKey<'a, Spec, K, Q>
where
    Q: ?Sized,
{
    pub fn new(key: &'a Q) -> Self {
        Self {
            key,
            _marker: PhantomData,
        }
    }
}

impl<Spec, K, Q> BstSeeker for SeekByKey<'_, Spec, K, Q>
where
    Spec: BstSpec<Data: BstDataAccess<data::marker::Key, Value = K>>,
    K: Borrow<Q>,
    Q: Ord + ?Sized,
{
    type Spec = Spec;

    fn bst_seek(&mut self, node: BstImmutRef<'_, Self::Spec>) -> Ordering {
        node.reborrow()
            .into_data()
            .bst_data()
            .borrow()
            .cmp(self.key)
    }
}

pub struct SeekBySize<Spec> {
    index: usize,
    _marker: PhantomData<fn() -> Spec>,
}

impl<Spec> SeekBySize<Spec> {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }
}

impl<Spec> BstSeeker for SeekBySize<Spec>
where
    Spec: BstSpec<Data: BstDataAccess<data::marker::Size, Value = usize>>,
{
    type Spec = Spec;

    fn bst_seek(&mut self, node: BstImmutRef<'_, Self::Spec>) -> Ordering {
        let lsize = node
            .reborrow()
            .left()
            .descend()
            .map(|l| *l.into_data().bst_data())
            .unwrap_or_default();
        let ord = lsize.cmp(&self.index);
        if matches!(ord, Ordering::Less) {
            self.index -= lsize + 1;
        }
        ord
    }
}

pub struct SeekByAccCond<Spec, L, F>
where
    L: LazyMapMonoid,
{
    acc: L::Agg,
    f: F,
    _marker: PhantomData<fn() -> (Spec, L)>,
}

impl<Spec, L, F> SeekByAccCond<Spec, L, F>
where
    L: LazyMapMonoid,
    F: FnMut(&L::Agg) -> bool,
{
    pub fn new(f: F) -> Self {
        Self {
            acc: L::agg_unit(),
            f,
            _marker: PhantomData,
        }
    }
}

impl<Spec, L, F> BstSeeker for SeekByAccCond<Spec, L, F>
where
    Spec: BstSpec<Data: BstDataAccess<data::marker::LazyMap, Value = LazyMapElement<L>>>,
    L: LazyMapMonoid,
    F: FnMut(&L::Agg) -> bool,
{
    type Spec = Spec;

    fn bst_seek(&mut self, node: BstImmutRef<'_, Self::Spec>) -> Ordering {
        if let Ok(left) = node.reborrow().left().descend() {
            let left_agg = &left.into_data().bst_data().agg;
            let nagg = L::agg_operate(&self.acc, left_agg);
            if (self.f)(&nagg) {
                return Ordering::Greater;
            }
            let nagg = L::agg_operate(
                &nagg,
                &L::single_agg(&node.reborrow().into_data().bst_data().key),
            );
            if (self.f)(&nagg) {
                Ordering::Equal
            } else {
                self.acc = nagg;
                Ordering::Less
            }
        } else {
            let nagg = L::agg_operate(
                &self.acc,
                &L::single_agg(&node.reborrow().into_data().bst_data().key),
            );
            if (self.f)(&nagg) {
                Ordering::Equal
            } else {
                self.acc = nagg;
                Ordering::Less
            }
        }
    }
}

pub struct SeekByRaccCond<Spec, L, F>
where
    L: LazyMapMonoid,
{
    acc: L::Agg,
    f: F,
    _marker: PhantomData<fn() -> (Spec, L)>,
}

impl<Spec, L, F> SeekByRaccCond<Spec, L, F>
where
    L: LazyMapMonoid,
    F: FnMut(&L::Agg) -> bool,
{
    pub fn new(f: F) -> Self {
        Self {
            acc: L::agg_unit(),
            f,
            _marker: PhantomData,
        }
    }
}

impl<Spec, L, F> BstSeeker for SeekByRaccCond<Spec, L, F>
where
    Spec: BstSpec<Data: BstDataAccess<data::marker::LazyMap, Value = LazyMapElement<L>>>,
    L: LazyMapMonoid,
    F: FnMut(&L::Agg) -> bool,
{
    type Spec = Spec;

    fn bst_seek(&mut self, node: BstImmutRef<'_, Self::Spec>) -> Ordering {
        if let Ok(right) = node.reborrow().right().descend() {
            let right_agg = &right.into_data().bst_data().agg;
            let nagg = L::agg_operate(right_agg, &self.acc);
            if (self.f)(&nagg) {
                return Ordering::Less;
            }
            let nagg = L::agg_operate(
                &L::single_agg(&node.reborrow().into_data().bst_data().key),
                &nagg,
            );
            if (self.f)(&nagg) {
                Ordering::Equal
            } else {
                self.acc = nagg;
                Ordering::Greater
            }
        } else {
            let nagg = L::agg_operate(
                &L::single_agg(&node.reborrow().into_data().bst_data().key),
                &self.acc,
            );
            if (self.f)(&nagg) {
                Ordering::Equal
            } else {
                self.acc = nagg;
                Ordering::Greater
            }
        }
    }
}
