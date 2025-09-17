use super::{Comparator, comparator::Less};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
    mem::swap,
    ops::{Deref, DerefMut},
};

#[derive(Clone)]
struct Node<T> {
    value: T,
    first_child: Option<Box<Node<T>>>,
    next_sibling: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            first_child: None,
            next_sibling: None,
        }
    }
}

impl<T> Debug for Node<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .field("first_child", &self.first_child)
            .field("next_sibling", &self.next_sibling)
            .finish()
    }
}

#[derive(Clone)]
pub struct PairingHeap<T, C = Less> {
    root: Option<Box<Node<T>>>,
    len: usize,
    cmp: C,
}

impl<T, C> PairingHeap<T, C>
where
    C: Comparator<T>,
{
    pub fn with_comparator(cmp: C) -> Self {
        Self {
            root: None,
            len: 0,
            cmp,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn peek(&self) -> Option<&T> {
        self.root.as_ref().map(|node| &node.value)
    }

    pub fn push(&mut self, value: T) {
        let node = Box::new(Node::new(value));
        let root = self.root.take();
        self.root = self.merge_option(root, Some(node));
        self.len += 1;
    }

    pub fn append(&mut self, other: &mut Self) {
        if other.is_empty() {
            return;
        }

        let left = self.root.take();
        self.root = self.merge_option(left, other.root.take());
        self.len += other.len;
        other.len = 0;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.root.take().map(|mut root| {
            self.len -= 1;
            let children = root.first_child.take();
            self.root = self.merge_pairs(children);
            root.value
        })
    }

    pub fn peek_mut(&mut self) -> Option<PeekMut<'_, T, C>> {
        let mut root = self.root.take()?;
        let children = root.first_child.take();
        debug_assert!(root.next_sibling.is_none());
        root.next_sibling = None;
        self.root = self.merge_pairs(children);
        Some(PeekMut {
            heap: self,
            node: Some(root),
        })
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    pub fn into_sorted_vec(mut self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        while let Some(value) = self.pop() {
            result.push(value);
        }
        result
    }

    fn merge_option(
        &mut self,
        a: Option<Box<Node<T>>>,
        b: Option<Box<Node<T>>>,
    ) -> Option<Box<Node<T>>> {
        match (a, b) {
            (None, None) => None,
            (Some(node), None) | (None, Some(node)) => Some(node),
            (Some(mut a), Some(mut b)) => {
                if self.cmp.compare(&a.value, &b.value) == Ordering::Greater {
                    swap(&mut a, &mut b);
                }
                b.next_sibling = a.first_child.take();
                a.first_child = Some(b);
                Some(a)
            }
        }
    }

    fn merge_pairs(&mut self, mut head: Option<Box<Node<T>>>) -> Option<Box<Node<T>>> {
        let mut pairs: Vec<Box<Node<T>>> = Vec::new();
        while let Some(mut first) = head {
            let next = first.next_sibling.take();
            if let Some(mut second) = next {
                head = second.next_sibling.take();
                pairs.push(self.merge_option(Some(first), Some(second)).unwrap());
            } else {
                pairs.push(first);
                break;
            }
        }

        let mut result = None;
        while let Some(node) = pairs.pop() {
            result = self.merge_option(Some(node), result);
        }
        result
    }
}

impl<T, C> Default for PairingHeap<T, C>
where
    C: Comparator<T> + Default,
{
    fn default() -> Self {
        Self::with_comparator(C::default())
    }
}

impl<T> PairingHeap<T, Less>
where
    T: Ord,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, C> Debug for PairingHeap<T, C>
where
    T: Debug,
    C: Debug + Comparator<T>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PairingHeap")
            .field("len", &self.len)
            .field("root", &self.root)
            .field("cmp", &self.cmp)
            .finish()
    }
}

impl<T, C> Extend<T> for PairingHeap<T, C>
where
    C: Comparator<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }
}

impl<T, C> FromIterator<T> for PairingHeap<T, C>
where
    C: Comparator<T> + Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::default();
        heap.extend(iter);
        heap
    }
}

pub struct PeekMut<'a, T, C = Less>
where
    C: Comparator<T>,
{
    heap: &'a mut PairingHeap<T, C>,
    node: Option<Box<Node<T>>>,
}

impl<'a, T, C> PeekMut<'a, T, C>
where
    C: Comparator<T>,
{
    pub fn pop(mut this: Self) -> T {
        this.heap.len -= 1;
        let node = this.node.take().expect("PeekMut already consumed");
        let Node { value, .. } = *node;
        value
    }
}

impl<'a, T, C> Deref for PeekMut<'a, T, C>
where
    C: Comparator<T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node.as_ref().expect("PeekMut already consumed").value
    }
}

impl<'a, T, C> DerefMut for PeekMut<'a, T, C>
where
    C: Comparator<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node.as_mut().expect("PeekMut already consumed").value
    }
}

impl<'a, T, C> Drop for PeekMut<'a, T, C>
where
    C: Comparator<T>,
{
    fn drop(&mut self) {
        if let Some(mut node) = self.node.take() {
            debug_assert!(node.next_sibling.is_none());
            let root = self.heap.root.take();
            node.first_child = None;
            self.heap.root = self.heap.merge_option(root, Some(node));
        }
    }
}

pub struct IntoIter<T, C = Less>
where
    C: Comparator<T>,
{
    heap: PairingHeap<T, C>,
}

impl<T, C> IntoIter<T, C>
where
    C: Comparator<T>,
{
    fn new(heap: PairingHeap<T, C>) -> Self {
        Self { heap }
    }
}

impl<T, C> Iterator for IntoIter<T, C>
where
    C: Comparator<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.heap.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.heap.len();
        (len, Some(len))
    }
}

impl<T, C> ExactSizeIterator for IntoIter<T, C>
where
    C: Comparator<T>,
{
    fn len(&self) -> usize {
        self.heap.len()
    }
}

impl<T, C> FusedIterator for IntoIter<T, C> where C: Comparator<T> {}

impl<T, C> IntoIterator for PairingHeap<T, C>
where
    C: Comparator<T>,
{
    type Item = T;
    type IntoIter = IntoIter<T, C>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Xorshift, comparator::Greater};
    use std::{cmp::Reverse, collections::BinaryHeap};

    #[test]
    fn test_min_heap() {
        let mut heap = PairingHeap::<i32>::default();
        assert!(heap.is_empty());
        heap.push(3);
        heap.push(1);
        heap.push(4);
        heap.push(2);
        assert_eq!(heap.len(), 4);
        assert_eq!(heap.peek(), Some(&1));
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(4));
        assert!(heap.is_empty());
    }

    #[test]
    fn test_max_heap() {
        let mut heap = PairingHeap::with_comparator(Greater);
        heap.extend([3, 1, 4, 2]);
        assert_eq!(heap.peek(), Some(&4));
        assert_eq!(heap.pop(), Some(4));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(1));
        assert!(heap.is_empty());
    }

    #[test]
    fn test_against_binary_heap() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let mut heap = PairingHeap::<i64>::default();
            let mut reference = BinaryHeap::new();
            let mut other = PairingHeap::<i64>::default();
            let mut reference_other = BinaryHeap::new();
            for _ in 0..2000 {
                match rng.rand(7) {
                    0 => {
                        let value = rng.random(..);
                        heap.push(value);
                        reference.push(Reverse(value));
                    }
                    1 => {
                        assert_eq!(heap.pop(), reference.pop().map(|Reverse(x)| x));
                    }
                    2 => {
                        let value = rng.random(..);
                        other.push(value);
                        reference_other.push(Reverse(value));
                    }
                    3 => {
                        heap.append(&mut other);
                        reference.append(&mut reference_other);
                    }
                    4 => {
                        if let Some(mut guard) = heap.peek_mut() {
                            let new_value = rng.random(..);
                            {
                                let mut reference_guard = reference
                                    .peek_mut()
                                    .expect("reference heap empty while pairing heap not");
                                reference_guard.0 = new_value;
                            }
                            *guard = new_value;
                        } else {
                            assert!(reference.is_empty());
                        }
                    }
                    5 => {
                        if let Some(mut guard) = other.peek_mut() {
                            let new_value = rng.random(..);
                            {
                                let mut reference_guard = reference_other
                                    .peek_mut()
                                    .expect("reference heap empty while pairing heap not");
                                reference_guard.0 = new_value;
                            }
                            *guard = new_value;
                        } else {
                            assert!(reference_other.is_empty());
                        }
                    }
                    _ => {
                        assert_eq!(other.pop(), reference_other.pop().map(|Reverse(x)| x));
                    }
                }
                assert_eq!(heap.peek(), reference.peek().map(|x| &x.0));
                assert_eq!(other.peek(), reference_other.peek().map(|x| &x.0));
                assert_eq!(heap.len(), reference.len());
                assert_eq!(other.len(), reference_other.len());
            }
            heap.append(&mut other);
            reference.append(&mut reference_other);
            while let Some(Reverse(value)) = reference.pop() {
                assert_eq!(heap.pop(), Some(value));
            }
            assert!(heap.is_empty());
            assert!(other.is_empty());
            assert!(reference.is_empty());
            assert!(reference_other.is_empty());
        }
    }
}
