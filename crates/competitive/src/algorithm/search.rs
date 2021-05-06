//! binary / ternary search

/// binary search helper
#[codesnip::entry("binary_search")]
pub trait Bisect: Clone {
    /// return between two elements
    fn halve(&self, other: &Self) -> Self;
    /// the end condition of binary search
    fn section_end(&self, other: &Self) -> bool;
}

#[codesnip::entry("binary_search")]
mod bisect_impl {
    use super::*;
    macro_rules! impl_bisect_unsigned {
        ($($t:ty)*) => {
            $(impl Bisect for $t {
                fn halve(&self, other: &Self) -> Self {
                    if self > other {
                        other + (self - other) / 2
                    } else {
                        self + (other - self) / 2
                    }
                }
                fn section_end(&self, other: &Self) -> bool {
                    (if self > other {
                        self - other
                    } else {
                        other - self
                    }) <= 1
                }
            })*
        };
    }
    macro_rules! impl_bisect_signed {
        ($($t:ty)*) => {
            $(impl Bisect for $t {
                fn halve(&self, other: &Self) -> Self {
                    (self + other) / 2
                }
                fn section_end(&self, other: &Self) -> bool {
                    (self - other).abs() <= 1
                }
            })*
        };
    }
    macro_rules! impl_bisect_float {
        ($($t:ty)*) => {
            $(impl Bisect for $t {
                fn halve(&self, other: &Self) -> Self {
                    (self + other) / 2.
                }
                fn section_end(&self, other: &Self) -> bool {
                    const BISECT_SECTION_END_EPS: $t = 1e-8;
                    (self - other).abs() <= BISECT_SECTION_END_EPS
                }
            })*
        };
    }
    impl_bisect_unsigned!(u8 u16 u32 u64 usize);
    impl_bisect_signed!(i8 i16 i32 i64 isize);
    impl_bisect_float!(f32 f64);
}

#[codesnip::entry("binary_search")]
/// binary search for monotone segment
///
/// if `ok < err` then search [ok, err) where t(`ok`), t, t, .... t, t(`ret`), f,  ... f, f, f, `err`
///
/// if `err < ok` then search (err, ok] where `err`, f, f, f, ... f, t(`ret`), ... t, t, t(`ok`)
pub fn binary_search<T>(mut f: impl FnMut(&T) -> bool, mut ok: T, mut err: T) -> T
where
    T: Bisect,
{
    while !ok.section_end(&err) {
        let m = ok.halve(&err);
        if f(&m) {
            ok = m;
        } else {
            err = m;
        }
    }
    ok
}

#[codesnip::entry("binary_search")]
/// binary search for slice
pub trait SliceBisectExt<T> {
    /// Returns the first element that satisfies a predicate.
    fn find_bisect(&self, f: impl FnMut(&T) -> bool) -> Option<&T>;
    /// Returns the last element that satisfies a predicate.
    fn rfind_bisect(&self, f: impl FnMut(&T) -> bool) -> Option<&T>;
    /// Returns the first index that satisfies a predicate.
    /// if not found, returns `len()`.
    fn position_bisect(&self, f: impl FnMut(&T) -> bool) -> usize;
    /// Returns the last index+1 that satisfies a predicate.
    /// if not found, returns `0`.
    fn rposition_bisect(&self, f: impl FnMut(&T) -> bool) -> usize;
}
#[codesnip::entry("binary_search")]
impl<T> SliceBisectExt<T> for [T] {
    fn find_bisect(&self, f: impl FnMut(&T) -> bool) -> Option<&T> {
        self.get(self.position_bisect(f))
    }
    fn rfind_bisect(&self, f: impl FnMut(&T) -> bool) -> Option<&T> {
        let pos = self.rposition_bisect(f);
        if pos == 0 {
            None
        } else {
            self.get(pos - 1)
        }
    }
    fn position_bisect(&self, mut f: impl FnMut(&T) -> bool) -> usize {
        binary_search(|i| f(&self[*i as usize]), self.len() as i64, -1) as usize
    }
    fn rposition_bisect(&self, mut f: impl FnMut(&T) -> bool) -> usize {
        binary_search(|i| f(&self[i - 1]), 0, self.len() + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const V: [i64; 10] = [0i64, 1, 1, 1, 2, 2, 3, 4, 7, 8];

    #[test]
    fn test_binary_search() {
        assert_eq!(binary_search(|&x| V[x] >= 1, V.len(), 0), 1);
        assert_eq!(binary_search(|&x| V[x] >= 2, V.len(), 0), 4);
        assert_eq!(binary_search(|&x| V[x] >= 3, V.len(), 0), 6);
        assert_eq!(binary_search(|&x| V[x] <= 1, 0, V.len()), 3);
        assert_eq!(binary_search(|&x| V[x] <= 2, 0, V.len()), 5);
        assert_eq!(binary_search(|&x| V[x] <= 3, 0, V.len()), 6);

        assert_eq!(
            binary_search(&|&x: &i64| V[x as usize] as i64 <= -1, -1, V.len() as i64),
            -1
        );

        let sq2 = binary_search(|&x| x * x <= 2., 1., 4.);
        let expect = 1.414_213_562_73;
        assert!(expect - 1e-8 <= sq2 && sq2 <= expect + 1e-8);
    }

    #[test]
    fn test_position() {
        assert_eq!(V.position_bisect(|&x| x >= -1), 0);
        assert_eq!(V.position_bisect(|&x| x >= 0), 0);
        assert_eq!(V.position_bisect(|&x| x >= 1), 1);
        assert_eq!(V.position_bisect(|&x| x >= 2), 4);
        assert_eq!(V.position_bisect(|&x| x >= 3), 6);
        assert_eq!(V.position_bisect(|&x| x >= 5), 8);
        assert_eq!(V.position_bisect(|&x| x >= 10), 10);
    }

    #[test]
    fn test_find() {
        assert_eq!(V.find_bisect(|&x| x >= -1), Some(&0));
        assert_eq!(V.find_bisect(|&x| x >= 0), Some(&0));
        assert_eq!(V.find_bisect(|&x| x >= 1), Some(&1));
        assert_eq!(V.find_bisect(|&x| x >= 2), Some(&2));
        assert_eq!(V.find_bisect(|&x| x >= 3), Some(&3));
        assert_eq!(V.find_bisect(|&x| x >= 5), Some(&7));
        assert_eq!(V.find_bisect(|&x| x >= 10), None);
    }

    #[test]
    fn test_rposition() {
        assert_eq!(V.rposition_bisect(|&x| x <= -1), 0);
        assert_eq!(V.rposition_bisect(|&x| x <= 0), 1);
        assert_eq!(V.rposition_bisect(|&x| x <= 1), 4);
        assert_eq!(V.rposition_bisect(|&x| x <= 2), 6);
        assert_eq!(V.rposition_bisect(|&x| x <= 3), 7);
        assert_eq!(V.rposition_bisect(|&x| x <= 5), 8);
        assert_eq!(V.rposition_bisect(|&x| x <= 10), 10);
    }

    #[test]
    fn test_rfind() {
        assert_eq!(V.rfind_bisect(|&x| x <= -1), None);
        assert_eq!(V.rfind_bisect(|&x| x <= 0), Some(&0));
        assert_eq!(V.rfind_bisect(|&x| x <= 1), Some(&1));
        assert_eq!(V.rfind_bisect(|&x| x <= 2), Some(&2));
        assert_eq!(V.rfind_bisect(|&x| x <= 3), Some(&3));
        assert_eq!(V.rfind_bisect(|&x| x <= 5), Some(&4));
        assert_eq!(V.rfind_bisect(|&x| x <= 10), Some(&8));
    }
}

#[codesnip::entry("ternary_search")]
/// ternary search helper
pub trait Trisect: Clone {
    /// Divide into 3 sections
    fn next_section(&self, other: &Self) -> (Self, Self);
    /// the end condition of ternary search
    fn section_end(&self, other: &Self) -> bool;
}
#[codesnip::entry("ternary_search")]
mod trisect_impl {
    use super::*;
    macro_rules! impl_trisect_unsigned {
        ($($t:ty)*) => {
            $(impl Trisect for $t {
                fn next_section(&self, other: &Self) -> (Self, Self) {
                    ((self * 2 + other) / 3, (self + other * 2) / 3)
                }
                fn section_end(&self, other: &Self) -> bool {
                    (if self > other {
                        self - other
                    } else {
                        other - self
                    }) <= 1
                }
            })*
        };
    }
    macro_rules! impl_trisect_signed {
        ($($t:ty)*) => {
            $(impl Trisect for $t {
                fn next_section(&self, other: &Self) -> (Self, Self) {
                    ((self * 2 + other) / 3, (self + other * 2) / 3)
                }
                fn section_end(&self, other: &Self) -> bool {
                    (self - other).abs() <= 1
                }
            })*
        };
    }
    macro_rules! impl_trisect_float {
        ($($t:ty)*) => {
            $(impl Trisect for $t {
                fn next_section(&self, other: &Self) -> (Self, Self) {
                    ((self * 2. + other) / 3., (self + other * 2.) / 3.)
                }
                fn section_end(&self, other: &Self) -> bool {
                    (self - other).abs() <= 1e-8
                }
            })*
        };
    }
    impl_trisect_unsigned!(u8 u16 u32 u64 usize);
    impl_trisect_signed!(i8 i16 i32 i64 isize);
    impl_trisect_float!(f32 f64);
}
#[codesnip::entry("ternary_search")]
/// like `(left..right).min_by_key(f)`
pub fn ternary_search<T, U>(mut f: impl FnMut(&T) -> U, mut left: T, mut right: T) -> T
where
    T: Trisect,
    U: PartialOrd,
{
    while !left.section_end(&right) {
        let (l, r) = left.next_section(&right);
        if f(&l) > f(&r) {
            left = l;
        } else {
            right = r;
        }
    }
    left
}
