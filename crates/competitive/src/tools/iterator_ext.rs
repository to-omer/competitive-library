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

    #[test]
    fn test_merge_by() {
        let a = vec![1, 4, 5];
        let b = vec![2, 3, 6];
        let merged: Vec<_> = a
            .into_iter()
            .merge_by(b.into_iter(), |x, y| x < y)
            .collect();
        assert_eq!(merged, vec![1, 2, 3, 4, 5, 6]);
    }
}
