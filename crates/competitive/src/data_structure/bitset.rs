#![allow(clippy::suspicious_op_assign_impl)]

use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitSet {
    size: usize,
    bits: Vec<u64>,
}
impl BitSet {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            bits: vec![0; (size + 63) / 64],
        }
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    pub fn ones(size: usize) -> Self {
        let mut self_ = Self {
            size,
            bits: vec![u64::MAX; (size + 63) / 64],
        };
        self_.trim();
        self_
    }
    #[inline]
    pub fn get(&self, i: usize) -> bool {
        self.bits[i >> 6] & (1 << (i & 63)) != 0
    }
    #[inline]
    pub fn set(&mut self, i: usize, b: bool) {
        if b {
            self.bits[i >> 6] |= 1 << (i & 63);
        } else {
            self.bits[i >> 6] &= !(1 << (i & 63));
        }
    }
    #[inline]
    pub fn count_ones(&self) -> u64 {
        self.bits.iter().map(|x| x.count_ones() as u64).sum()
    }
    #[inline]
    pub fn count_zeros(&self) -> u64 {
        self.size as u64 - self.count_ones()
    }
    pub fn push(&mut self, b: bool) {
        let d = self.size & 63;
        if d == 0 {
            self.bits.push(b as u64);
        } else {
            *self.bits.last_mut().unwrap() |= (b as u64) << d;
        }
        self.size += 1;
    }
    #[inline]
    fn trim(&mut self) {
        if self.size & 63 != 0 {
            if let Some(x) = self.bits.last_mut() {
                *x &= 0xffff_ffff_ffff_ffff >> (64 - (self.size & 63));
            }
        }
    }
    #[inline]
    pub fn shl_bitor_assign(&mut self, rhs: usize) {
        let n = self.bits.len();
        let k = rhs >> 6;
        let d = rhs & 63;
        if k < n {
            if d == 0 {
                for i in (0..n - k).rev() {
                    self.bits[i + k] |= self.bits[i];
                }
            } else {
                for i in (1..n - k).rev() {
                    self.bits[i + k] |= (self.bits[i] << d) | (self.bits[i - 1] >> (64 - d));
                }
                self.bits[k] |= self.bits[0] << d;
            }
            self.trim();
        }
    }
    #[inline]
    pub fn shr_bitor_assign(&mut self, rhs: usize) {
        let n = self.bits.len();
        let k = rhs >> 6;
        let d = rhs & 63;
        if k < n {
            if d == 0 {
                for i in k..n {
                    self.bits[i - k] |= self.bits[i];
                }
            } else {
                for i in k..n - 1 {
                    self.bits[i - k] |= (self.bits[i] >> d) | (self.bits[i + 1] << (64 - d));
                }
                self.bits[n - k - 1] |= self.bits[n - 1] >> d;
            }
        }
    }
}
impl Extend<bool> for BitSet {
    fn extend<T: IntoIterator<Item = bool>>(&mut self, iter: T) {
        let d = self.size & 63;
        let mut iter = iter.into_iter();
        let Some(first) = iter.next() else {
            return;
        };
        if d == 0 {
            self.bits.push(0);
        }
        let mut e = self.bits.last_mut().unwrap();
        *e |= (first as u64) << d;
        self.size += 1;
        for b in iter {
            let d = self.size & 63;
            if d == 0 {
                self.bits.push(b as u64);
                e = self.bits.last_mut().unwrap();
            } else {
                *e |= (b as u64) << d;
            }
            self.size += 1;
        }
    }
}
impl FromIterator<bool> for BitSet {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let mut set = BitSet::new(0);
        set.extend(iter);
        set
    }
}
impl ShlAssign<usize> for BitSet {
    #[inline]
    fn shl_assign(&mut self, rhs: usize) {
        let n = self.bits.len();
        let k = rhs >> 6;
        let d = rhs & 63;
        if k >= n {
            for x in self.bits.iter_mut() {
                *x = 0;
            }
        } else {
            if d == 0 {
                for i in (0..n - k).rev() {
                    self.bits[i + k] = self.bits[i];
                }
            } else {
                for i in (1..n - k).rev() {
                    self.bits[i + k] = (self.bits[i] << d) | (self.bits[i - 1] >> (64 - d));
                }
                self.bits[k] = self.bits[0] << d;
            }
            for x in self.bits[..k].iter_mut() {
                *x = 0;
            }
            self.trim();
        }
    }
}
impl Shl<usize> for BitSet {
    type Output = Self;
    #[inline]
    fn shl(mut self, rhs: usize) -> Self::Output {
        self <<= rhs;
        self
    }
}
impl ShrAssign<usize> for BitSet {
    #[inline]
    fn shr_assign(&mut self, rhs: usize) {
        let n = self.bits.len();
        let k = rhs >> 6;
        let d = rhs & 63;
        if k >= n {
            for x in self.bits.iter_mut() {
                *x = 0;
            }
        } else {
            if d == 0 {
                for i in k..n {
                    self.bits[i - k] = self.bits[i];
                }
            } else {
                for i in k..n - 1 {
                    self.bits[i - k] = (self.bits[i] >> d) | (self.bits[i + 1] << (64 - d));
                }
                self.bits[n - k - 1] = self.bits[n - 1] >> d;
            }
            for x in self.bits[n - k..].iter_mut() {
                *x = 0;
            }
        }
    }
}
impl Shr<usize> for BitSet {
    type Output = Self;
    #[inline]
    fn shr(mut self, rhs: usize) -> Self::Output {
        self >>= rhs;
        self
    }
}
impl<'a> BitOrAssign<&'a BitSet> for BitSet {
    #[inline]
    fn bitor_assign(&mut self, rhs: &'a Self) {
        for (l, r) in self.bits.iter_mut().zip(rhs.bits.iter()) {
            *l |= *r;
        }
        self.trim();
    }
}
impl<'a> BitOr<&'a BitSet> for BitSet {
    type Output = Self;
    #[inline]
    fn bitor(mut self, rhs: &'a Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl<'b> BitOr<&'b BitSet> for &BitSet {
    type Output = BitSet;
    #[inline]
    fn bitor(self, rhs: &'b BitSet) -> Self::Output {
        let mut res = self.clone();
        res |= rhs;
        res
    }
}
impl<'a> BitAndAssign<&'a BitSet> for BitSet {
    #[inline]
    fn bitand_assign(&mut self, rhs: &'a Self) {
        for (l, r) in self.bits.iter_mut().zip(rhs.bits.iter()) {
            *l &= *r;
        }
    }
}
impl<'a> BitAnd<&'a BitSet> for BitSet {
    type Output = Self;
    #[inline]
    fn bitand(mut self, rhs: &'a Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl<'b> BitAnd<&'b BitSet> for &BitSet {
    type Output = BitSet;
    #[inline]
    fn bitand(self, rhs: &'b BitSet) -> Self::Output {
        let mut res = self.clone();
        res &= rhs;
        res
    }
}
impl<'a> BitXorAssign<&'a BitSet> for BitSet {
    #[inline]
    fn bitxor_assign(&mut self, rhs: &'a Self) {
        for (l, r) in self.bits.iter_mut().zip(rhs.bits.iter()) {
            *l ^= *r;
        }
        self.trim();
    }
}
impl<'a> BitXor<&'a BitSet> for BitSet {
    type Output = Self;
    #[inline]
    fn bitxor(mut self, rhs: &'a Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl<'b> BitXor<&'b BitSet> for &BitSet {
    type Output = BitSet;
    #[inline]
    fn bitxor(self, rhs: &'b BitSet) -> Self::Output {
        let mut res = self.clone();
        res ^= rhs;
        res
    }
}
impl Not for BitSet {
    type Output = Self;
    #[inline]
    fn not(mut self) -> Self::Output {
        for x in self.bits.iter_mut() {
            *x = !*x;
        }
        self.trim();
        self
    }
}
impl Not for &BitSet {
    type Output = BitSet;
    #[inline]
    fn not(self) -> Self::Output {
        !self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    #[test]
    fn test_extend() {
        for _ in 0..100 {
            let mut rng = Xorshift::new();
            rand!(rng, arr: [0..=1u32; 200], n1: 0..=200);
            let mut bitset: BitSet = arr[..n1].iter().map(|&x| x != 0).collect();
            bitset.extend(arr[n1..].iter().map(|&x| x != 0));
            assert_eq!(bitset.len(), 200);
            for (i, &x) in arr.iter().enumerate() {
                assert_eq!(bitset.get(i), x != 0);
            }
        }
    }
}
