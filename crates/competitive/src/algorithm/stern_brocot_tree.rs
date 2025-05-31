use super::{Rational, Signed};
use std::mem::swap;

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

impl<T> From<Rational<T>> for SbtPath<T>
where
    T: Signed,
{
    fn from(r: Rational<T>) -> Self {
        SbtPath::encode(r)
    }
}

impl<T> From<SbtNode<T>> for Rational<T>
where
    T: Signed,
{
    fn from(node: SbtNode<T>) -> Self {
        node.eval()
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
        SbtNode::decode(iter)
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

impl<T> SbtNode<T>
where
    T: Signed,
{
    pub fn root() -> Self {
        Self {
            l: Rational::new(T::zero(), T::one()),
            r: Rational::new(T::one(), T::zero()),
        }
    }
    pub fn decode<I>(path: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut node = SbtNode::root();
        for (i, count) in path.into_iter().enumerate() {
            if i % 2 == 0 {
                node.down_right(count);
            } else {
                node.down_left(count);
            }
        }
        node
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
    pub fn eval(&self) -> Rational<T> {
        Rational::new_unchecked(self.l.num + self.r.num, self.l.den + self.r.den)
    }
    pub fn down_left(&mut self, count: T) {
        assert!(!count.is_negative(), "count must be non-negative");
        self.r.num += self.l.num * count;
        self.r.den += self.l.den * count;
    }
    pub fn down_right(&mut self, count: T) {
        assert!(!count.is_negative(), "count must be non-negative");
        self.l.num += self.r.num * count;
        self.l.den += self.r.den * count;
    }
}

impl<T> SbtPath<T>
where
    T: Signed,
{
    pub fn encode(r: Rational<T>) -> Self {
        assert!(r.num.is_positive(), "rational must be positive");
        assert!(r.den.is_positive(), "rational must be positive");

        let (mut a, mut b) = (r.num, r.den);
        let mut path = vec![];
        loop {
            let x = a / b;
            a %= b;
            if a.is_zero() {
                path.push(x - T::one());
                break;
            }
            path.push(x);
            swap(&mut a, &mut b);
        }
        Self { path }
    }
    pub fn decode(&self) -> SbtNode<T> {
        self.path.iter().cloned().collect()
    }
    pub fn down_left(&mut self, count: T) {
        assert!(!count.is_negative(), "count must be non-negative");
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
    pub fn down_right(&mut self, count: T) {
        assert!(!count.is_negative(), "count must be non-negative");
        if self.path.len() % 2 == 0 {
            self.path.push(count);
        } else {
            *self.path.last_mut().unwrap() += count;
        }
    }
    pub fn up(&mut self, mut count: T) -> T {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbt_path_encode_decode() {
        for a in 1..50 {
            for b in 1..50 {
                let r = Rational::new(a, b);
                let path = SbtPath::encode(r);
                let node = path.decode();
                assert_eq!(node.eval(), r);
            }
        }
    }
}
