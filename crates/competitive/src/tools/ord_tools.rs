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
    use super::*;
    use crate::tools::Xorshift;
    use std::array;

    #[test]
    fn test_order_operations() {
        let mut rng = Xorshift::default();
        for _ in 0..10_000 {
            let values: [i32; 4] = array::from_fn(|_| rng.random(-100..=100));
            let [a, b, c, d] = values;
            let lo = *values.iter().min().unwrap();
            let hi = *values.iter().max().unwrap();
            assert_eq!(min!(a), a);
            assert_eq!(max!(a), a);
            assert_eq!(min!(a, b), a.min(b));
            assert_eq!(max!(a, b), a.max(b));
            assert_eq!(min!(a, b, c, d,), lo);
            assert_eq!(max!(a, b, c, d,), hi);
            assert_eq!(minmax!(a, b, c, d), (lo, hi));
            let mut x = a;
            chmin!(x, b, c, d);
            assert_eq!(x, lo);
            let mut x = a;
            chmax!(x, b, c, d);
            assert_eq!(x, hi);
            let mut x = a;
            x.chmin(b);
            assert_eq!(x, a.min(b));
            x.chmax(c);
            assert_eq!(x, a.min(b).max(c));
            assert_eq!(a.minmax(b), (a.min(b), a.max(b)));
            assert_eq!(min!(a as f64, b as f64, c as f64, d as f64), lo as f64);
            assert_eq!(max!(a as f64, b as f64, c as f64, d as f64), hi as f64);
        }
    }
}
