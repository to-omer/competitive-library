use std::iter::Peekable;

pub trait IteratorExt: Iterator {
    fn merge_by<I, F>(self, other: I, is_first: F) -> MergeBy<Self, I, F>
    where
        Self: Sized,
        I: Iterator<Item = Self::Item>,
        F: FnMut(&Self::Item, &Self::Item) -> bool,
    {
        MergeBy {
            left: self.peekable(),
            right: other.peekable(),
            is_first,
        }
    }
}

impl<I> IteratorExt for I where I: Iterator {}

pub struct MergeBy<I, J, F>
where
    I: Iterator,
    J: Iterator<Item = I::Item>,
{
    left: Peekable<I>,
    right: Peekable<J>,
    is_first: F,
}

impl<I, J, F> Iterator for MergeBy<I, J, F>
where
    I: Iterator,
    J: Iterator<Item = I::Item>,
    F: FnMut(&I::Item, &I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.left.peek(), self.right.peek()) {
            (Some(l), Some(r)) => {
                if (self.is_first)(l, r) {
                    self.left.next()
                } else {
                    self.right.next()
                }
            }
            (Some(_), None) => self.left.next(),
            (None, Some(_)) => self.right.next(),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_merge_by() {
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let n = rng.random(0..=100);
            let m = rng.random(0..=100);
            let mut a: Vec<_> = rng.random_iter(-20..=20).take(n).collect();
            let mut b: Vec<_> = rng.random_iter(-20..=20).take(m).collect();
            let mut expected: Vec<_> = a.iter().chain(&b).copied().collect();
            a.sort();
            b.sort();
            expected.sort();
            assert_eq!(
                a.iter()
                    .merge_by(b.iter(), |x, y| x < y)
                    .copied()
                    .collect::<Vec<_>>(),
                expected
            );
            expected.reverse();
            assert_eq!(
                a.iter()
                    .rev()
                    .merge_by(b.iter().rev(), |x, y| x > y)
                    .copied()
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }
}
