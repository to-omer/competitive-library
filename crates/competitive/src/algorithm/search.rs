//! binary / ternary search

/// binary search helper
#[codesnip::entry("binary_search")]
pub trait Bisect: Copy {
    /// return between two elements
    fn halve(self, other: Self) -> Self;
    /// the end condition of binary search
    fn section_end(self, other: Self) -> bool;
}

#[codesnip::entry("binary_search")]
mod bisect_impl {
    use super::*;
    macro_rules! impl_bisect_unsigned {
        ($($t:ty)*) => {
            $(impl Bisect for $t {
                fn halve(self, other: Self) -> Self {
                    if self > other {
                        other + (self - other) / 2
                    } else {
                        self + (other - self) / 2
                    }
                }
                fn section_end(self, other: Self) -> bool {
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
                fn halve(self, other: Self) -> Self {
                    (self + other) / 2
                }
                fn section_end(self, other: Self) -> bool {
                    (self - other).abs() <= 1
                }
            })*
        };
    }
    macro_rules! impl_bisect_float {
        ($($t:ty)*) => {
            $(impl Bisect for $t {
                fn halve(self, other: Self) -> Self {
                    (self + other) / 2.
                }
                fn section_end(self, other: Self) -> bool {
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
pub fn binary_search<T>(mut f: impl FnMut(T) -> bool, mut ok: T, mut err: T) -> T
where
    T: Bisect,
{
    while !ok.section_end(err) {
        let m = ok.halve(err);
        if f(m) {
            ok = m;
        } else {
            err = m;
        }
    }
    ok
}

#[codesnip::entry("binary_search")]
/// binary search for sorted slice
pub trait SliceBisectExt<T: Bisect + Ord> {
    /// Returns the first index with elements greater than or equal to x.
    /// if not found, returns `len()`.
    fn lower_bound(&self, x: T) -> usize;
    /// Returns the first index with elements greater than x.
    /// if not found, returns `len()`.
    fn upper_bound(&self, x: T) -> usize;
}
#[codesnip::entry("binary_search")]
impl<T: Bisect + Ord> SliceBisectExt<T> for [T] {
    fn lower_bound(&self, x: T) -> usize {
        binary_search(|i| self[i as usize] >= x, self.len() as i64, -1) as usize
    }
    fn upper_bound(&self, x: T) -> usize {
        binary_search(|i| self[i as usize] > x, self.len() as i64, -1) as usize
    }
}

#[codesnip::entry("binary_search")]
/// Count the number of elements that meet the condition in `range`.
pub fn count_monotone<T, R>(mut f: impl FnMut(T) -> bool, mut range: R) -> usize
where
    T: Bisect,
    R: std::ops::RangeBounds<T> + DoubleEndedIterator<Item = T> + ExactSizeIterator,
    std::ops::Range<T>: ExactSizeIterator,
{
    let length = range.len();
    match (range.next(), range.next_back()) {
        (None, None) => 0,
        (None, Some(right)) => f(right) as usize,
        (Some(left), None) => f(left) as usize,
        (Some(left), Some(right)) => match (f(left), f(right)) {
            (false, false) => 0,
            (false, true) => {
                let ok_l = binary_search(f, right, left);
                length.wrapping_sub((left..ok_l).len())
            }
            (true, false) => {
                let ok_r = binary_search(f, left, right);
                length.wrapping_sub((ok_r..right).len())
            }
            (true, true) => length,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_binary_search() {
        let v = vec![0, 1, 1, 1, 2, 2, 3, 4, 7, 8];
        assert_eq!(binary_search(&|x| v[x] >= 1, v.len(), 0), 1);
        assert_eq!(binary_search(&|x| v[x] >= 2, v.len(), 0), 4);
        assert_eq!(binary_search(&|x| v[x] >= 3, v.len(), 0), 6);
        assert_eq!(binary_search(&|x| v[x] <= 1, 0, v.len()), 3);
        assert_eq!(binary_search(&|x| v[x] <= 2, 0, v.len()), 5);
        assert_eq!(binary_search(&|x| v[x] <= 3, 0, v.len()), 6);

        assert_eq!(
            binary_search(&|x: i64| v[x as usize] as i64 <= -1, -1, v.len() as i64),
            -1
        );

        let sq2 = binary_search(&|x| x * x <= 2., 1., 4.);
        let expect = 1.414_213_562_73;
        assert!(expect - 1e-8 <= sq2 && sq2 <= expect + 1e-8);
    }

    #[test]
    fn test_lower_bound() {
        let v = vec![0i64, 1, 1, 1, 2, 2, 3, 4, 7, 8];
        assert_eq!(v.lower_bound(-1), 0);
        assert_eq!(v.lower_bound(0), 0);
        assert_eq!(v.lower_bound(1), 1);
        assert_eq!(v.lower_bound(2), 4);
        assert_eq!(v.lower_bound(3), 6);
    }

    #[test]
    fn test_upper_bound() {
        let v = vec![0i64, 1, 1, 1, 2, 2, 3, 4, 7, 8];
        assert_eq!(v.upper_bound(-1), 0);
        assert_eq!(v.upper_bound(0), 1);
        assert_eq!(v.upper_bound(1), 4);
        assert_eq!(v.upper_bound(2), 6);
        assert_eq!(v.upper_bound(3), 7);
    }

    #[test]
    fn test_count_monotone() {
        let v = vec![0i64, 1, 1, 1, 2, 2, 3, 4, 7, 8];
        assert_eq!(count_monotone(|i| v[i] >= 0, 0..v.len()), 10);
        assert_eq!(count_monotone(|i| v[i] >= 1, 0..v.len()), 9);
        assert_eq!(count_monotone(|i| v[i] >= 2, 0..v.len()), 6);
        assert_eq!(count_monotone(|i| v[i] >= 3, 0..v.len()), 4);
        assert_eq!(count_monotone(|i| v[i] >= 5, 0..v.len()), 2);
        assert_eq!(count_monotone(|i| v[i] >= 10, 0..v.len()), 0);

        assert_eq!(count_monotone(|i| v[i] < 0, 0..v.len()), 0);
        assert_eq!(count_monotone(|i| v[i] < 1, 0..v.len()), 1);
        assert_eq!(count_monotone(|i| v[i] < 2, 0..v.len()), 4);
        assert_eq!(count_monotone(|i| v[i] < 3, 0..v.len()), 6);
        assert_eq!(count_monotone(|i| v[i] < 5, 0..v.len()), 8);
        assert_eq!(count_monotone(|i| v[i] < 10, 0..v.len()), 10);
    }
}

#[codesnip::entry("ternary_search")]
/// ternary search helper
pub trait Trisect: Copy {
    /// Divide into 3 sections
    fn next_section(self, other: Self) -> (Self, Self);
    /// the end condition of ternary search
    fn section_end(self, other: Self) -> bool;
}
#[codesnip::entry("ternary_search")]
mod trisect_impl {
    use super::*;
    macro_rules! impl_trisect_unsigned {
        ($($t:ty)*) => {
            $(impl Trisect for $t {
                fn next_section(self, other: Self) -> (Self, Self) {
                    ((self * 2 + other) / 3, (self + other * 2) / 3)
                }
                fn section_end(self, other: Self) -> bool {
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
                fn next_section(self, other: Self) -> (Self, Self) {
                    ((self * 2 + other) / 3, (self + other * 2) / 3)
                }
                fn section_end(self, other: Self) -> bool {
                    (self - other).abs() <= 1
                }
            })*
        };
    }
    macro_rules! impl_trisect_float {
        ($($t:ty)*) => {
            $(impl Trisect for $t {
                fn next_section(self, other: Self) -> (Self, Self) {
                    ((self * 2. + other) / 3., (self + other * 2.) / 3.)
                }
                fn section_end(self, other: Self) -> bool {
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
pub fn ternary_search<T, U>(mut f: impl FnMut(T) -> U, mut left: T, mut right: T) -> T
where
    T: Trisect,
    U: PartialOrd,
{
    while !left.section_end(right) {
        let (l, r) = left.next_section(right);
        if f(l) > f(r) {
            left = l;
        } else {
            right = r;
        }
    }
    left
}
