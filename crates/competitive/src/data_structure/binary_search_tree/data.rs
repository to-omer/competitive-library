use super::{BstDataMutRef, BstSpec, LazyMapMonoid, Monoid, MonoidAct};
use std::{fmt, fmt::Debug, mem::replace};

pub trait BstDataAccess<Tag> {
    type Value: ?Sized;

    fn bst_data(&self) -> &Self::Value;

    fn bst_data_mut(&mut self) -> &mut Self::Value;
}

pub struct MonoidAggElement<M>
where
    M: Monoid,
{
    pub agg: M::T,
}

impl<M> Debug for MonoidAggElement<M>
where
    M: Monoid,
    M::T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MonoidAggElement")
            .field("agg", &self.agg)
            .finish()
    }
}

impl<M> MonoidAggElement<M>
where
    M: Monoid,
{
    pub fn top_down<Spec>(_node: BstDataMutRef<'_, Spec>)
    where
        Spec: BstSpec,
        Spec::Data: BstDataAccess<marker::MonoidAct, Value = Self>,
    {
    }

    pub fn bottom_up<Spec>(mut node: BstDataMutRef<'_, Spec>)
    where
        Spec: BstSpec,
        Spec::Data: BstDataAccess<marker::MonoidAct, Value = Self>,
    {
        let mut agg = M::unit();
        if let Ok(left) = node.reborrow().left().descend() {
            agg = M::operate(&left.into_data().bst_data().agg, &agg);
        }
        if let Ok(right) = node.reborrow().right().descend() {
            agg = M::operate(&agg, &right.into_data().bst_data().agg);
        }
        node.data_mut().bst_data_mut().agg = agg;
    }
}

pub struct MonoidActElement<M>
where
    M: MonoidAct,
{
    pub key: M::Key,
    pub act: M::Act,
}

impl<M> Debug for MonoidActElement<M>
where
    M: MonoidAct,
    M::Key: Debug,
    M::Act: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MonoidActElement")
            .field("key", &self.key)
            .field("act", &self.act)
            .finish()
    }
}

impl<M> MonoidActElement<M>
where
    M: MonoidAct,
{
    pub fn from_key(key: M::Key) -> Self {
        Self {
            key,
            act: M::unit(),
        }
    }

    pub fn update_act<Spec>(mut node: BstDataMutRef<'_, Spec>, act: &M::Act)
    where
        Spec: BstSpec,
        Spec::Data: BstDataAccess<marker::MonoidAct, Value = Self>,
    {
        M::operate_assign(&mut node.data_mut().bst_data_mut().act, act);
        M::act_assign(&mut node.data_mut().bst_data_mut().key, act);
    }

    pub fn top_down<Spec>(mut node: BstDataMutRef<'_, Spec>)
    where
        Spec: BstSpec,
        Spec::Data: BstDataAccess<marker::MonoidAct, Value = Self>,
    {
        let act = replace(&mut node.data_mut().bst_data_mut().act, M::unit());
        if let Ok(left) = node.reborrow_datamut().left().descend() {
            Self::update_act(left, &act);
        }
        if let Ok(right) = node.reborrow_datamut().right().descend() {
            Self::update_act(right, &act);
        }
    }
}

pub struct LazyMapElement<L>
where
    L: LazyMapMonoid,
{
    pub key: L::Key,
    pub agg: L::Agg,
    pub act: L::Act,
}

impl<L> Debug for LazyMapElement<L>
where
    L: LazyMapMonoid,
    L::Key: Debug,
    L::Agg: Debug,
    L::Act: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyMapElement")
            .field("key", &self.key)
            .field("agg", &self.agg)
            .field("act", &self.act)
            .finish()
    }
}

impl<L> LazyMapElement<L>
where
    L: LazyMapMonoid,
{
    pub fn from_key(key: L::Key) -> Self {
        let agg = L::single_agg(&key);
        Self {
            key,
            agg,
            act: L::act_unit(),
        }
    }

    pub fn update_act<Spec>(mut node: BstDataMutRef<'_, Spec>, act: &L::Act)
    where
        Spec: BstSpec,
        Spec::Data: BstDataAccess<marker::LazyMap, Value = Self>,
    {
        L::act_operate_assign(&mut node.data_mut().bst_data_mut().act, act);
        node.data_mut().bst_data_mut().key =
            L::act_key(&node.reborrow().into_data().bst_data().key, act);
        if let Some(nxlazy) = L::act_agg(&node.reborrow().into_data().bst_data().agg, act) {
            node.data_mut().bst_data_mut().agg = nxlazy;
        } else {
            Self::top_down(node.reborrow_datamut());
            Self::bottom_up(node.reborrow_datamut());
        }
    }

    pub fn top_down<Spec>(mut node: BstDataMutRef<'_, Spec>)
    where
        Spec: BstSpec,
        Spec::Data: BstDataAccess<marker::LazyMap, Value = Self>,
    {
        let act = replace(&mut node.data_mut().bst_data_mut().act, L::act_unit());
        if let Ok(left) = node.reborrow_datamut().left().descend() {
            Self::update_act(left, &act);
        }
        if let Ok(right) = node.reborrow_datamut().right().descend() {
            Self::update_act(right, &act);
        }
    }

    pub fn bottom_up<Spec>(mut node: BstDataMutRef<'_, Spec>)
    where
        Spec: BstSpec,
        Spec::Data: BstDataAccess<marker::LazyMap, Value = Self>,
    {
        let mut agg = L::single_agg(&node.reborrow().into_data().bst_data().key);
        if let Ok(left) = node.reborrow().left().descend() {
            agg = L::agg_operate(&left.into_data().bst_data().agg, &agg);
        }
        if let Ok(right) = node.reborrow().right().descend() {
            agg = L::agg_operate(&agg, &right.into_data().bst_data().agg);
        }
        node.data_mut().bst_data_mut().agg = agg;
    }
}

pub mod marker {
    pub enum Key {}
    pub enum Size {}
    pub enum MonoidAct {}
    pub enum LazyMap {}
}
