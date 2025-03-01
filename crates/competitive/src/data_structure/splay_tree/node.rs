use super::Allocator;
use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{replace, swap},
    ops::Bound,
    ptr::NonNull,
};

pub trait SplaySpec: Sized {
    type T;
    fn has_bottom_up() -> bool {
        false
    }
    fn top_down(_node: NodeRef<marker::DataMut<'_>, Self>) {}
    fn bottom_up(_node: NodeRef<marker::DataMut<'_>, Self>) {}
}

pub trait SplaySeeker {
    type S: SplaySpec;
    fn splay_seek(&mut self, _node: NodeRef<marker::Immut<'_>, Self::S>) -> Ordering;
}

pub struct SeekLeft<S> {
    _marker: PhantomData<fn() -> S>,
}
impl<S> Default for SeekLeft<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
impl<S> SeekLeft<S> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<S> SplaySeeker for SeekLeft<S>
where
    S: SplaySpec,
{
    type S = S;
    fn splay_seek(&mut self, _node: NodeRef<marker::Immut<'_>, Self::S>) -> Ordering {
        Ordering::Less
    }
}

pub struct SeekRight<S> {
    _marker: PhantomData<fn() -> S>,
}
impl<S> Default for SeekRight<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
impl<S> SeekRight<S> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<S> SplaySeeker for SeekRight<S>
where
    S: SplaySpec,
{
    type S = S;
    fn splay_seek(&mut self, _node: NodeRef<marker::Immut<'_>, Self::S>) -> Ordering {
        Ordering::Greater
    }
}

pub struct Node<T> {
    data: T,
    left: Option<NonNull<Node<T>>>,
    right: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            left: None,
            right: None,
        }
    }
}

pub struct NodeRef<B, S>
where
    S: SplaySpec,
{
    node: NonNull<Node<S::T>>,
    _marker: PhantomData<B>,
}

pub struct Root<S>
where
    S: SplaySpec,
{
    root: Option<NodeRef<marker::Owned, S>>,
}

impl<S> Default for Root<S>
where
    S: SplaySpec,
{
    fn default() -> Self {
        Self { root: None }
    }
}

impl<'a, S> Copy for NodeRef<marker::Immut<'a>, S>
where
    S: SplaySpec,
    S::T: 'a,
{
}
impl<'a, S> Clone for NodeRef<marker::Immut<'a>, S>
where
    S: SplaySpec,
    S::T: 'a,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<B, S> NodeRef<B, S>
where
    S: SplaySpec,
{
    unsafe fn new_unchecked(node: NonNull<Node<S::T>>) -> Self {
        Self {
            node,
            _marker: PhantomData,
        }
    }
    fn reborrow(&self) -> NodeRef<marker::Immut<'_>, S> {
        unsafe { NodeRef::new_unchecked(self.node) }
    }
    fn as_ptr(&self) -> *mut Node<S::T> {
        self.node.as_ptr()
    }
}

impl<S> NodeRef<marker::Owned, S>
where
    S: SplaySpec,
{
    pub fn new(node: NonNull<Node<S::T>>) -> Self {
        unsafe { NodeRef::new_unchecked(node) }
    }
    pub unsafe fn from_data<A>(data: S::T, allocator: &mut A) -> Self
    where
        A: Allocator<Node<S::T>>,
    {
        Self::new(allocator.allocate(Node::new(data)))
    }
    pub fn borrow_mut(&mut self) -> NodeRef<marker::Mut<'_>, S> {
        unsafe { NodeRef::new_unchecked(self.node) }
    }
    pub fn borrow_datamut(&mut self) -> NodeRef<marker::DataMut<'_>, S> {
        unsafe { NodeRef::new_unchecked(self.node) }
    }
    pub fn into_dying(self) -> NodeRef<marker::Dying, S> {
        unsafe { NodeRef::new_unchecked(self.node) }
    }
}

impl<'a, S> NodeRef<marker::Immut<'a>, S>
where
    S: SplaySpec,
    S::T: 'a,
{
    pub fn data(&self) -> &'a S::T {
        unsafe { &(*self.as_ptr()).data }
    }
    pub fn left(&self) -> Option<Self> {
        unsafe {
            (*self.as_ptr())
                .left
                .map(|node| NodeRef::new_unchecked(node))
        }
    }
    pub fn right(&self) -> Option<Self> {
        unsafe {
            (*self.as_ptr())
                .right
                .map(|node| NodeRef::new_unchecked(node))
        }
    }
    pub fn traverse<F>(self, f: &mut F)
    where
        S::T: 'a,
        F: FnMut(Self),
    {
        if let Some(left) = self.clone().left() {
            left.traverse(f);
        }
        f(self);
        if let Some(right) = self.clone().right() {
            right.traverse(f);
        }
    }
}

impl<'a, S> NodeRef<marker::DataMut<'a>, S>
where
    S: SplaySpec,
    S::T: 'a,
{
    pub fn data(&self) -> &'a S::T {
        unsafe { &(*self.as_ptr()).data }
    }
    pub fn data_mut(&self) -> &'a mut S::T {
        unsafe { &mut (*self.as_ptr()).data }
    }
    pub fn left(&self) -> Option<Self> {
        unsafe {
            (*self.as_ptr())
                .left
                .map(|node| NodeRef::new_unchecked(node))
        }
    }
    pub fn right(&self) -> Option<Self> {
        unsafe {
            (*self.as_ptr())
                .right
                .map(|node| NodeRef::new_unchecked(node))
        }
    }
    pub fn reverse(&self) {
        unsafe {
            let node = &mut (*self.as_ptr());
            swap(&mut node.left, &mut node.right);
        }
    }
}

impl<'a, S> NodeRef<marker::Mut<'a>, S>
where
    S: SplaySpec,
    S::T: 'a,
{
    pub fn data(self) -> &'a S::T {
        unsafe { &(*self.as_ptr()).data }
    }
    pub fn data_mut(self) -> &'a mut S::T {
        unsafe { &mut (*self.as_ptr()).data }
    }
    pub fn take_left(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        Some(NodeRef::new(unsafe { (*self.as_ptr()).left.take()? }))
    }
    pub fn take_right(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        Some(NodeRef::new(unsafe { (*self.as_ptr()).right.take()? }))
    }
    pub fn set_left(&mut self, node: Option<NodeRef<marker::Owned, S>>) {
        unsafe { (*self.as_ptr()).left = node.map(|node| node.node) }
    }
    pub fn set_right(&mut self, node: Option<NodeRef<marker::Owned, S>>) {
        unsafe { (*self.as_ptr()).right = node.map(|node| node.node) }
    }
}

impl<S> NodeRef<marker::Dying, S>
where
    S: SplaySpec,
{
    pub unsafe fn into_inner(self) -> NonNull<Node<S::T>> {
        let node = self.node;
        unsafe {
            debug_assert!((*node.as_ptr()).left.is_none());
            debug_assert!((*node.as_ptr()).right.is_none());
        }
        node
    }
    pub unsafe fn into_data<A>(self, allocator: &mut A) -> S::T
    where
        A: Allocator<Node<S::T>>,
    {
        let Node { data, left, right } = allocator.deallocate(self.node);
        debug_assert!(left.is_none());
        debug_assert!(right.is_none());
        data
    }
}

impl<S> NodeRef<marker::Owned, S>
where
    S: SplaySpec,
{
    /// `cmp(key)`: [`Ordering`] between splaying and `key`
    pub fn splay_by<Seeker>(self, mut seeker: Seeker) -> (Ordering, Self)
    where
        Seeker: SplaySeeker<S = S>,
    {
        let mut x = self;
        // let dummy = Node { data: None, left: left_subtree, right: right_subtree };
        let mut left_subtree: Option<NonNull<Node<S::T>>> = None;
        let mut right_subtree: Option<NonNull<Node<S::T>>> = None;
        let mut left_entry = &mut left_subtree;
        let mut right_entry = &mut right_subtree;
        let mut stack = vec![];

        macro_rules! add {
            (@left Some($ptr:ident)) => { add!(@inner Some($ptr.node), left_entry $ptr right); };
            (@right Some($ptr:ident)) => { add!(@inner Some($ptr.node), right_entry $ptr left); };
            (@inner $node:expr, $entry:ident $ptr:ident $dir:ident) => {
                *$entry = $node;
                if S::has_bottom_up() {
                    stack.push($ptr.node);
                }
                $entry = unsafe { &mut (*$entry.as_mut().unwrap().as_ptr()).$dir };
            };
        }

        let root_ord = loop {
            S::top_down(x.borrow_datamut());
            match seeker.splay_seek(x.reborrow()) {
                Ordering::Less => {
                    if let Some(mut y) = x.borrow_mut().take_left() {
                        S::top_down(y.borrow_datamut());
                        match seeker.splay_seek(y.reborrow()) {
                            Ordering::Less => {
                                if let Some(mut z) = y.borrow_mut().take_left() {
                                    S::top_down(z.borrow_datamut());
                                    x.borrow_mut().set_left(y.borrow_mut().take_right());
                                    S::bottom_up(x.borrow_datamut());
                                    y.borrow_mut().set_right(Some(x));
                                    add!(@right Some(y));
                                    x = z;
                                } else {
                                    add!(@right Some(x));
                                    x = y;
                                    break Ordering::Less;
                                }
                            }
                            Ordering::Equal => {
                                add!(@right Some(x));
                                x = y;
                                break Ordering::Equal;
                            }
                            Ordering::Greater => {
                                if let Some(mut z) = y.borrow_mut().take_right() {
                                    S::top_down(z.borrow_datamut());
                                    add!(@right Some(x));
                                    add!(@left Some(y));
                                    x = z;
                                } else {
                                    add!(@right Some(x));
                                    x = y;
                                    break Ordering::Greater;
                                }
                            }
                        }
                    } else {
                        break Ordering::Less;
                    }
                }
                Ordering::Equal => break Ordering::Equal,
                Ordering::Greater => {
                    if let Some(mut y) = x.borrow_mut().take_right() {
                        S::top_down(y.borrow_datamut());
                        match seeker.splay_seek(y.reborrow()) {
                            Ordering::Less => {
                                if let Some(mut z) = y.borrow_mut().take_left() {
                                    S::top_down(z.borrow_datamut());
                                    add!(@left Some(x));
                                    add!(@right Some(y));
                                    x = z;
                                } else {
                                    add!(@left Some(x));
                                    x = y;
                                    break Ordering::Less;
                                }
                            }
                            Ordering::Equal => {
                                add!(@left Some(x));
                                x = y;
                                break Ordering::Equal;
                            }
                            Ordering::Greater => {
                                if let Some(mut z) = y.borrow_mut().take_right() {
                                    S::top_down(z.borrow_datamut());
                                    x.borrow_mut().set_right(y.borrow_mut().take_left());
                                    S::bottom_up(x.borrow_datamut());
                                    y.borrow_mut().set_left(Some(x));
                                    add!(@left Some(y));
                                    x = z;
                                } else {
                                    add!(@left Some(x));
                                    x = y;
                                    break Ordering::Greater;
                                }
                            }
                        }
                    } else {
                        break Ordering::Greater;
                    }
                }
            }
        };
        *left_entry = x.borrow_mut().take_left().map(|node| node.node);
        *right_entry = x.borrow_mut().take_right().map(|node| node.node);
        unsafe {
            x.borrow_mut()
                .set_left(left_subtree.map(|node| NodeRef::new_unchecked(node)));
            x.borrow_mut()
                .set_right(right_subtree.map(|node| NodeRef::new_unchecked(node)));
            if S::has_bottom_up() {
                while let Some(node) = stack.pop() {
                    S::bottom_up(NodeRef::new_unchecked(node));
                }
            }
        }
        S::bottom_up(x.borrow_datamut());
        (root_ord, x)
    }
    pub fn insert_left(mut self, mut node: Self) -> Self {
        if let Some(left) = self.borrow_mut().take_left() {
            node.borrow_mut().set_left(Some(left));
            S::bottom_up(self.borrow_datamut());
        };
        node.borrow_mut().set_right(Some(self));
        S::bottom_up(node.borrow_datamut());
        node
    }
    pub fn insert_right(mut self, mut node: Self) -> Self {
        if let Some(right) = self.borrow_mut().take_right() {
            node.borrow_mut().set_right(Some(right));
            S::bottom_up(self.borrow_datamut());
        }
        node.borrow_mut().set_left(Some(self));
        S::bottom_up(node.borrow_datamut());
        node
    }
    pub fn insert_first(self, mut node: Self) -> Self {
        node.borrow_mut().set_right(Some(self));
        S::bottom_up(node.borrow_datamut());
        node
    }
    pub fn insert_last(self, mut node: Self) -> Self {
        node.borrow_mut().set_left(Some(self));
        S::bottom_up(node.borrow_datamut());
        node
    }
    pub fn merge(mut self, mut other: Self) -> Self {
        if other.reborrow().left().is_none() {
            S::top_down(other.borrow_datamut());
            other.borrow_mut().set_left(Some(self));
            S::bottom_up(other.borrow_datamut());
            other
        } else {
            self = self.splay_by(SeekRight::new()).1;
            self.borrow_mut().set_right(Some(other));
            S::bottom_up(self.borrow_datamut());
            self
        }
    }
}

impl<S> Root<S>
where
    S: SplaySpec,
{
    pub fn new(root: Option<NodeRef<marker::Owned, S>>) -> Self {
        Self { root }
    }
    pub unsafe fn from_single_nodes(nodes: Vec<NodeRef<marker::Owned, S>>) -> Self {
        unsafe { Self::from_single_nodes_inner(&nodes) }
    }
    unsafe fn from_single_nodes_inner(nodes: &[NodeRef<marker::Owned, S>]) -> Self {
        if nodes.is_empty() {
            Self::new(None)
        } else {
            let m = nodes.len() / 2;
            let left = unsafe { Self::from_single_nodes_inner(&nodes[..m]) };
            let right = unsafe { Self::from_single_nodes_inner(&nodes[m + 1..]) };
            let mut node = NodeRef::new(nodes[m].node);
            node.borrow_mut().set_left(left.root);
            node.borrow_mut().set_right(right.root);
            S::bottom_up(node.borrow_datamut());
            Self::new(Some(node))
        }
    }
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
    pub fn root(&self) -> Option<NodeRef<marker::Immut<'_>, S>> {
        Some(self.root.as_ref()?.reborrow())
    }
    pub fn root_data_mut(&mut self) -> Option<NodeRef<marker::DataMut<'_>, S>> {
        Some(self.root.as_mut()?.borrow_datamut())
    }
    pub fn splay_by<Seeker>(&mut self, seeker: Seeker) -> Option<Ordering>
    where
        Seeker: SplaySeeker<S = S>,
    {
        let (ord, root) = self.root.take()?.splay_by(seeker);
        self.root = Some(root);
        Some(ord)
    }
    pub fn insert_left(&mut self, mut node: NodeRef<marker::Owned, S>) {
        self.root = Some(match self.root.take() {
            Some(root) => root.insert_left(node),
            None => {
                S::bottom_up(node.borrow_datamut());
                node
            }
        });
    }
    pub fn insert_right(&mut self, mut node: NodeRef<marker::Owned, S>) {
        self.root = Some(match self.root.take() {
            Some(root) => root.insert_right(node),
            None => {
                S::bottom_up(node.borrow_datamut());
                node
            }
        });
    }
    pub fn insert_first(&mut self, mut node: NodeRef<marker::Owned, S>) {
        self.root = Some(match self.root.take() {
            Some(root) => root.insert_first(node),
            None => {
                S::bottom_up(node.borrow_datamut());
                node
            }
        });
    }
    pub fn insert_last(&mut self, mut node: NodeRef<marker::Owned, S>) {
        self.root = Some(match self.root.take() {
            Some(root) => root.insert_last(node),
            None => {
                S::bottom_up(node.borrow_datamut());
                node
            }
        });
    }
    pub fn take_root(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        let mut root = self.root.take()?;
        let right = root.borrow_mut().take_right();
        self.root = root.borrow_mut().take_left();
        self.append(&mut Self::new(right));
        Some(root)
    }
    pub fn take_first(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        let mut root = self.root.take()?.splay_by(SeekLeft::new()).1;
        let right = root.borrow_mut().take_right();
        self.root = right;
        Some(root)
    }
    pub fn take_last(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        let mut root = self.root.take()?.splay_by(SeekRight::new()).1;
        let left = root.borrow_mut().take_left();
        self.root = left;
        Some(root)
    }
    pub fn split_left(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        let root = self.root.as_mut()?;
        let left = root.borrow_mut().take_left();
        S::bottom_up(root.borrow_datamut());
        left
    }
    pub fn split_right(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        let root = self.root.as_mut()?;
        let right = root.borrow_mut().take_right();
        S::bottom_up(root.borrow_datamut());
        right
    }
    pub fn split_left_eq(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        let right = self.split_right();
        replace(&mut self.root, right)
    }
    pub fn split_right_eq(&mut self) -> Option<NodeRef<marker::Owned, S>> {
        let left = self.split_left();
        replace(&mut self.root, left)
    }
    pub fn append(&mut self, other: &mut Self) {
        self.root = match (self.root.take(), other.root.take()) {
            (Some(node), None) | (None, Some(node)) => Some(node),
            (Some(left), Some(right)) => Some(left.merge(right)),
            (None, None) => None,
        }
    }
    pub fn range<Seeker>(&mut self, start: Bound<Seeker>, end: Bound<Seeker>) -> NodeRange<'_, S>
    where
        Seeker: SplaySeeker<S = S>,
    {
        if self.is_empty() {
            return NodeRange::new(self);
        }
        let right = match end {
            Bound::Included(seeker) => match self.splay_by(seeker).unwrap() {
                Ordering::Greater | Ordering::Equal => self.split_right(),
                Ordering::Less => self.split_right_eq(),
            },
            Bound::Excluded(seeker) => match self.splay_by(seeker).unwrap() {
                Ordering::Greater => self.split_right(),
                Ordering::Less | Ordering::Equal => self.split_right_eq(),
            },
            Bound::Unbounded => None,
        };
        if self.is_empty() {
            return NodeRange::three_way(None, self, right);
        }
        let left = match start {
            Bound::Included(seeker) => match self.splay_by(seeker).unwrap() {
                Ordering::Less | Ordering::Equal => self.split_left(),
                Ordering::Greater => self.split_left_eq(),
            },
            Bound::Excluded(seeker) => match self.splay_by(seeker).unwrap() {
                Ordering::Less => self.split_left(),
                Ordering::Greater | Ordering::Equal => self.split_left_eq(),
            },
            Bound::Unbounded => None,
        };
        NodeRange::three_way(left, self, right)
    }
}

impl<S> Debug for Root<S>
where
    S: SplaySpec,
    S::T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_recurse<'a, S>(
            node: NodeRef<marker::Immut<'a>, S>,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result
        where
            S: SplaySpec,
            S::T: 'a + Debug,
        {
            write!(f, "[")?;
            if let Some(left) = node.left() {
                fmt_recurse(left, f)?;
            }
            node.data().fmt(f)?;
            if let Some(right) = node.right() {
                fmt_recurse(right, f)?;
            }
            write!(f, "]")?;
            Ok(())
        }
        if let Some(root) = self.root.as_ref() {
            let root = root.reborrow();
            fmt_recurse(root, f)?;
        }
        Ok(())
    }
}

pub struct NodeRange<'a, S>
where
    S: SplaySpec,
    S::T: 'a,
{
    front: Root<S>,
    back: Root<S>,
    root: &'a mut Root<S>,
}

impl<'a, S> Debug for NodeRange<'a, S>
where
    S: SplaySpec,
    S::T: 'a + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeRange")
            .field("front", &self.front)
            .field("back", &self.back)
            .field("root", &self.root)
            .finish()
    }
}
impl<'a, S> NodeRange<'a, S>
where
    S: SplaySpec,
    S::T: 'a,
{
    pub fn new(root: &'a mut Root<S>) -> Self {
        Self {
            front: Default::default(),
            back: Default::default(),
            root,
        }
    }
    pub fn three_way(
        front: Option<NodeRef<marker::Owned, S>>,
        root: &'a mut Root<S>,
        back: Option<NodeRef<marker::Owned, S>>,
    ) -> Self {
        Self {
            front: Root::new(front),
            back: Root::new(back),
            root,
        }
    }
    pub fn next_checked(&mut self) -> Option<NodeRef<marker::DataMut<'a>, S>> {
        let first = self.root.take_first()?;
        let noderef = unsafe { NodeRef::new_unchecked(first.node) };
        self.front.insert_last(first);
        Some(noderef)
    }
    pub fn next_back_checked(&mut self) -> Option<NodeRef<marker::DataMut<'a>, S>> {
        let last = self.root.take_last()?;
        let noderef = unsafe { NodeRef::new_unchecked(last.node) };
        self.back.insert_first(last);
        Some(noderef)
    }
    pub fn root(&self) -> &Root<S> {
        self.root
    }
    pub fn root_mut(&mut self) -> &mut Root<S> {
        self.root
    }
    pub fn front(&self) -> &Root<S> {
        &self.front
    }
    pub fn drop_rotate_left(mut self) {
        self.root.append(&mut self.back);
        self.root.append(&mut self.front);
    }
}
impl<'a, S> Drop for NodeRange<'a, S>
where
    S: SplaySpec,
    S::T: 'a,
{
    fn drop(&mut self) {
        swap(self.root, &mut self.front);
        self.root.append(&mut self.front);
        self.root.append(&mut self.back);
    }
}

pub mod marker {
    use std::marker::PhantomData;

    pub enum Owned {}
    pub enum Dying {}
    pub struct Immut<'a>(PhantomData<&'a ()>);
    pub struct Mut<'a>(PhantomData<&'a mut ()>);
    pub struct DataMut<'a>(PhantomData<&'a mut ()>);
}
