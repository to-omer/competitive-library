use super::{Allocator, BstSeeker};
use std::{marker::PhantomData, ptr::NonNull};

pub trait BstSpec: Sized {
    type Parent: ParentStrategy<Data = Self::Data>;
    type Data;

    fn top_down(_node: BstDataMutRef<'_, Self>) {}

    fn bottom_up(_node: BstDataMutRef<'_, Self>) {}

    fn merge(left: Option<BstRoot<Self>>, right: Option<BstRoot<Self>>) -> Option<BstRoot<Self>>;

    fn split<Seeker>(
        node: Option<BstRoot<Self>>,
        seeker: Seeker,
        eq_left: bool,
    ) -> (Option<BstRoot<Self>>, Option<BstRoot<Self>>)
    where
        Seeker: BstSeeker<Spec = Self>;
}

pub struct BstNode<Data, Parent = WithNoParent<Data>> {
    pub data: Data,
    pub parent: Parent,
    pub child: [Option<NonNull<BstNode<Data, Parent>>>; 2],
}

impl<Data, Parent> BstNode<Data, Parent>
where
    Parent: Default,
{
    pub fn new(data: Data) -> Self {
        Self {
            data,
            parent: Parent::default(),
            child: [None, None],
        }
    }
}

pub trait ParentStrategy: Sized + Default {
    type Data;

    fn take_parent<Spec>(_node: BstNodeRef<marker::Mut<'_>, Spec>)
    where
        Spec: BstSpec<Data = Self::Data, Parent = Self>,
    {
    }

    fn set_parent<Spec>(
        _node: BstNodeRef<marker::Mut<'_>, Spec>,
        _parent: Option<NonNull<BstNode<Spec::Data, Self>>>,
    ) where
        Spec: BstSpec<Data = Self::Data, Parent = Self>,
    {
    }
}

pub struct WithNoParent<Data> {
    _marker: PhantomData<fn() -> Data>,
}

impl<Data> Default for WithNoParent<Data> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Data> ParentStrategy for WithNoParent<Data> {
    type Data = Data;

    fn take_parent<Spec>(_node: BstNodeRef<marker::Mut<'_>, Spec>)
    where
        Spec: BstSpec<Data = Self::Data, Parent = Self>,
    {
    }

    fn set_parent<Spec>(
        _node: BstNodeRef<marker::Mut<'_>, Spec>,
        _parent: Option<NonNull<BstNode<Spec::Data, Self>>>,
    ) where
        Spec: BstSpec<Data = Self::Data, Parent = Self>,
    {
    }
}

pub struct WithParent<Data> {
    pub parent: Option<NonNull<BstNode<Data, Self>>>,
}

impl<Data> Default for WithParent<Data> {
    fn default() -> Self {
        Self {
            parent: Default::default(),
        }
    }
}

impl<Data> ParentStrategy for WithParent<Data> {
    type Data = Data;

    fn take_parent<Spec>(mut node: BstNodeRef<marker::Mut<'_>, Spec>)
    where
        Spec: BstSpec<Data = Self::Data, Parent = Self>,
    {
        unsafe { node.node.as_mut().parent = Default::default() };
    }

    fn set_parent<Spec>(
        mut node: BstNodeRef<marker::Mut<'_>, Spec>,
        parent: Option<NonNull<BstNode<Spec::Data, Self>>>,
    ) where
        Spec: BstSpec<Data = Self::Data, Parent = Self>,
    {
        unsafe { node.node.as_mut().parent.parent = parent };
    }
}

impl<Data> WithParent<Data> {
    pub unsafe fn resolve_top_down<Spec>(mut node: NonNull<BstNode<Spec::Data, Spec::Parent>>)
    where
        Spec: BstSpec<Data = Data, Parent = Self>,
    {
        unsafe fn datamut<Spec, Data>(
            node: &mut NonNull<BstNode<Spec::Data, Spec::Parent>>,
        ) -> BstNodeRef<marker::DataMut<'_>, Spec>
        where
            Spec: BstSpec<Data = Data, Parent = WithParent<Data>>,
        {
            BstNodeRef {
                node: *node,
                _marker: PhantomData,
            }
        }
        unsafe {
            let (mut node, mut stack) = datamut::<Spec, Data>(&mut node).root_path();
            while let Some(is_left) = stack.pop() {
                Spec::top_down(node.reborrow_datamut());
                if is_left {
                    node = node.left().descend().unwrap_unchecked();
                } else {
                    node = node.right().descend().unwrap_unchecked();
                }
            }
            Spec::top_down(node.reborrow_datamut());
        }
    }
}

pub struct BstNodeRef<BorrowType, Spec>
where
    Spec: BstSpec,
{
    pub node: NonNull<BstNode<Spec::Data, Spec::Parent>>,
    _marker: PhantomData<BorrowType>,
}

impl<'a, Spec> Copy for BstNodeRef<marker::Immut<'a>, Spec>
where
    Spec: BstSpec,
    Spec::Data: 'a,
{
}
impl<'a, Spec> Clone for BstNodeRef<marker::Immut<'a>, Spec>
where
    Spec: BstSpec,
    Spec::Data: 'a,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<BorrowType, Spec> BstNodeRef<BorrowType, Spec>
where
    Spec: BstSpec,
    BorrowType: marker::BorrowType,
{
    pub unsafe fn new_unchecked(node: NonNull<BstNode<Spec::Data, Spec::Parent>>) -> Self {
        Self {
            node,
            _marker: PhantomData,
        }
    }
    pub fn reborrow(&self) -> BstNodeRef<marker::Immut<'_>, Spec> {
        BstNodeRef {
            node: self.node,
            _marker: PhantomData,
        }
    }
    pub fn left(self) -> BstEdgeHandle<Self, marker::Left> {
        BstEdgeHandle {
            node: self,
            _marker: PhantomData,
        }
    }
    pub fn right(self) -> BstEdgeHandle<Self, marker::Right> {
        BstEdgeHandle {
            node: self,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, Spec, Data> BstNodeRef<BorrowType, Spec>
where
    Spec: BstSpec<Data = Data, Parent = WithParent<Data>>,
    BorrowType: marker::BorrowType,
{
    pub fn ascend(self) -> Result<BstNodeRef<BorrowType, Spec>, Self> {
        // const { [()][!BorrowType::TRAVERSAL_PERMIT as usize] };
        assert!(BorrowType::TRAVERSAL_PERMIT);
        let parent = unsafe { self.node.as_ref().parent.parent };
        parent
            .map(|node| BstNodeRef {
                node,
                _marker: PhantomData,
            })
            .ok_or(self)
    }
    pub fn root_path(self) -> (Self, Vec<bool>) {
        let mut node = self;
        let mut nn = node.node;
        let mut stack = vec![];
        let root = loop {
            match node.ascend() {
                Ok(parent) => {
                    node = parent;
                    stack.push(
                        node.reborrow()
                            .left()
                            .descend()
                            .map_or(false, |node| node.node == nn),
                    );
                    nn = node.node;
                }
                Err(node) => {
                    break node;
                }
            }
        };
        (root, stack)
    }
}

impl<Spec> BstNodeRef<marker::Owned, Spec>
where
    Spec: BstSpec,
{
    pub fn new(node: NonNull<BstNode<Spec::Data, Spec::Parent>>) -> Self {
        Self {
            node,
            _marker: PhantomData,
        }
    }
    pub fn from_data<A>(data: Spec::Data, allocator: &mut A) -> Self
    where
        A: Allocator<BstNode<Spec::Data, Spec::Parent>>,
    {
        Self::new(allocator.allocate(BstNode::new(data)))
    }
    pub fn borrow_mut(&mut self) -> BstNodeRef<marker::Mut<'_>, Spec> {
        BstNodeRef {
            node: self.node,
            _marker: PhantomData,
        }
    }
    pub fn borrow_datamut(&mut self) -> BstNodeRef<marker::DataMut<'_>, Spec> {
        BstNodeRef {
            node: self.node,
            _marker: PhantomData,
        }
    }
    pub fn into_dying(self) -> BstNodeRef<marker::Dying, Spec> {
        BstNodeRef {
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<'a, Spec> BstNodeRef<marker::Immut<'a>, Spec>
where
    Spec: BstSpec,
    Spec::Parent: 'a,
    Spec::Data: 'a,
{
    pub fn into_data(self) -> &'a Spec::Data {
        unsafe { &self.node.as_ref().data }
    }

    pub fn traverse<F>(self, f: &mut F)
    where
        Spec::Data: 'a,
        F: FnMut(Self),
    {
        if let Ok(left) = self.left().descend() {
            left.traverse(f);
        }
        f(self);
        if let Ok(right) = self.right().descend() {
            right.traverse(f);
        }
    }

    pub fn leftmost(self) -> Option<Self> {
        let mut node = self;
        while let Ok(left) = node.left().descend() {
            node = left;
        }
        Some(node)
    }

    pub fn rightmost(self) -> Option<Self> {
        let mut node = self;
        while let Ok(right) = node.right().descend() {
            node = right;
        }
        Some(node)
    }
}

impl<'a, Spec> BstNodeRef<marker::DataMut<'a>, Spec>
where
    Spec: BstSpec,
{
    pub fn reborrow_datamut(&mut self) -> BstNodeRef<marker::DataMut<'_>, Spec> {
        BstNodeRef {
            node: self.node,
            _marker: PhantomData,
        }
    }
    pub fn data_mut(&mut self) -> &mut Spec::Data {
        unsafe { &mut self.node.as_mut().data }
    }
}

impl<'a, Spec> BstNodeRef<marker::DataMut<'a>, Spec>
where
    Spec: BstSpec,
    Spec::Data: 'a,
    Spec::Parent: 'a,
{
    pub fn into_data_mut(mut self) -> &'a mut Spec::Data {
        unsafe { &mut self.node.as_mut().data }
    }
}

impl<'a, Spec> BstNodeRef<marker::Mut<'a>, Spec>
where
    Spec: BstSpec,
{
    pub fn reborrow_datamut(&mut self) -> BstNodeRef<marker::DataMut<'_>, Spec> {
        BstNodeRef {
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<'a, Spec> BstNodeRef<marker::Mut<'a>, Spec>
where
    Spec: BstSpec,
    Spec::Data: 'a,
{
    pub fn dormant(self) -> BstNodeRef<marker::DormantMut, Spec> {
        BstNodeRef {
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<Spec> BstNodeRef<marker::DormantMut, Spec>
where
    Spec: BstSpec,
{
    pub unsafe fn awaken<'a>(self) -> BstNodeRef<marker::Mut<'a>, Spec> {
        BstNodeRef {
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<Spec> BstNodeRef<marker::Dying, Spec>
where
    Spec: BstSpec,
{
    pub unsafe fn into_data<A>(self, allocator: &mut A) -> Spec::Data
    where
        A: Allocator<BstNode<Spec::Data, Spec::Parent>>,
    {
        assert!(self.reborrow().left().descend().is_err());
        assert!(self.reborrow().right().descend().is_err());
        allocator.deallocate(self.node).data
    }

    pub unsafe fn drop_all<A>(self, allocator: &mut A)
    where
        A: Allocator<BstNode<Spec::Data, Spec::Parent>>,
    {
        let BstNode { child, .. } = allocator.deallocate(self.node);
        for node in child.into_iter().flatten() {
            unsafe {
                BstNodeRef::<marker::Owned, Spec>::new(node)
                    .into_dying()
                    .drop_all(allocator)
            }
        }
    }
}

pub struct BstEdgeHandle<Node, Dir> {
    node: Node,
    _marker: PhantomData<Dir>,
}

impl<BorrowType, Spec, Dir> BstEdgeHandle<BstNodeRef<BorrowType, Spec>, Dir>
where
    Spec: BstSpec,
    BorrowType: marker::BorrowType,
    Dir: marker::BstDirection,
{
    pub fn descend(self) -> Result<BstNodeRef<BorrowType, Spec>, Self> {
        // const { [()][!BorrowType::TRAVERSAL_PERMIT as usize] };
        assert!(BorrowType::TRAVERSAL_PERMIT);
        let child = unsafe { self.node.node.as_ref().child.get_unchecked(Dir::IDX) };
        child
            .map(|node| BstNodeRef {
                node,
                _marker: PhantomData,
            })
            .ok_or(self)
    }
}

impl<'a, Spec, Dir> BstEdgeHandle<BstNodeRef<marker::Mut<'a>, Spec>, Dir>
where
    Spec: BstSpec,
    Dir: marker::BstDirection,
{
    pub unsafe fn take(&mut self) -> Option<BstNodeRef<marker::Owned, Spec>> {
        let child = unsafe { self.node.node.as_mut().child.get_unchecked_mut(Dir::IDX) };
        child.take().map(|node| {
            let mut node = BstNodeRef {
                node,
                _marker: PhantomData,
            };
            Spec::Parent::take_parent(node.borrow_mut());
            node
        })
    }
    pub unsafe fn replace(
        &mut self,
        mut other: BstNodeRef<marker::Owned, Spec>,
    ) -> Option<BstNodeRef<marker::Owned, Spec>> {
        let child = unsafe { self.node.node.as_mut().child.get_unchecked_mut(Dir::IDX) };
        Spec::Parent::set_parent(other.borrow_mut(), Some(self.node.node));
        child.replace(other.node).map(|node| {
            let mut node = BstNodeRef {
                node,
                _marker: PhantomData,
            };
            Spec::Parent::take_parent(node.borrow_mut());
            node
        })
    }
    pub unsafe fn set(&mut self, mut other: BstNodeRef<marker::Owned, Spec>) {
        let child = unsafe { self.node.node.as_mut().child.get_unchecked_mut(Dir::IDX) };
        Spec::Parent::set_parent(other.borrow_mut(), Some(self.node.node));
        *child = Some(other.node);
    }
}

pub type BstRoot<Spec> = BstNodeRef<marker::Owned, Spec>;
pub type BstDataMutRef<'a, Spec> = BstNodeRef<marker::DataMut<'a>, Spec>;
pub type BstImmutRef<'a, Spec> = BstNodeRef<marker::Immut<'a>, Spec>;

pub mod marker {
    use std::marker::PhantomData;

    pub enum Left {}
    pub enum Right {}
    pub trait BstDirection {
        const IDX: usize;
    }
    impl BstDirection for Left {
        const IDX: usize = 0;
    }
    impl BstDirection for Right {
        const IDX: usize = 1;
    }

    pub enum Owned {}
    pub enum Dying {}
    pub enum DormantMut {}
    pub struct Immut<'a>(PhantomData<&'a ()>);
    pub struct Mut<'a>(PhantomData<&'a mut ()>);
    pub struct DataMut<'a>(PhantomData<&'a mut ()>);

    pub trait BorrowType {
        const TRAVERSAL_PERMIT: bool = true;
    }
    impl BorrowType for Owned {
        const TRAVERSAL_PERMIT: bool = false;
    }
    impl BorrowType for Dying {}
    impl BorrowType for DormantMut {}
    impl<'a> BorrowType for Immut<'a> {}
    impl<'a> BorrowType for Mut<'a> {}
    impl<'a> BorrowType for DataMut<'a> {}
}
