use super::Monoid;
use std::fmt::{self, Debug, Formatter};

pub struct QueueAggregation<M>
where
    M: Monoid,
{
    front_stack: Vec<(M::T, M::T)>,
    back_stack: Vec<(M::T, M::T)>,
}

impl<M> Clone for QueueAggregation<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            front_stack: self.front_stack.clone(),
            back_stack: self.back_stack.clone(),
        }
    }
}

impl<M> Debug for QueueAggregation<M>
where
    M: Monoid<T: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueueAggregation")
            .field("front_stack", &self.front_stack)
            .field("back_stack", &self.back_stack)
            .finish()
    }
}

impl<M> Default for QueueAggregation<M>
where
    M: Monoid,
{
    fn default() -> Self {
        Self {
            front_stack: Vec::new(),
            back_stack: Vec::new(),
        }
    }
}

impl<M> QueueAggregation<M>
where
    M: Monoid,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.front_stack.len() + self.back_stack.len()
    }
    pub fn is_empty(&self) -> bool {
        self.front_stack.is_empty() && self.back_stack.is_empty()
    }
    pub fn fold_all(&self) -> M::T {
        M::operate(
            self.front_stack.last().map(|t| &t.0).unwrap_or(&M::unit()),
            self.back_stack.last().map(|t| &t.0).unwrap_or(&M::unit()),
        )
    }
    pub fn last(&self) -> Option<&M::T> {
        self.back_stack
            .last()
            .or_else(|| self.front_stack.first())
            .map(|t| &t.1)
    }
    pub fn push(&mut self, value: M::T) {
        let x = M::operate(
            self.back_stack.last().map(|t| &t.0).unwrap_or(&M::unit()),
            &value,
        );
        self.back_stack.push((x, value));
    }
    fn push_front(&mut self, value: M::T) {
        let x = M::operate(
            &value,
            self.front_stack.last().map(|t| &t.0).unwrap_or(&M::unit()),
        );
        self.front_stack.push((x, value));
    }
    pub fn pop(&mut self) -> Option<M::T> {
        if self.front_stack.is_empty() {
            let mut back_stack = std::mem::take(&mut self.back_stack);
            for x in back_stack.drain(..).map(|t| t.1).rev() {
                self.push_front(x);
            }
            self.back_stack = back_stack;
        }
        self.front_stack.pop().map(|t| t.1)
    }
}

pub struct DequeAggregation<M>
where
    M: Monoid,
{
    front_stack: Vec<(M::T, M::T)>,
    back_stack: Vec<(M::T, M::T)>,
}

impl<M> Clone for DequeAggregation<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            front_stack: self.front_stack.clone(),
            back_stack: self.back_stack.clone(),
        }
    }
}

impl<M> Debug for DequeAggregation<M>
where
    M: Monoid<T: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DequeAggregation")
            .field("front_stack", &self.front_stack)
            .field("back_stack", &self.back_stack)
            .finish()
    }
}

impl<M> Default for DequeAggregation<M>
where
    M: Monoid,
{
    fn default() -> Self {
        Self {
            front_stack: Vec::new(),
            back_stack: Vec::new(),
        }
    }
}

impl<M> DequeAggregation<M>
where
    M: Monoid,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.front_stack.len() + self.back_stack.len()
    }
    pub fn is_empty(&self) -> bool {
        self.front_stack.is_empty() && self.back_stack.is_empty()
    }
    pub fn fold_all(&self) -> M::T {
        M::operate(
            self.front_stack.last().map(|t| &t.0).unwrap_or(&M::unit()),
            self.back_stack.last().map(|t| &t.0).unwrap_or(&M::unit()),
        )
    }
    pub fn front(&self) -> Option<&M::T> {
        self.front_stack
            .last()
            .or_else(|| self.back_stack.first())
            .map(|t| &t.1)
    }
    pub fn back(&self) -> Option<&M::T> {
        self.back_stack
            .last()
            .or_else(|| self.front_stack.first())
            .map(|t| &t.1)
    }
    pub fn push_front(&mut self, value: M::T) {
        let x = M::operate(
            &value,
            self.front_stack.last().map(|t| &t.0).unwrap_or(&M::unit()),
        );
        self.front_stack.push((x, value));
    }
    pub fn push_back(&mut self, value: M::T) {
        let x = M::operate(
            self.back_stack.last().map(|t| &t.0).unwrap_or(&M::unit()),
            &value,
        );
        self.back_stack.push((x, value));
    }
    pub fn pop_front(&mut self) -> Option<M::T> {
        if self.front_stack.is_empty() {
            let n = self.back_stack.len();
            let mut back_stack = std::mem::take(&mut self.back_stack);
            for x in back_stack.drain(..n.div_ceil(2)).map(|t| t.1).rev() {
                self.push_front(x);
            }
            for x in back_stack.drain(..).map(|t| t.1) {
                self.push_back(x);
            }
        }
        self.front_stack.pop().map(|t| t.1)
    }
    pub fn pop_back(&mut self) -> Option<M::T> {
        if self.back_stack.is_empty() {
            let n = self.front_stack.len();
            let mut front_stack = std::mem::take(&mut self.front_stack);
            for x in front_stack.drain(..n.div_ceil(2)).map(|t| t.1).rev() {
                self.push_back(x);
            }
            for x in front_stack.drain(..).map(|t| t.1) {
                self.push_front(x);
            }
        }
        self.back_stack.pop().map(|t| t.1)
    }
}

#[cfg(kani)]
mod verification {
    use super::*;
    use crate::algebra::LogicalLinearOperation;

    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_queue_transfer() {
        let values: [(u8, u8); 4] = kani::any();
        let pops: usize = kani::any();
        let input: u8 = kani::any();
        kani::assume(pops <= 3);
        let mut queue = QueueAggregation::<LogicalLinearOperation<u8>>::new();
        kani::assert(
            queue.pop().is_none(),
            "An empty QueueAggregation returns None",
        );
        for &value in &values[..3] {
            queue.push(value);
        }
        for &value in &values[..pops] {
            kani::assert(
                queue.pop() == Some(value),
                "For any three u8 bitwise affine maps and 0..=3 removals, QueueAggregation preserves FIFO order through transfer to the front stack",
            );
        }
        queue.push(values[3]);
        kani::cover!(!queue.front_stack.is_empty() && !queue.back_stack.is_empty());
        kani::assert(
            queue.pop() == Some(values[pops]),
            "Appending a fourth arbitrary map preserves the oldest remaining value, including when the queue was emptied",
        );
        let mut output = input;
        for &(mask, xor) in &values[pops + 1..] {
            output = (output & mask) ^ xor;
        }
        kani::assert(
            queue.len() == 3 - pops
                && queue.is_empty() == (pops == 3)
                && queue.last() == (pops < 3).then_some(&values[3])
                && LogicalLinearOperation::eval(&queue.fold_all(), &input) == output,
            "After the bounded transfer/refill sequence, queue length, last value and noncommutative aggregate match the remaining FIFO values",
        );
        kani::cover!(pops == 0);
        kani::cover!(pops == 3);
    }

    macro_rules! proof_deque_rebalance {
        ($name:ident, $len:expr, $push:ident, $pop:ident, $initial_front:expr) => {
            #[kani::proof]
            #[kani::unwind(6)]
            fn $name() {
                let mut values: [(u8, u8); $len] = kani::any();
                let added: (u8, u8) = kani::any();
                let push_front: bool = kani::any();
                let input: u8 = kani::any();
                let mut deque = DequeAggregation::<LogicalLinearOperation<u8>>::new();
                kani::assert(
                    deque.pop_front().is_none() && deque.pop_back().is_none(),
                    "Both removal directions return None on an empty DequeAggregation",
                );
                for &value in &values {
                    deque.$push(value);
                }
                if $initial_front {
                    values.reverse();
                }
                let (left, right) = if $initial_front {
                    (0, $len - 1)
                } else {
                    (1, $len)
                };
                kani::assert(
                    deque.$pop() == Some(values[if $initial_front { $len - 1 } else { 0 }]),
                    concat!(
                        "For all ", stringify!($len),
                        " u8 bitwise affine maps inserted with ", stringify!($push),
                        ", ", stringify!($pop), " rebalances and returns the opposite endpoint"
                    ),
                );
                if push_front {
                    deque.push_front(added);
                } else {
                    deque.push_back(added);
                }
                let mut output = input;
                if push_front {
                    output = (output & added.0) ^ added.1;
                }
                for &(mask, xor) in &values[left..right] {
                    output = (output & mask) ^ xor;
                }
                if !push_front {
                    output = (output & added.0) ^ added.1;
                }
                kani::assert(
                    deque.len() == $len
                        && !deque.is_empty()
                        && deque.front() == Some(if push_front { &added } else { &values[left] })
                        && deque.back() == Some(if push_front { &values[right - 1] } else { &added })
                        && LogicalLinearOperation::eval(&deque.fold_all(), &input) == output,
                    concat!(
                        "After an endpoint removal and insertion from a ", stringify!($len),
                        "-element deque, length, endpoints and noncommutative aggregate match the logical order"
                    ),
                );
                kani::cover!(!deque.front_stack.is_empty() && !deque.back_stack.is_empty());
                kani::cover!(
                    ((input & values[0].0) ^ values[0].1) & values[1].0 ^ values[1].1
                        != ((input & values[1].0) ^ values[1].1) & values[0].0 ^ values[0].1
                );
            }
        };
    }

    // Odd and even lengths exercise both div_ceil rebalancing boundaries.
    proof_deque_rebalance!(proof_deque_front_three, 3, push_back, pop_front, false);
    proof_deque_rebalance!(proof_deque_front_four, 4, push_back, pop_front, false);
    proof_deque_rebalance!(proof_deque_back_three, 3, push_front, pop_back, true);
    proof_deque_rebalance!(proof_deque_back_four, 4, push_front, pop_back, true);
}
