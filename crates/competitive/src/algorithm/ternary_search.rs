/// ternary search helper
pub trait Trisect: Clone {
    /// Divide into 3 sections
    fn next_section(&self, other: &Self) -> (Self, Self);
    /// the end condition of ternary search
    fn section_end(&self, other: &Self) -> bool;
    /// middle point of section
    fn middle(&self, other: &Self) -> Self;
}

macro_rules! impl_trisect_unsigned {
    ($($t:ty)*) => {
        $(impl Trisect for $t {
            fn next_section(&self, other: &Self) -> (Self, Self) {
                ((self * 2 + other) / 3, (self + other * 2) / 3)
            }
            fn section_end(&self, other: &Self) -> bool {
                &(self + 2) >= other
            }
            fn middle(&self, other: &Self) -> Self {
                (self + other) / 2
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
                &(self + 2) >= other
            }
            fn middle(&self, other: &Self) -> Self {
                (self + other) / 2
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
                &(self +  1e-8) >= other
            }
            fn middle(&self, other: &Self) -> Self {
                (self + other) / 2.
            }
        })*
    };
}
impl_trisect_unsigned!(u8 u16 u32 u64 usize);
impl_trisect_signed!(i8 i16 i32 i64 isize);
impl_trisect_float!(f32 f64);

/// like `(left..=right).min_by_key(f)`
///
/// `f` should be strictly concave up
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
    let mid = left.middle(&right);
    let mut res = (f(&left), left);
    for x in [mid, right].iter().cloned() {
        let y = f(&x);
        if res.0 > y {
            res = (y, x);
        }
    }
    res.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_search() {
        for p in -20..=20 {
            assert_eq!(
                ternary_search(|&x| 10 * (x - p).pow(2) + 5, -10i64, 10),
                p.clamp(-10, 10)
            );
        }
    }
}
