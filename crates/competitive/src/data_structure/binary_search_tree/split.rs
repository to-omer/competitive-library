use super::{
    BstDataAccess, BstDataMutRef, BstImmutRef, BstRoot, BstSeeker, BstSpec,
    data::{self},
    seeker::{SeekByKey, SeekBySize},
};
use std::{
    borrow::Borrow,
    ops::{Bound, RangeBounds},
};

pub struct Split<'a, Spec>
where
    Spec: BstSpec,
{
    left: Option<BstRoot<Spec>>,
    right: Option<BstRoot<Spec>>,
    root: &'a mut Option<BstRoot<Spec>>,
}

impl<'a, Spec> Split<'a, Spec>
where
    Spec: BstSpec,
{
    pub fn new<Seek>(node: &'a mut Option<BstRoot<Spec>>, seeker: Seek, eq_left: bool) -> Self
    where
        Seek: BstSeeker<Spec = Spec>,
    {
        let (left, right) = Spec::split(node.take(), seeker, eq_left);
        Self {
            left,
            right,
            root: node,
        }
    }

    pub fn left(&self) -> Option<BstImmutRef<'_, Spec>> {
        self.left.as_ref().map(|node| node.reborrow())
    }

    pub fn right(&self) -> Option<BstImmutRef<'_, Spec>> {
        self.right.as_ref().map(|node| node.reborrow())
    }

    pub fn left_datamut(&mut self) -> Option<BstDataMutRef<'_, Spec>> {
        self.left.as_mut().map(|node| node.borrow_datamut())
    }

    pub fn right_datamut(&mut self) -> Option<BstDataMutRef<'_, Spec>> {
        self.right.as_mut().map(|node| node.borrow_datamut())
    }

    pub fn manually_merge<F>(&mut self, mut f: F)
    where
        F: FnMut(Option<BstRoot<Spec>>, Option<BstRoot<Spec>>) -> Option<BstRoot<Spec>>,
    {
        self.left = f(self.left.take(), self.right.take());
    }
}

impl<'a, Spec> Drop for Split<'a, Spec>
where
    Spec: BstSpec,
{
    fn drop(&mut self) {
        *self.root = Spec::merge(self.left.take(), self.right.take());
    }
}

pub struct Split3<'a, Spec>
where
    Spec: BstSpec,
{
    left: Option<BstRoot<Spec>>,
    mid: Option<BstRoot<Spec>>,
    right: Option<BstRoot<Spec>>,
    root: &'a mut Option<BstRoot<Spec>>,
}

impl<'a, Spec> Split3<'a, Spec>
where
    Spec: BstSpec,
{
    pub fn new<Seek1, Seek2>(
        node: &'a mut Option<BstRoot<Spec>>,
        start: Bound<Seek1>,
        end: Bound<Seek2>,
    ) -> Self
    where
        Seek1: BstSeeker<Spec = Spec>,
        Seek2: BstSeeker<Spec = Spec>,
    {
        let (mut rest, right) = match end {
            Bound::Included(seeker) => Spec::split(node.take(), seeker, true),
            Bound::Excluded(seeker) => Spec::split(node.take(), seeker, false),
            Bound::Unbounded => (node.take(), None),
        };
        let (left, mid) = match start {
            Bound::Included(seeker) => Spec::split(rest.take(), seeker, false),
            Bound::Excluded(seeker) => Spec::split(rest.take(), seeker, true),
            Bound::Unbounded => (None, rest),
        };
        Self {
            left,
            mid,
            right,
            root: node,
        }
    }

    pub fn left(&self) -> Option<BstImmutRef<'_, Spec>> {
        self.left.as_ref().map(|node| node.reborrow())
    }

    pub fn mid(&self) -> Option<BstImmutRef<'_, Spec>> {
        self.mid.as_ref().map(|node| node.reborrow())
    }

    pub fn right(&self) -> Option<BstImmutRef<'_, Spec>> {
        self.right.as_ref().map(|node| node.reborrow())
    }

    pub fn left_datamut(&mut self) -> Option<BstDataMutRef<'_, Spec>> {
        self.left.as_mut().map(|node| node.borrow_datamut())
    }

    pub fn mid_datamut(&mut self) -> Option<BstDataMutRef<'_, Spec>> {
        self.mid.as_mut().map(|node| node.borrow_datamut())
    }

    pub fn right_datamut(&mut self) -> Option<BstDataMutRef<'_, Spec>> {
        self.right.as_mut().map(|node| node.borrow_datamut())
    }

    pub fn manually_merge<F>(&mut self, mut f: F)
    where
        F: FnMut(Option<BstRoot<Spec>>, Option<BstRoot<Spec>>) -> Option<BstRoot<Spec>>,
    {
        let rest = f(self.mid.take(), self.right.take());
        self.mid = f(self.left.take(), rest);
    }

    pub fn seek_by_key<K, Q, R>(node: &'a mut Option<BstRoot<Spec>>, range: R) -> Self
    where
        Spec::Data: BstDataAccess<data::marker::Key, Value = K>,
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
        Self::new(node, start, end)
    }

    pub fn seek_by_size<R>(node: &'a mut Option<BstRoot<Spec>>, range: R) -> Self
    where
        Spec::Data: BstDataAccess<data::marker::Size, Value = usize>,
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
        Self::new(node, start, end)
    }
}

impl<'a, Spec> Drop for Split3<'a, Spec>
where
    Spec: BstSpec,
{
    fn drop(&mut self) {
        let rest = Spec::merge(self.mid.take(), self.right.take());
        *self.root = Spec::merge(self.left.take(), rest);
    }
}
