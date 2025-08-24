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

    #[test]
    fn test_contains() {
        assert!(!0b1010u8.contains(0));
        assert!(0b1010u8.contains(1));
        assert!(!0b1010u8.contains(2));
        assert!(0b1010u8.contains(3));
    }

    #[test]
    fn test_insert() {
        assert_eq!(0b1010u8.insert(0), 0b1011);
        assert_eq!(0b1010u8.insert(1), 0b1010);
        assert_eq!(0b1010u8.insert(2), 0b1110);
        assert_eq!(0b1010u8.insert(3), 0b1010);
    }

    #[test]
    fn test_remove() {
        assert_eq!(0b1010u8.remove(0), 0b1010);
        assert_eq!(0b1010u8.remove(1), 0b1000);
        assert_eq!(0b1010u8.remove(2), 0b1010);
        assert_eq!(0b1010u8.remove(3), 0b0010);
    }

    #[test]
    fn test_is_subset() {
        assert!(0b1010u8.is_subset(0b1010));
        assert!(0b1010u8.is_subset(0b0000));
        assert!(!0b1010u8.is_subset(0b0100));
        assert!(!0b1010u8.is_subset(0b10000));
    }

    #[test]
    fn test_is_superset() {
        assert!(0b1010u8.is_superset(0b1010));
        assert!(0b1010u8.is_superset(0b1111));
        assert!(!0b1010u8.is_superset(0b0000));
        assert!(!0b1010u8.is_superset(0b10000));
    }

    #[test]
    fn test_subsets() {
        for mask in 0usize..1 << 12 {
            let mut subsets = mask.subsets().collect::<Vec<_>>();
            let n = subsets.len();
            assert_eq!(n, 1 << mask.count_ones());
            assert!(subsets.iter().all(|&s| mask.is_subset(s)));
            subsets.sort_unstable();
            subsets.dedup();
            assert_eq!(n, subsets.len());
        }
    }

    #[test]
    fn test_combinations() {
        let mut comb = vec![vec![0; 14]; 14];
        comb[0][0] = 1;
        for i in 0..=12 {
            for j in 0..=12 {
                comb[i + 1][j] += comb[i][j];
                comb[i][j + 1] += comb[i][j];
            }
        }

        for n in 0..=12 {
            for k in 0..=n {
                let mut combinations = usize::combinations(n, k).collect::<Vec<_>>();
                let len = combinations.len();
                assert_eq!(len, comb[n - k][k]);
                assert!(combinations.iter().all(|&s| s.count_ones() as usize == k));
                combinations.sort_unstable();
                combinations.dedup();
                assert_eq!(len, combinations.len());
            }
        }
    }
}
