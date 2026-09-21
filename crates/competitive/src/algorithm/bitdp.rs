use super::{One, Zero};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Not, Shl, Shr, Sub};

pub trait BitDpExt:
    Sized
    + Copy
    + Default
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + Zero
    + One
{
    fn contains(self, x: usize) -> bool {
        self & (Self::one() << x) != Self::zero()
    }
    fn insert(self, x: usize) -> Self {
        self | (Self::one() << x)
    }
    fn remove(self, x: usize) -> Self {
        self & !(Self::one() << x)
    }
    fn is_subset(self, elements: Self) -> bool {
        self & elements == elements
    }
    fn is_superset(self, elements: Self) -> bool {
        elements.is_subset(self)
    }
    fn subsets(self) -> Subsets<Self> {
        Subsets {
            mask: self,
            cur: Some(self),
        }
    }
    fn combinations(n: usize, k: usize) -> Combinations<Self> {
        Combinations {
            mask: Self::one() << n,
            cur: Some((Self::one() << k) - Self::one()),
        }
    }
}

impl BitDpExt for u8 {}
impl BitDpExt for u16 {}
impl BitDpExt for u32 {}
impl BitDpExt for u64 {}
impl BitDpExt for u128 {}
impl BitDpExt for usize {}

#[derive(Debug, Clone)]
pub struct Subsets<T> {
    mask: T,
    cur: Option<T>,
}

impl<T> Iterator for Subsets<T>
where
    T: BitDpExt,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur) = self.cur {
            self.cur = if cur.is_zero() {
                None
            } else {
                Some((cur - T::one()) & self.mask)
            };
            Some(cur)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Combinations<T> {
    mask: T,
    cur: Option<T>,
}

impl<T> Iterator for Combinations<T>
where
    T: BitDpExt,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur) = self.cur {
            if cur < self.mask {
                self.cur = if cur == T::zero() {
                    None
                } else {
                    let x = cur & (!cur + T::one());
                    let y = cur + x;
                    Some(((cur & !y) / x / (T::one() + T::one())) | y)
                };
                Some(cur)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::BTreeSet;

    #[test]
    fn test_bit_operations() {
        let mut rng = Xorshift::default();
        let pairs: Vec<_> = (0..256usize)
            .flat_map(|a| (0..256).map(move |b| (a, b)))
            .chain(
                rng.random_iter((0..1usize << 12, 0..1usize << 12))
                    .take(1000),
            )
            .collect();
        for (a, b) in pairs {
            let set: BTreeSet<_> = (0..12).filter(|&i| a >> i & 1 != 0).collect();
            let other: BTreeSet<_> = (0..12).filter(|&i| b >> i & 1 != 0).collect();
            for i in 0..12 {
                assert_eq!(a.contains(i), set.contains(&i));
                let mut inserted = set.clone();
                inserted.insert(i);
                assert_eq!(a.insert(i), inserted.iter().map(|i| 1 << i).sum());
                let mut removed = set.clone();
                removed.remove(&i);
                assert_eq!(a.remove(i), removed.iter().map(|i| 1 << i).sum());
            }
            assert_eq!(a.is_subset(b), other.is_subset(&set));
            assert_eq!(a.is_superset(b), other.is_superset(&set));
        }
        for a in 0..1usize << 12 {
            let mut subsets: Vec<_> = a.subsets().collect();
            subsets.sort();
            assert_eq!(subsets, (0..=a).filter(|x| x & a == *x).collect::<Vec<_>>());
        }
        for n in 0..=12 {
            for k in 0..=n {
                let expected: Vec<_> = (0..1usize << n)
                    .filter(|x| x.count_ones() as usize == k)
                    .collect();
                assert_eq!(usize::combinations(n, k).collect::<Vec<_>>(), expected);
            }
        }
    }
}
