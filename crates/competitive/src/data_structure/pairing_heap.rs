use super::{Comparator, EmptyAct, MonoidAct, Unital, comparator::Less};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
    mem::{replace, swap},
    ops::{Deref, DerefMut},
};

#[derive(Clone)]
struct Node<T, A>
where
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    value: T,
    first_child: Option<Box<Node<T, A>>>,
    next_sibling: Option<Box<Node<T, A>>>,
    lazy: A::Act,
}

impl<T, A> Node<T, A>
where
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    fn new(value: T) -> Self {
        Self {
            value,
            first_child: None,
            next_sibling: None,
            lazy: A::unit(),
        }
    }

    fn apply(&mut self, act: &A::Act) {
        A::act_assign(&mut self.value, act);
        A::operate_assign(&mut self.lazy, act);
    }

    fn propagate(&mut self) {
        if !<A::ActMonoid as Unital>::is_unit(&self.lazy) {
            let act = replace(&mut self.lazy, A::unit());
            if let Some(node) = self.first_child.as_mut() {
                node.apply(&act);
            }
            if let Some(node) = self.next_sibling.as_mut() {
                node.apply(&act);
            }
        }
    }
}

impl<T, A> Debug for Node<T, A>
where
    T: Debug,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
    A::Act: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .field("first_child", &self.first_child)
            .field("next_sibling", &self.next_sibling)
            .field("lazy", &self.lazy)
            .finish()
    }
}

#[derive(Clone)]
pub struct PairingHeap<T, C = Less, A = EmptyAct<T>>
where
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    root: Option<Box<Node<T, A>>>,
    len: usize,
    cmp: C,
}

impl<T, C, A> PairingHeap<T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
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
            root.propagate();
            let children = root.first_child.take();
            self.root = self.merge_pairs(children);
            root.value
        })
    }

    pub fn peek_mut(&mut self) -> Option<PeekMut<'_, T, C, A>> {
        let mut root = self.root.take()?;
        root.propagate();
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

    pub fn apply_all(&mut self, act: A::Act) {
        if let Some(root) = self.root.as_mut() {
            root.apply(&act);
        }
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
        a: Option<Box<Node<T, A>>>,
        b: Option<Box<Node<T, A>>>,
    ) -> Option<Box<Node<T, A>>> {
        match (a, b) {
            (None, None) => None,
            (Some(node), None) | (None, Some(node)) => Some(node),
            (Some(mut a), Some(mut b)) => {
                a.propagate();
                b.propagate();
                if self.cmp.compare(&a.value, &b.value) == Ordering::Greater {
                    swap(&mut a, &mut b);
                }
                b.next_sibling = a.first_child.take();
                a.first_child = Some(b);
                Some(a)
            }
        }
    }

    fn merge_pairs(&mut self, mut head: Option<Box<Node<T, A>>>) -> Option<Box<Node<T, A>>> {
        let mut pairs: Vec<Box<Node<T, A>>> = Vec::new();
        while let Some(mut first) = head {
            first.propagate();
            let next = first.next_sibling.take();
            if let Some(mut second) = next {
                second.propagate();
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

impl<T, C, A> Default for PairingHeap<T, C, A>
where
    C: Comparator<T> + Default,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    fn default() -> Self {
        Self::with_comparator(C::default())
    }
}

impl<T, A> PairingHeap<T, Less, A>
where
    T: Ord,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, C, A> Debug for PairingHeap<T, C, A>
where
    T: Debug,
    C: Debug + Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
    A::Act: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PairingHeap")
            .field("len", &self.len)
            .field("root", &self.root)
            .field("cmp", &self.cmp)
            .finish()
    }
}

impl<T, C, A> Extend<T> for PairingHeap<T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }
}

impl<T, C, A> FromIterator<T> for PairingHeap<T, C, A>
where
    C: Comparator<T> + Default,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::default();
        heap.extend(iter);
        heap
    }
}

pub struct PeekMut<'a, T, C = Less, A = EmptyAct<T>>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    heap: &'a mut PairingHeap<T, C, A>,
    node: Option<Box<Node<T, A>>>,
}

impl<'a, T, C, A> PeekMut<'a, T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    pub fn pop(mut this: Self) -> T {
        this.heap.len -= 1;
        let node = this.node.take().expect("PeekMut already consumed");
        let Node { value, .. } = *node;
        value
    }
}

impl<'a, T, C, A> Deref for PeekMut<'a, T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node.as_ref().expect("PeekMut already consumed").value
    }
}

impl<'a, T, C, A> DerefMut for PeekMut<'a, T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node.as_mut().expect("PeekMut already consumed").value
    }
}

impl<'a, T, C, A> Drop for PeekMut<'a, T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
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

pub struct IntoIter<T, C = Less, A = EmptyAct<T>>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    heap: PairingHeap<T, C, A>,
}

impl<T, C, A> IntoIter<T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    fn new(heap: PairingHeap<T, C, A>) -> Self {
        Self { heap }
    }
}

impl<T, C, A> Iterator for IntoIter<T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
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

impl<T, C, A> ExactSizeIterator for IntoIter<T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    fn len(&self) -> usize {
        self.heap.len()
    }
}

impl<T, C, A> FusedIterator for IntoIter<T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
}

impl<T, C, A> IntoIterator for PairingHeap<T, C, A>
where
    C: Comparator<T>,
    A: MonoidAct<Key = T>,
    A::Act: PartialEq,
{
    type Item = T;
    type IntoIter = IntoIter<T, C, A>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, FlattenAct},
        tools::{Xorshift, comparator::Greater},
    };
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
        let mut heap = PairingHeap::<i32, Greater>::with_comparator(Greater);
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
            type Heap = PairingHeap<i64, Less, FlattenAct<AdditiveOperation<i64>>>;
            let mut heap = Heap::default();
            let mut reference = BinaryHeap::new();
            let mut heap_offset = 0i64;
            let mut other = Heap::default();
            let mut reference_other = BinaryHeap::new();
            let mut other_offset = 0i64;
            for _ in 0..2000 {
                match rng.rand(9) {
                    0 => {
                        let value: i64 = rng.random(-1_000_000..=1_000_000);
                        heap.push(value);
                        reference.push(Reverse(value - heap_offset));
                    }
                    1 => {
                        assert_eq!(
                            heap.pop(),
                            reference.pop().map(|Reverse(x)| x + heap_offset)
                        );
                    }
                    2 => {
                        let value: i64 = rng.random(-1_000_000..=1_000_000);
                        other.push(value);
                        reference_other.push(Reverse(value - other_offset));
                    }
                    3 => {
                        heap.append(&mut other);
                        while let Some(Reverse(value)) = reference_other.pop() {
                            reference.push(Reverse(value + other_offset - heap_offset));
                        }
                    }
                    4 => {
                        if let Some(mut guard) = heap.peek_mut() {
                            let new_value: i64 = rng.random(-1_000_000..=1_000_000);
                            {
                                let mut reference_guard = reference
                                    .peek_mut()
                                    .expect("reference heap empty while pairing heap not");
                                reference_guard.0 = new_value - heap_offset;
                            }
                            *guard = new_value;
                        } else {
                            assert!(reference.is_empty());
                        }
                    }
                    5 => {
                        if let Some(mut guard) = other.peek_mut() {
                            let new_value: i64 = rng.random(-1_000_000..=1_000_000);
                            {
                                let mut reference_guard = reference_other
                                    .peek_mut()
                                    .expect("reference heap empty while pairing heap not");
                                reference_guard.0 = new_value - other_offset;
                            }
                            *guard = new_value;
                        } else {
                            assert!(reference_other.is_empty());
                        }
                    }
                    6 => {
                        let add: i64 = rng.random(-1_000..=1_000);
                        heap.apply_all(add);
                        if !reference.is_empty() {
                            heap_offset += add;
                        }
                    }
                    7 => {
                        let add: i64 = rng.random(-1_000..=1_000);
                        other.apply_all(add);
                        if !reference_other.is_empty() {
                            other_offset += add;
                        }
                    }
                    _ => {
                        assert_eq!(
                            other.pop(),
                            reference_other.pop().map(|Reverse(x)| x + other_offset)
                        );
                    }
                }
                assert_eq!(
                    heap.peek().copied(),
                    reference.peek().map(|x| x.0 + heap_offset)
                );
                assert_eq!(
                    other.peek().copied(),
                    reference_other.peek().map(|x| x.0 + other_offset)
                );
                assert_eq!(heap.len(), reference.len());
                assert_eq!(other.len(), reference_other.len());
            }
            heap.append(&mut other);
            while let Some(Reverse(value)) = reference_other.pop() {
                reference.push(Reverse(value + other_offset - heap_offset));
            }
            while let Some(Reverse(value)) = reference.pop() {
                assert_eq!(heap.pop(), Some(value + heap_offset));
            }
            assert!(heap.is_empty());
            assert!(other.is_empty());
            assert!(reference.is_empty());
            assert!(reference_other.is_empty());
        }
    }
}
