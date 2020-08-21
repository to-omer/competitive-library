#[cargo_snippet::snippet("BitSet")]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitSet {
    size: usize,
    bits: Vec<u64>,
}
#[cargo_snippet::snippet("BitSet")]
impl BitSet {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            bits: vec![0; (size + 63) / 64],
        }
    }
    pub fn ones(size: usize) -> Self {
        let mut self_ = Self {
            size,
            bits: vec![std::u64::MAX; (size + 63) / 64],
        };
        self_.trim();
        self_
    }
    #[inline]
    pub fn get(&self, i: usize) -> bool {
        self.bits[i >> 6] & 1 << (i & 63) != 0
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
    #[inline]
    fn trim(&mut self) {
        if self.size & 63 != 0 {
            if let Some(x) = self.bits.last_mut() {
                *x &= 0xffffffffffffffff >> 64 - (self.size & 63);
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
                    self.bits[i + k] |= self.bits[i] << d | self.bits[i - 1] >> 64 - d;
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
                    self.bits[i - k] |= self.bits[i] >> d | self.bits[i + 1] << 64 - d;
                }
                self.bits[n - k - 1] |= self.bits[n - 1] >> d;
            }
        }
    }
}
#[cargo_snippet::snippet("BitSet")]
impl std::ops::ShlAssign<usize> for BitSet {
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
                    self.bits[i + k] = self.bits[i] << d | self.bits[i - 1] >> 64 - d;
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
#[cargo_snippet::snippet("BitSet")]
impl std::ops::Shl<usize> for BitSet {
    type Output = Self;
    #[inline]
    fn shl(mut self, rhs: usize) -> Self::Output {
        self <<= rhs;
        self
    }
}
#[cargo_snippet::snippet("BitSet")]
impl std::ops::ShrAssign<usize> for BitSet {
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
                    self.bits[i - k] = self.bits[i] >> d | self.bits[i + 1] << 64 - d;
                }
                self.bits[n - k - 1] = self.bits[n - 1] >> d;
            }
            for x in self.bits[n - k..].iter_mut() {
                *x = 0;
            }
        }
    }
}
#[cargo_snippet::snippet("BitSet")]
impl std::ops::Shr<usize> for BitSet {
    type Output = Self;
    #[inline]
    fn shr(mut self, rhs: usize) -> Self::Output {
        self >>= rhs;
        self
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a> std::ops::BitOrAssign<&'a BitSet> for BitSet {
    #[inline]
    fn bitor_assign(&mut self, rhs: &'a Self) {
        for (l, r) in self.bits.iter_mut().zip(rhs.bits.iter()) {
            *l |= *r;
        }
        self.trim();
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a> std::ops::BitOr<&'a BitSet> for BitSet {
    type Output = Self;
    #[inline]
    fn bitor(mut self, rhs: &'a Self) -> Self::Output {
        self |= rhs;
        self
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a, 'b> std::ops::BitOr<&'b BitSet> for &'a BitSet {
    type Output = BitSet;
    #[inline]
    fn bitor(self, rhs: &'b BitSet) -> Self::Output {
        let mut res = self.clone();
        res |= rhs;
        res
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a> std::ops::BitAndAssign<&'a BitSet> for BitSet {
    #[inline]
    fn bitand_assign(&mut self, rhs: &'a Self) {
        for (l, r) in self.bits.iter_mut().zip(rhs.bits.iter()) {
            *l &= *r;
        }
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a> std::ops::BitAnd<&'a BitSet> for BitSet {
    type Output = Self;
    #[inline]
    fn bitand(mut self, rhs: &'a Self) -> Self::Output {
        self &= rhs;
        self
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a, 'b> std::ops::BitAnd<&'b BitSet> for &'a BitSet {
    type Output = BitSet;
    #[inline]
    fn bitand(self, rhs: &'b BitSet) -> Self::Output {
        let mut res = self.clone();
        res &= rhs;
        res
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a> std::ops::BitXorAssign<&'a BitSet> for BitSet {
    #[inline]
    fn bitxor_assign(&mut self, rhs: &'a Self) {
        for (l, r) in self.bits.iter_mut().zip(rhs.bits.iter()) {
            *l ^= *r;
        }
        self.trim();
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a> std::ops::BitXor<&'a BitSet> for BitSet {
    type Output = Self;
    #[inline]
    fn bitxor(mut self, rhs: &'a Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
#[cargo_snippet::snippet("BitSet")]
impl<'a, 'b> std::ops::BitXor<&'b BitSet> for &'a BitSet {
    type Output = BitSet;
    #[inline]
    fn bitxor(self, rhs: &'b BitSet) -> Self::Output {
        let mut res = self.clone();
        res ^= rhs;
        res
    }
}
#[cargo_snippet::snippet("BitSet")]
impl std::ops::Not for BitSet {
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
#[cargo_snippet::snippet("BitSet")]
impl<'a> std::ops::Not for &'a BitSet {
    type Output = BitSet;
    #[inline]
    fn not(self) -> Self::Output {
        !self.clone()
    }
}
