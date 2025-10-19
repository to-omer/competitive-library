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
    M: Monoid,
    M::T: Debug,
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
    M: Monoid,
    M::T: Debug,
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
            for x in back_stack.drain(..(n + 1) / 2).map(|t| t.1).rev() {
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
            for x in front_stack.drain(..(n + 1) / 2).map(|t| t.1).rev() {
                self.push_back(x);
            }
            for x in front_stack.drain(..).map(|t| t.1) {
                self.push_front(x);
            }
        }
        self.back_stack.pop().map(|t| t.1)
    }
}
