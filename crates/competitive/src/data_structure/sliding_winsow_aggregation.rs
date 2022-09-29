use super::Monoid;
use std::{
    collections::VecDeque,
    fmt::{self, Debug, Formatter},
};

pub struct QueueAggregation<M>
where
    M: Monoid,
{
    deque: VecDeque<(M::T, M::T)>,
    mid: usize,
}

impl<M> Clone for QueueAggregation<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            deque: self.deque.clone(),
            mid: self.mid,
        }
    }
}

impl<M> Debug for QueueAggregation<M>
where
    M: Monoid,
    M::T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueueAggregation")
            .field("deque", &self.deque)
            .field("mid", &self.mid)
            .finish()
    }
}

impl<M> Default for QueueAggregation<M>
where
    M: Monoid,
{
    fn default() -> Self {
        Self {
            deque: Default::default(),
            mid: 0,
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
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            deque: VecDeque::with_capacity(capacity),
            mid: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.deque.len()
    }
    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }
    pub fn fold_all(&self) -> M::T {
        match (self.mid > 0, self.mid < self.len()) {
            (true, true) => M::operate(
                &self.deque.front().unwrap().0,
                &self.deque.back().unwrap().0,
            ),
            (true, false) => self.deque.front().unwrap().0.clone(),
            (false, true) => self.deque.back().unwrap().0.clone(),
            (false, false) => M::unit(),
        }
    }
    pub fn first(&self) -> Option<&M::T> {
        self.deque.front().map(|t| &t.1)
    }
    pub fn last(&self) -> Option<&M::T> {
        self.deque.back().map(|t| &t.1)
    }
    pub fn push_first(&mut self, value: M::T) {
        let x = if self.mid > 0 {
            M::operate(&value, &self.deque.front().unwrap().0)
        } else {
            value.clone()
        };
        self.mid += 1;
        self.deque.push_front((x, value));
    }
    pub fn push(&mut self, value: M::T) {
        let x = if self.mid < self.len() {
            M::operate(&self.deque.back().unwrap().0.clone(), &value)
        } else {
            value.clone()
        };
        self.deque.push_back((x, value));
    }
    pub fn pop(&mut self) -> Option<M::T> {
        if self.mid == 0 {
            self.mid = self.len();
            let mut acc = M::unit();
            for (x, y) in self.deque.range_mut(..).rev() {
                acc = M::operate(y, &acc);
                *x = acc.clone();
            }
        }
        if self.mid > 0 {
            self.mid -= 1;
        }
        self.deque.pop_front().map(|t| t.1)
    }
}

pub struct DequeAggregation<M>
where
    M: Monoid,
{
    deque: VecDeque<(M::T, M::T)>,
    mid: usize,
}

impl<M> Clone for DequeAggregation<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            deque: self.deque.clone(),
            mid: self.mid,
        }
    }
}

impl<M> Debug for DequeAggregation<M>
where
    M: Monoid,
    M::T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DequeAggregation")
            .field("deque", &self.deque)
            .field("mid", &self.mid)
            .finish()
    }
}

impl<M> Default for DequeAggregation<M>
where
    M: Monoid,
{
    fn default() -> Self {
        Self {
            deque: Default::default(),
            mid: 0,
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
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            deque: VecDeque::with_capacity(capacity),
            mid: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.deque.len()
    }
    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }
    pub fn fold_all(&self) -> M::T {
        match (self.mid > 0, self.mid < self.len()) {
            (true, true) => M::operate(
                &self.deque.front().unwrap().0,
                &self.deque.back().unwrap().0,
            ),
            (true, false) => self.deque.front().unwrap().0.clone(),
            (false, true) => self.deque.back().unwrap().0.clone(),
            (false, false) => M::unit(),
        }
    }
    pub fn front(&self) -> Option<&M::T> {
        self.deque.front().map(|t| &t.1)
    }
    pub fn back(&self) -> Option<&M::T> {
        self.deque.back().map(|t| &t.1)
    }
    pub fn push_front(&mut self, value: M::T) {
        let x = if self.mid > 0 {
            M::operate(&value, &self.deque.front().unwrap().0)
        } else {
            value.clone()
        };
        self.mid += 1;
        self.deque.push_front((x, value));
    }
    pub fn push_back(&mut self, value: M::T) {
        let x = if self.mid < self.len() {
            M::operate(&self.deque.back().unwrap().0.clone(), &value)
        } else {
            value.clone()
        };
        self.deque.push_back((x, value));
    }
    fn rebuild(&mut self) {
        let mut acc = M::unit();
        for (x, y) in self.deque.range_mut(self.mid..) {
            acc = M::operate(&acc, y);
            *x = acc.clone();
        }
        let mut acc = M::unit();
        for (x, y) in self.deque.range_mut(..self.mid).rev() {
            acc = M::operate(y, &acc);
            *x = acc.clone();
        }
    }
    pub fn pop_front(&mut self) -> Option<M::T> {
        if self.mid == 0 {
            self.mid = (self.len() + 1) / 2;
            self.rebuild();
        }
        if self.mid > 0 {
            self.mid -= 1;
        }
        self.deque.pop_front().map(|t| t.1)
    }
    pub fn pop_back(&mut self) -> Option<M::T> {
        if self.mid == self.len() {
            self.mid = self.len() / 2;
            self.rebuild();
        }
        self.deque.pop_back().map(|t| t.1)
    }
}
