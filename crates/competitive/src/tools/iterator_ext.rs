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
    use crate::tools::{
        Xorshift,
        testutil::{integer_cases, sequence_cases},
    };

    #[test]
    fn test_merge_by() {
        let mut rng = Xorshift::default();
        for values in sequence_cases(&mut rng, -20..=20, 0..=1024) {
            for split in integer_cases(0..=values.len()) {
                let mut a: Vec<_> = values[..split]
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| (value, 0, i))
                    .collect();
                let mut b: Vec<_> = values[split..]
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| (value, 1, i))
                    .collect();
                a.sort_by_key(|x| x.0);
                b.sort_by_key(|x| x.0);
                for descending in [false, true] {
                    if descending {
                        a.reverse();
                        b.reverse();
                    }
                    for left_first in [false, true] {
                        let mut expected: Vec<_> = a.iter().chain(&b).copied().collect();
                        expected.sort_by_key(|&(value, side, _)| {
                            (
                                if descending { -value } else { value },
                                if left_first { side } else { 1 - side },
                            )
                        });
                        let actual = a.iter().merge_by(b.iter(), |x, y| {
                            if x.0 == y.0 {
                                left_first
                            } else if descending {
                                x.0 > y.0
                            } else {
                                x.0 < y.0
                            }
                        });
                        assert!(
                            actual.copied().eq(expected),
                            "left={a:?}, right={b:?}, descending={descending}, left_first={left_first}"
                        );
                    }
                }
            }
        }
    }
}
