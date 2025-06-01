use super::{Rational, Signed};
use std::mem::swap;

pub trait SternBrocotTree: From<Rational<Self::T>> + FromIterator<Self::T> {
    type T: Signed;

    fn root() -> Self;

    fn is_root(&self) -> bool;

    fn eval(&self) -> Rational<Self::T>;

    fn down_left(&mut self, count: Self::T);

    fn down_right(&mut self, count: Self::T);

    /// Returns the remaining count after moving up.
    fn up(&mut self, count: Self::T) -> Self::T;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SbtNode<T>
where
    T: Signed,
{
    pub l: Rational<T>,
    pub r: Rational<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SbtPath<T>
where
    T: Signed,
{
    pub path: Vec<T>,
}

impl<T> From<Rational<T>> for SbtNode<T>
where
    T: Signed,
{
    fn from(r: Rational<T>) -> Self {
        SbtPath::from(r).to_node()
    }
}

impl<T> FromIterator<T> for SbtNode<T>
where
    T: Signed,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut node = SbtNode::root();
        for (i, count) in iter.into_iter().enumerate() {
            if i % 2 == 0 {
                node.down_right(count);
            } else {
                node.down_left(count);
            }
        }
        node
    }
}

impl<T> From<Rational<T>> for SbtPath<T>
where
    T: Signed,
{
    fn from(r: Rational<T>) -> Self {
        assert!(r.num.is_positive(), "rational must be positive");
        assert!(r.den.is_positive(), "rational must be positive");

        let (mut a, mut b) = (r.num, r.den);
        let mut path = vec![];
        loop {
            let x = a / b;
            a %= b;
            if a.is_zero() {
                if !x.is_one() {
                    path.push(x - T::one());
                }
                break;
            }
            path.push(x);
            swap(&mut a, &mut b);
        }
        Self { path }
    }
}

impl<T> FromIterator<T> for SbtPath<T>
where
    T: Signed,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut path = SbtPath::root();
        for (i, count) in iter.into_iter().enumerate() {
            if i % 2 == 0 {
                path.down_right(count);
            } else {
                path.down_left(count);
            }
        }
        path
    }
}

impl<T> IntoIterator for SbtPath<T>
where
    T: Signed,
{
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.path.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a SbtPath<T>
where
    T: Signed,
{
    type Item = T;
    type IntoIter = std::iter::Cloned<std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.path.iter().cloned()
    }
}

impl<T> SternBrocotTree for SbtNode<T>
where
    T: Signed,
{
    type T = T;

    fn root() -> Self {
        Self {
            l: Rational::new(T::zero(), T::one()),
            r: Rational::new(T::one(), T::zero()),
        }
    }

    fn is_root(&self) -> bool {
        self.l.num.is_zero() && self.r.den.is_zero()
    }

    fn eval(&self) -> Rational<Self::T> {
        Rational::new_unchecked(self.l.num + self.r.num, self.l.den + self.r.den)
    }

    fn down_left(&mut self, count: Self::T) {
        assert!(!count.is_negative(), "count must be non-negative");
        self.r.num += self.l.num * count;
        self.r.den += self.l.den * count;
    }

    fn down_right(&mut self, count: Self::T) {
        assert!(!count.is_negative(), "count must be non-negative");
        self.l.num += self.r.num * count;
        self.l.den += self.r.den * count;
    }

    fn up(&mut self, mut count: Self::T) -> Self::T {
        assert!(!count.is_negative(), "count must be non-negative");
        while count > T::zero() && !self.is_root() {
            if self.l.den > self.r.den {
                let x = count.min(self.l.num / self.r.num);
                count -= x;
                self.l.num -= self.r.num * x;
                self.l.den -= self.r.den * x;
            } else {
                let x = count.min(self.r.den / self.l.den);
                count -= x;
                self.r.num -= self.l.num * x;
                self.r.den -= self.l.den * x;
            }
        }
        count
    }
}

impl<T> SternBrocotTree for SbtPath<T>
where
    T: Signed,
{
    type T = T;

    fn root() -> Self {
        Self::default()
    }

    fn is_root(&self) -> bool {
        self.path.is_empty() || (self.path.len() == 1 && self.path[0].is_zero())
    }

    fn eval(&self) -> Rational<Self::T> {
        self.to_node().eval()
    }

    fn down_left(&mut self, count: Self::T) {
        assert!(!count.is_negative(), "count must be non-negative");
        if count.is_zero() {
            return;
        }
        if self.path.len() % 2 == 0 {
            if let Some(last) = self.path.last_mut() {
                *last += count;
            } else {
                self.path.push(T::zero());
                self.path.push(count);
            }
        } else {
            self.path.push(count);
        }
    }

    fn down_right(&mut self, count: Self::T) {
        assert!(!count.is_negative(), "count must be non-negative");
        if count.is_zero() {
            return;
        }
        if self.path.len() % 2 == 0 {
            self.path.push(count);
        } else {
            *self.path.last_mut().unwrap() += count;
        }
    }

    fn up(&mut self, mut count: Self::T) -> Self::T {
        assert!(!count.is_negative(), "count must be non-negative");
        while let Some(last) = self.path.last_mut() {
            let x = count.min(*last);
            *last -= x;
            count -= x;
            if !last.is_zero() {
                break;
            }
            self.path.pop();
        }
        count
    }
}

impl<T> SbtNode<T>
where
    T: Signed,
{
    pub fn to_path(&self) -> SbtPath<T> {
        self.eval().into()
    }
    pub fn lca<I, J>(path1: I, path2: J) -> Self
    where
        I: IntoIterator<Item = T>,
        J: IntoIterator<Item = T>,
    {
        let mut node = SbtNode::root();
        for (i, (count1, count2)) in path1.into_iter().zip(path2).enumerate() {
            let count = count1.min(count2);
            if i % 2 == 0 {
                node.down_right(count);
            } else {
                node.down_left(count);
            }
            if count1 != count2 {
                break;
            }
        }
        node
    }
}

impl<T> SbtPath<T>
where
    T: Signed,
{
    pub fn to_node(&self) -> SbtNode<T> {
        self.path.iter().cloned().collect()
    }
    pub fn depth(&self) -> T {
        self.path.iter().cloned().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_sbt_path_encode_decode() {
        for a in 1..50 {
            for b in 1..50 {
                let r = Rational::new(a, b);
                let path = SbtPath::from(r);
                let node = path.to_node();
                assert_eq!(node.eval(), r);
            }
        }
    }

    #[test]
    fn test_sbt_explore() {
        let mut rng = Xorshift::default();
        for _ in 0..10000 {
            let mut node = SbtNode::<i128>::root();
            let mut path = SbtPath::<i128>::root();
            for _ in 0..30 {
                match rng.random(0..3) {
                    0 => {
                        let count = rng.random(0..=100);
                        node.down_left(count);
                        path.down_left(count);
                    }
                    1 => {
                        let count = rng.random(0..=100);
                        node.down_right(count);
                        path.down_right(count);
                    }
                    _ => {
                        let count = rng.random(0..=100);
                        let r1 = path.up(count);
                        let r2 = node.up(count);
                        assert_eq!(r1, r2);
                    }
                }
                assert_eq!(node, path.to_node());
                assert_eq!(node.eval(), path.eval());
                assert_eq!(node.is_root(), path.is_root());
                assert_eq!(node.to_path(), path);
                assert_eq!(node, path.to_node());
            }
        }
    }
}
