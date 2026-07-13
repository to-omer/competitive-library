use super::binary_search_tree::{
    BstDataMutRef, BstNodePtr, BstRoot, BstSeeker, BstSpec, EqualSide,
    node::{WithNoParent, WithParent},
    seeker::SeekRight,
};
use std::{cmp::Ordering, mem::MaybeUninit};

pub mod with_parent {
    use super::{BstDataMutRef, BstNodePtr, BstSpec, MaybeUninit, WithParent};

    type NodePtr<Spec> = BstNodePtr<<Spec as BstSpec>::Data, <Spec as BstSpec>::Parent>;

    #[inline]
    unsafe fn internal_parent<Spec, Data>(
        node: NodePtr<Spec>,
    ) -> Result<(NodePtr<Spec>, usize), Option<NodePtr<Spec>>>
    where
        Spec: BstSpec<Data = Data, Parent = WithParent<Data>>,
    {
        let Some(parent) = (unsafe { node.as_ref().parent.parent }) else {
            return Err(None);
        };
        let children = unsafe { parent.as_ref().child };
        if children[0] == Some(node) {
            Ok((parent, 0))
        } else if children[1] == Some(node) {
            Ok((parent, 1))
        } else {
            Err(Some(parent))
        }
    }

    #[inline(always)]
    unsafe fn rotate<Spec, Data>(mut node: NodePtr<Spec>)
    where
        Spec: BstSpec<Data = Data, Parent = WithParent<Data>>,
    {
        let (mut parent, direction) = unsafe { internal_parent::<Spec, Data>(node) }
            .expect("an auxiliary root cannot be rotated");
        let (grandparent, parent_direction) = match unsafe { internal_parent::<Spec, Data>(parent) }
        {
            Ok((grandparent, direction)) => (Some(grandparent), Some(direction)),
            Err(grandparent) => (grandparent, None),
        };
        let middle = unsafe { node.as_mut().child[direction ^ 1].take() };

        unsafe {
            parent.as_mut().child[direction] = middle;
            if let Some(mut middle) = middle {
                middle.as_mut().parent.parent = Some(parent);
            }
            node.as_mut().child[direction ^ 1] = Some(parent);
            node.as_mut().parent.parent = grandparent;
            parent.as_mut().parent.parent = Some(node);
            if let (Some(mut grandparent), Some(parent_direction)) = (grandparent, parent_direction)
            {
                grandparent.as_mut().child[parent_direction] = Some(node);
            }
            Spec::bottom_up(BstDataMutRef::new_unchecked(parent));
        }
    }

    /// Moves `node` to the root of its auxiliary tree and returns the previous root.
    ///
    /// # Safety
    ///
    /// `node` and every pointer reachable through its auxiliary-parent chain must
    /// refer to live nodes of the same tree.
    #[inline(always)]
    pub unsafe fn splay<Spec, Data>(node: NodePtr<Spec>) -> NodePtr<Spec>
    where
        Spec: BstSpec<Data = Data, Parent = WithParent<Data>>,
    {
        let mut inline_stack = [const { MaybeUninit::uninit() }; 64];
        let mut inline_len = 0;
        let mut overflow_stack = Vec::new();
        let mut current = node;
        loop {
            if inline_len < inline_stack.len() {
                inline_stack[inline_len].write(current);
                inline_len += 1;
            } else {
                overflow_stack.push(current);
            }
            match unsafe { internal_parent::<Spec, Data>(current) } {
                Ok((parent, _)) => current = parent,
                Err(_) => break,
            }
        }
        for &node in overflow_stack.iter().rev() {
            unsafe { Spec::top_down(BstDataMutRef::new_unchecked(node)) };
        }
        while inline_len > 0 {
            inline_len -= 1;
            unsafe {
                Spec::top_down(BstDataMutRef::new_unchecked(
                    *inline_stack[inline_len].assume_init_ref(),
                ));
            }
        }

        while let Ok((parent, node_direction)) = unsafe { internal_parent::<Spec, Data>(node) } {
            if let Ok((_, parent_direction)) = unsafe { internal_parent::<Spec, Data>(parent) } {
                if node_direction == parent_direction {
                    unsafe { rotate::<Spec, Data>(parent) };
                } else {
                    unsafe { rotate::<Spec, Data>(node) };
                }
            }
            unsafe { rotate::<Spec, Data>(node) };
        }
        unsafe { Spec::bottom_up(BstDataMutRef::new_unchecked(node)) };
        current
    }

    /// Moves `node` to the root by propagating only the nodes involved in each rotation and
    /// returns the previous root.
    ///
    /// # Safety
    ///
    /// `node` and every pointer reachable through its auxiliary-parent chain must refer to live
    /// nodes of the same tree. Propagating an ancestor after its descendant must be valid for
    /// `Spec`.
    #[inline(always)]
    pub unsafe fn splay_with_local_top_down<Spec, Data>(node: NodePtr<Spec>) -> NodePtr<Spec>
    where
        Spec: BstSpec<Data = Data, Parent = WithParent<Data>>,
    {
        let mut current = node;
        unsafe { Spec::top_down(BstDataMutRef::new_unchecked(node)) };
        while let Ok((parent, node_direction)) = unsafe { internal_parent::<Spec, Data>(node) } {
            match unsafe { internal_parent::<Spec, Data>(parent) } {
                Ok((grandparent, parent_direction)) => {
                    current = grandparent;
                    unsafe {
                        Spec::top_down(BstDataMutRef::new_unchecked(grandparent));
                        Spec::top_down(BstDataMutRef::new_unchecked(parent));
                        Spec::top_down(BstDataMutRef::new_unchecked(node));
                    }
                    if node_direction == parent_direction {
                        unsafe { rotate::<Spec, Data>(parent) };
                    } else {
                        unsafe { rotate::<Spec, Data>(node) };
                    }
                }
                Err(_) => {
                    current = parent;
                    unsafe {
                        Spec::top_down(BstDataMutRef::new_unchecked(parent));
                        Spec::top_down(BstDataMutRef::new_unchecked(node));
                    }
                }
            }
            unsafe { rotate::<Spec, Data>(node) };
        }
        unsafe { Spec::bottom_up(BstDataMutRef::new_unchecked(node)) };
        current
    }
}

pub fn rooted_heavy_order(
    vertices_size: usize,
    edges: &[(usize, usize)],
) -> Vec<(usize, usize, bool)> {
    if vertices_size == 0 {
        return Vec::new();
    }
    let mut head = vec![usize::MAX; vertices_size];
    let mut to = Vec::with_capacity(edges.len() * 2);
    let mut next = Vec::with_capacity(edges.len() * 2);
    for &(u, v) in edges {
        to.push(v);
        next.push(head[u]);
        head[u] = to.len() - 1;
        to.push(u);
        next.push(head[v]);
        head[v] = to.len() - 1;
    }
    let mut parent = vec![usize::MAX; vertices_size];
    let mut stack = vec![0];
    let mut order = Vec::with_capacity(vertices_size - 1);
    parent[0] = 0;
    while let Some(u) = stack.pop() {
        let mut edge = head[u];
        while edge != usize::MAX {
            let v = to[edge];
            if parent[v] == usize::MAX {
                parent[v] = u;
                order.push((v, u));
                stack.push(v);
            }
            edge = next[edge];
        }
    }
    let mut size = vec![1usize; vertices_size];
    let mut heavy = vec![usize::MAX; vertices_size];
    for &(child, parent) in order.iter().rev() {
        size[parent] += size[child];
        if heavy[parent] == usize::MAX || size[heavy[parent]] < size[child] {
            heavy[parent] = child;
        }
    }
    order
        .into_iter()
        .map(|(child, parent)| (child, parent, heavy[parent] == child))
        .collect()
}

#[inline]
pub fn splay<Spec, Data, Seeker>(
    root: BstRoot<Spec>,
    mut seeker: Seeker,
) -> (Ordering, BstRoot<Spec>)
where
    Spec: BstSpec<Data = Data, Parent = WithNoParent<Data>>,
    Seeker: BstSeeker<Spec = Spec>,
{
    let mut root = root;
    let mut left_subtree = None;
    let mut right_subtree = None;
    let mut left_entry = &mut left_subtree;
    let mut right_entry = &mut right_subtree;
    let mut inline_stack = [None; 24];
    let mut inline_len = 0;
    let mut overflow_stack = vec![];

    macro_rules! push_node {
        ($node:expr) => {
            if inline_len < inline_stack.len() {
                inline_stack[inline_len] = Some($node);
                inline_len += 1;
            } else {
                overflow_stack.push($node);
            }
        };
    }

    macro_rules! add {
        (@left $node:ident) => {
            *left_entry = Some($node.node);
            push_node!($node.node);
            left_entry = unsafe { &mut $node.node.as_mut().child[1] };
        };
        (@right $node:ident) => {
            *right_entry = Some($node.node);
            push_node!($node.node);
            right_entry = unsafe { &mut $node.node.as_mut().child[0] };
        };
    }

    let root_ordering = loop {
        Spec::top_down(root.borrow_datamut());
        match seeker.bst_seek(root.reborrow()) {
            Ordering::Greater => {
                let Some(mut child) = (unsafe { root.borrow_mut().left_mut().take() }) else {
                    break Ordering::Greater;
                };
                Spec::top_down(child.borrow_datamut());
                match seeker.bst_seek(child.reborrow()) {
                    Ordering::Greater => {
                        let Some(mut grandchild) =
                            (unsafe { child.borrow_mut().left_mut().take() })
                        else {
                            add!(@right root);
                            root = child;
                            break Ordering::Greater;
                        };
                        Spec::top_down(grandchild.borrow_datamut());
                        let child_right = unsafe { child.borrow_mut().right_mut().take() };
                        if let Some(child_right) = child_right {
                            unsafe { root.borrow_mut().left_mut().set(child_right) };
                        }
                        Spec::bottom_up(root.borrow_datamut());
                        unsafe { child.borrow_mut().right_mut().set(root) };
                        add!(@right child);
                        root = grandchild;
                    }
                    Ordering::Equal => {
                        add!(@right root);
                        root = child;
                        break Ordering::Equal;
                    }
                    Ordering::Less => {
                        let Some(mut grandchild) =
                            (unsafe { child.borrow_mut().right_mut().take() })
                        else {
                            add!(@right root);
                            root = child;
                            break Ordering::Less;
                        };
                        Spec::top_down(grandchild.borrow_datamut());
                        add!(@right root);
                        add!(@left child);
                        root = grandchild;
                    }
                }
            }
            Ordering::Equal => break Ordering::Equal,
            Ordering::Less => {
                let Some(mut child) = (unsafe { root.borrow_mut().right_mut().take() }) else {
                    break Ordering::Less;
                };
                Spec::top_down(child.borrow_datamut());
                match seeker.bst_seek(child.reborrow()) {
                    Ordering::Greater => {
                        let Some(mut grandchild) =
                            (unsafe { child.borrow_mut().left_mut().take() })
                        else {
                            add!(@left root);
                            root = child;
                            break Ordering::Greater;
                        };
                        Spec::top_down(grandchild.borrow_datamut());
                        add!(@left root);
                        add!(@right child);
                        root = grandchild;
                    }
                    Ordering::Equal => {
                        add!(@left root);
                        root = child;
                        break Ordering::Equal;
                    }
                    Ordering::Less => {
                        let Some(mut grandchild) =
                            (unsafe { child.borrow_mut().right_mut().take() })
                        else {
                            add!(@left root);
                            root = child;
                            break Ordering::Less;
                        };
                        Spec::top_down(grandchild.borrow_datamut());
                        let child_left = unsafe { child.borrow_mut().left_mut().take() };
                        if let Some(child_left) = child_left {
                            unsafe { root.borrow_mut().right_mut().set(child_left) };
                        }
                        Spec::bottom_up(root.borrow_datamut());
                        unsafe { child.borrow_mut().left_mut().set(root) };
                        add!(@left child);
                        root = grandchild;
                    }
                }
            }
        }
    };

    *left_entry = unsafe { root.borrow_mut().left_mut().take() }.map(|node| node.node);
    *right_entry = unsafe { root.borrow_mut().right_mut().take() }.map(|node| node.node);
    unsafe {
        root.node.as_mut().child[0] = left_subtree;
        root.node.as_mut().child[1] = right_subtree;
        while let Some(node) = overflow_stack.pop() {
            Spec::bottom_up(BstRoot::new(node).borrow_datamut());
        }
        while inline_len > 0 {
            inline_len -= 1;
            let node = inline_stack[inline_len].unwrap_unchecked();
            Spec::bottom_up(BstRoot::new(node).borrow_datamut());
        }
    }
    Spec::bottom_up(root.borrow_datamut());
    (root_ordering, root)
}

#[inline]
pub fn merge<Spec, Data>(
    left: Option<BstRoot<Spec>>,
    right: Option<BstRoot<Spec>>,
) -> Option<BstRoot<Spec>>
where
    Spec: BstSpec<Data = Data, Parent = WithNoParent<Data>>,
{
    match (left, right) {
        (None, None) => None,
        (None, Some(root)) | (Some(root), None) => Some(root),
        (Some(left), Some(mut right)) if right.reborrow().left().descend().is_err() => {
            Spec::top_down(right.borrow_datamut());
            unsafe { right.borrow_mut().left_mut().set(left) };
            Spec::bottom_up(right.borrow_datamut());
            Some(right)
        }
        (Some(left), Some(right)) => {
            let (_, mut root) = splay(left, SeekRight::default());
            unsafe { root.borrow_mut().right_mut().set(right) };
            Spec::bottom_up(root.borrow_datamut());
            Some(root)
        }
    }
}

#[inline]
pub fn split<Spec, Data, Seeker>(
    root: Option<BstRoot<Spec>>,
    seeker: Seeker,
    equal_side: EqualSide,
) -> (Option<BstRoot<Spec>>, Option<BstRoot<Spec>>)
where
    Spec: BstSpec<Data = Data, Parent = WithNoParent<Data>>,
    Seeker: BstSeeker<Spec = Spec>,
{
    let Some(root) = root else {
        return (None, None);
    };
    let (ordering, mut root) = splay(root, seeker);
    if equal_side.goes_left(ordering) {
        let right = unsafe { root.borrow_mut().right_mut().take() };
        Spec::bottom_up(root.borrow_datamut());
        (Some(root), right)
    } else {
        let left = unsafe { root.borrow_mut().left_mut().take() };
        Spec::bottom_up(root.borrow_datamut());
        (left, Some(root))
    }
}
