use super::binary_search_tree::{
    BstRoot, BstSeeker, BstSpec, node::WithNoParent, seeker::SeekRight,
};
use std::cmp::Ordering;

#[inline]
pub(super) fn splay<Spec, Data, Seeker>(
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
    let mut inline_stack = [None; 64];
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
pub(super) fn merge<Spec, Data>(
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
pub(super) fn split<Spec, Data, Seeker>(
    root: Option<BstRoot<Spec>>,
    seeker: Seeker,
    eq_left: bool,
) -> (Option<BstRoot<Spec>>, Option<BstRoot<Spec>>)
where
    Spec: BstSpec<Data = Data, Parent = WithNoParent<Data>>,
    Seeker: BstSeeker<Spec = Spec>,
{
    let Some(root) = root else {
        return (None, None);
    };
    let (ordering, mut root) = splay(root, seeker);
    match ordering {
        Ordering::Less => {
            let right = unsafe { root.borrow_mut().right_mut().take() };
            Spec::bottom_up(root.borrow_datamut());
            (Some(root), right)
        }
        Ordering::Greater => {
            let left = unsafe { root.borrow_mut().left_mut().take() };
            Spec::bottom_up(root.borrow_datamut());
            (left, Some(root))
        }
        Ordering::Equal if eq_left => {
            let right = unsafe { root.borrow_mut().right_mut().take() };
            Spec::bottom_up(root.borrow_datamut());
            (Some(root), right)
        }
        Ordering::Equal => {
            let left = unsafe { root.borrow_mut().left_mut().take() };
            Spec::bottom_up(root.borrow_datamut());
            (left, Some(root))
        }
    }
}
