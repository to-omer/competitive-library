pub trait PartialOrdExt: Sized {
    fn chmin(&mut self, other: Self);
    fn chmax(&mut self, other: Self);
    fn minmax(self, other: Self) -> (Self, Self);
}
impl<T> PartialOrdExt for T
where
    T: PartialOrd,
{
    #[inline]
    fn chmin(&mut self, other: Self) {
        if *self > other {
            *self = other;
        }
    }
    #[inline]
    fn chmax(&mut self, other: Self) {
        if *self < other {
            *self = other;
        }
    }
    #[inline]
    fn minmax(self, other: Self) -> (Self, Self) {
        if self < other {
            (self, other)
        } else {
            (other, self)
        }
    }
}

#[macro_export]
macro_rules! min {
    ($l:expr) => { $l };
    ($l:expr,) => { $crate::min!($l) };
    ($l:expr, $r:expr) => { ($l).min($r) };
    ($l:expr, $r:expr,) => { $crate::min!($l, $r) };
    ($l:expr, $r:expr, $($t:tt)*) => { $crate::min!($crate::min!($l, $r), $($t)*) };
}

#[macro_export]
macro_rules! chmin {
    ($l:expr) => {};
    ($l:expr,) => {};
    ($l:expr, $r:expr) => {{ let r = $r; if $l > r { $l = r; } }};
    ($l:expr, $r:expr,) => { $crate::chmin!($l, $r) };
    ($l:expr, $r:expr, $($t:tt)*) => { $crate::chmin!($l, $r); $crate::chmin!($l, $($t)*) };
}

#[macro_export]
macro_rules! max {
    ($l:expr) => { $l };
    ($l:expr,) => { $crate::max!($l) };
    ($l:expr, $r:expr) => { ($l).max($r) };
    ($l:expr, $r:expr,) => { $crate::max!($l, $r) };
    ($l:expr, $r:expr, $($t:tt)*) => { $crate::max!($crate::max!($l, $r), $($t)*) };
}

#[macro_export]
macro_rules! chmax {
    ($l:expr) => {};
    ($l:expr,) => {};
    ($l:expr, $r:expr) => {{ let r = $r; if $l < r { $l = r; } }};
    ($l:expr, $r:expr,) => { $crate::chmax!($l, $r) };
    ($l:expr, $r:expr, $($t:tt)*) => { $crate::chmax!($l, $r); $crate::chmax!($l, $($t)*) };
}

#[macro_export]
macro_rules! minmax {
    ($($t:tt)*) => { ($crate::min!($($t)*), $crate::max!($($t)*)) };
}

#[cfg(test)]
mod tests {
    #![allow(clippy::eq_op)]
    use super::*;
    use crate::{chmax, chmin, max, min, minmax};

    macro_rules! assert_eq_f64 {
        ($l:expr, $r:expr) => { assert_eq_f64!($l, $r,) };
        ($l:expr, $r:expr, $($t:tt)*) => {
            ::std::assert!(($l - $r).abs() < ::std::f64::EPSILON, $($t)*);
        };
    }

    #[test]
    fn test_min() {
        assert_eq!(min!(1), 1);
        assert_eq!(min!(1, 2), 1);
        assert_eq_f64!(min!(4.0f64, 1., 2.), 1.0f64);
        assert_eq!(min!(4, 9, 2, 3,), 2);
    }

    #[test]
    fn test_chmin() {
        let mut x = 100;
        chmin!(x, 101);
        assert_eq!(x, 100);
        chmin!(x, 91, 78);
        assert_eq!(x, 78);
        chmin!(x, 61, 42, 51);
        assert_eq!(x, 42);

        let mut v = vec![31, 12];
        chmin!(v[0], v[1], 14);
        assert_eq!(v[0], v[1]);
    }

    #[test]
    fn test_max() {
        assert_eq!(max!(1), 1);
        assert_eq!(max!(1, 2), 2);
        assert_eq_f64!(max!(4.0f64, 1., 2.), 4.0f64);
        assert_eq!(max!(4, 9, 2, 3,), 9);
    }

    #[test]
    fn test_chmax() {
        let mut x = 100;
        chmax!(x, 91);
        assert_eq!(x, 100);
        chmax!(x, 191, 178);
        assert_eq!(x, 191);
        chmax!(x, 261, 242, 251);
        assert_eq!(x, 261);

        let mut v = vec![31, 42];
        chmax!(v[0], v[1], 14);
        assert_eq!(v[0], v[1]);
    }

    #[test]
    fn test_minmax() {
        assert_eq!(minmax!(1), (1, 1));
        assert_eq!(minmax!(1, 2), (1, 2));
        assert_eq_f64!(minmax!(4.0f64, 1., 2.).0, 1.0f64);
        assert_eq_f64!(minmax!(4.0f64, 1., 2.).1, 4.0f64);
        assert_eq!(minmax!(4, 9, 2, 3,), (2, 9));
    }

    #[test]
    fn test_partial_ord_ext() {
        let mut x = 100;
        x.chmin(91);
        assert_eq!(x, 91);
        x.chmax(101);
        assert_eq!(x, 101);
        assert_eq!(100.minmax(91), (91, 100));
    }
}
