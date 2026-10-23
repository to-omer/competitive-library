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

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn proof_subset_step() {
        let mask: u64 = kani::any();
        let current: u64 = kani::any();
        let candidate: u64 = kani::any();
        kani::assume(current & mask == current);
        kani::assert(
            mask.subsets().next() == Some(mask),
            "For every u64 mask, subset enumeration starts with the entire mask",
        );
        // Every submask is a reachable iterator state; verify a transition without enumerating 2^64 states.
        let mut iter = Subsets {
            mask,
            cur: Some(current),
        };
        kani::assert(
            iter.next() == Some(current),
            "For every u64 mask and current submask, the iterator yields that submask",
        );
        match iter.next() {
            Some(next) => {
                kani::assert(
                    next < current && next & mask == next,
                    "Each u64 subset step strictly decreases while remaining a submask",
                );
                kani::assert(
                    !(candidate < current && candidate & mask == candidate) || candidate <= next,
                    "Each u64 subset step chooses the greatest smaller submask, so it skips none",
                );
            }
            None => kani::assert(
                current == 0 && iter.next().is_none(),
                "u64 subset enumeration ends exactly after zero and stays exhausted",
            ),
        }
        kani::cover!(mask == u64::MAX && current == 1 << 63);
        kani::cover!(mask == 0);
    }

    #[kani::proof]
    fn proof_combination_start() {
        let n: usize = kani::any();
        let k: usize = kani::any();
        let candidate: u64 = kani::any();
        kani::assume(k <= n && n < u64::BITS as usize);
        let first = u64::combinations(n, k).next().unwrap();
        kani::assert(
            first < 1u64 << n && first.count_ones() as usize == k,
            "For every 0 <= k <= n < 64, u64 combination enumeration starts with a valid k-bit selection among n bits",
        );
        kani::assert(
            candidate.count_ones() as usize != k || first <= candidate,
            "The first u64 combination is the smallest integer with the requested number of selected bits",
        );
        kani::cover!(n == 63 && k == 63);
        kani::cover!(n == 0 && k == 0);
    }

    #[kani::proof]
    fn proof_combination_step() {
        let n: usize = kani::any();
        let current: u64 = kani::any();
        let candidate: u64 = kani::any();
        kani::assume(n < u64::BITS as usize);
        let mask = 1u64 << n;
        kani::assume(current < mask);
        // A valid state of any fixed-popcount enumeration on at most sixty-three bits.
        let mut iter = Combinations {
            mask,
            cur: Some(current),
        };
        kani::assert(
            iter.next() == Some(current),
            "For every u64 combination state on 0 through 63 bits, the iterator yields the current combination",
        );
        match iter.next() {
            Some(next) => {
                kani::assert(
                    current < next && next < mask && next.count_ones() == current.count_ones(),
                    "Every next u64 combination on at most 63 bits increases and preserves the number of selected bits",
                );
                kani::assert(
                    !(current < candidate
                        && candidate < mask
                        && candidate.count_ones() == current.count_ones())
                        || next <= candidate,
                    "The next u64 combination is the smallest larger value with the same popcount, so it skips none",
                );
            }
            None => kani::assert(
                !(current < candidate
                    && candidate < mask
                    && candidate.count_ones() == current.count_ones())
                    && iter.next().is_none(),
                "u64 combination enumeration ends exactly when no larger combination remains and stays exhausted",
            ),
        }
        kani::cover!(n == 0);
        kani::cover!(n == 63 && current == 0b0101011);
        kani::cover!(n == 63 && current == (1u64 << 63) - 1);
        kani::cover!(n == 63 && current == 1u64 << 62);
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
