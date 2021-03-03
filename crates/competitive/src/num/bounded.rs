/// Trait for max/min bounds
pub trait Bounded: PartialOrd {
    const MAX: Self;
    const MIN: Self;
}

macro_rules! bounded_num_impls {
    ($($t:ident)*) => {
        $(impl Bounded for $t {
            const MAX: Self = std::$t::MAX;
            const MIN: Self = std::$t::MIN;
        })*
    };
}
bounded_num_impls!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);

macro_rules! bounded_tuple_impls {
    ($($t:ident)*) => {
        impl<$($t: Bounded),*> Bounded for ($($t,)*) {
            const MAX: Self = ($(<$t as Bounded>::MAX,)*);
            const MIN: Self = ($(<$t as Bounded>::MIN,)*);
        }
    }
}
bounded_tuple_impls!();
bounded_tuple_impls!(A);
bounded_tuple_impls!(A B);
bounded_tuple_impls!(A B C);
bounded_tuple_impls!(A B C D);
bounded_tuple_impls!(A B C D E);
bounded_tuple_impls!(A B C D E F);
bounded_tuple_impls!(A B C D E F G);
bounded_tuple_impls!(A B C D E F G H);
bounded_tuple_impls!(A B C D E F G H I);
bounded_tuple_impls!(A B C D E F G H I J);

impl Bounded for bool {
    const MAX: Self = true;
    const MIN: Self = false;
}
impl<T> Bounded for Option<T>
where
    T: Bounded,
{
    const MAX: Self = Some(<T as Bounded>::MAX);
    const MIN: Self = None;
}
impl<T> Bounded for std::cmp::Reverse<T>
where
    T: Bounded,
{
    const MAX: Self = std::cmp::Reverse(<T as Bounded>::MIN);
    const MIN: Self = std::cmp::Reverse(<T as Bounded>::MAX);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Reverse;

    fn assert_bounded<T: Bounded + Copy, I: Iterator<Item = T>>(iter: I) {
        assert!(T::MIN <= T::MAX);
        for item in iter {
            assert!(T::MIN <= item);
            assert!(item <= T::MAX);
        }
    }

    #[test]
    fn test_num_bounded() {
        assert_bounded([0u32, 1, 2, !0].iter().cloned());
        assert_bounded([0u64, 1, 2, !0].iter().cloned());
        assert_bounded([0usize, 1, 2, !0].iter().cloned());
        assert_bounded([0i32, 1, 2, !0].iter().cloned());
        assert_bounded([0i64, 1, 2, !0].iter().cloned());
        assert_bounded([0isize, 1, 2, !0].iter().cloned());
        assert_bounded([false, true].iter().cloned());
    }

    #[test]
    fn test_tuple_bounded() {
        assert_bounded([(1, 0, 3)].iter().cloned());
        assert_bounded([((), (1,), (2, 3))].iter().cloned());
    }

    #[test]
    fn test_option_bounded() {
        assert_bounded([None, Some((false, 3))].iter().cloned());
    }

    #[test]
    fn test_reverse_bounded() {
        assert_bounded([Reverse(0), Reverse(1), Reverse(!0)].iter().cloned());
    }
}
