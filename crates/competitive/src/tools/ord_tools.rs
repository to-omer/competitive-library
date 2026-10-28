#[cfg(verus_keep_ghost)]
use vstd::{prelude::*, std_specs::cmp::PartialOrdSpec};

#[cfg_attr(verus_keep_ghost, verus_verify)]
pub trait PartialOrdExt: Sized {
    fn chmin(&mut self, other: Self);
    fn chmax(&mut self, other: Self);
    fn minmax(self, other: Self) -> (Self, Self);
}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T> PartialOrdExt for T
where
    T: PartialOrd,
{
    #[inline]
    #[cfg_attr(verus_keep_ghost, verus_spec(
        ensures T::obeys_partial_cmp_spec() ==> *final(self) ==
            if (*old(self)).partial_cmp_spec(&other) == Some(std::cmp::Ordering::Greater) { other } else { *old(self) },
    ))]
    fn chmin(&mut self, other: Self) {
        if *self > other {
            *self = other;
        }
    }
    #[inline]
    #[cfg_attr(verus_keep_ghost, verus_spec(
        ensures T::obeys_partial_cmp_spec() ==> *final(self) ==
            if (*old(self)).partial_cmp_spec(&other) == Some(std::cmp::Ordering::Less) { other } else { *old(self) },
    ))]
    fn chmax(&mut self, other: Self) {
        if *self < other {
            *self = other;
        }
    }
    #[inline]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures T::obeys_partial_cmp_spec() ==> result ==
            if self.partial_cmp_spec(&other) == Some(std::cmp::Ordering::Less) { (self, other) } else { (other, self) },
    ))]
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

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
        fn extrema(values: &[i64]) -> (result: (i64, i64))
            requires values.len() > 0,
            ensures values@.contains(result.0), values@.contains(result.1),
                forall|i: int| 0 <= i < values.len() ==> result.0 <= #[trigger] values@[i] <= result.1,
        {
            let mut minimum = values[0];
            let mut maximum = values[0];
            for &value in cursor: values
                invariant values@.contains(minimum), values@.contains(maximum),
                    forall|i: int| 0 <= i < cursor.index() ==> minimum <= #[trigger] values@[i] <= maximum,
            {
                minimum.chmin(value);
                maximum.chmax(value);
            }
            let result = minimum.minmax(maximum);
            proof {
                assert(minimum <= values@[0] <= maximum);
                #[verifier::proof_note("Folding the actual chmin and chmax methods over every nonempty i64 slice, followed by minmax, returns attained global extrema that bound every input element; duplicate values and the full signed range are included")]
                assert(values@.contains(result.0) && values@.contains(result.1)
                    && forall|i: int| 0 <= i < values.len() ==> result.0 <= #[trigger] values@[i] <= result.1);
            }
            result
        }

        fn unordered_or_equal<T>(left: T, right: T)
            where T: PartialOrd,
            requires T::obeys_partial_cmp_spec(),
                left.partial_cmp_spec(&right) == None || left.partial_cmp_spec(&right) == Some(std::cmp::Ordering::Equal),
        {
            let mut minimum = &left;
            let mut maximum = &left;
            minimum.chmin(&right);
            maximum.chmax(&right);
            let pair = (&left).minmax(&right);
            proof {
                #[verifier::proof_note("For any type obeying its partial-comparison model, chmin and chmax retain the original value when the comparison is equal or incomparable, while minmax reverses the pair in those cases; the helpers do not assume all pairs are comparable")]
                assert(*minimum == left && *maximum == left && pair == (&right, &left));
            }
        }
    }
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
