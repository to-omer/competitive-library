/// Trait for max/min bounds
pub trait Bounded: Sized + PartialOrd {
    fn maximum() -> Self;
    fn minimum() -> Self;
    fn is_maximum(&self) -> bool {
        self == &Self::maximum()
    }
    fn is_minimum(&self) -> bool {
        self == &Self::minimum()
    }
    fn set_maximum(&mut self) {
        *self = Self::maximum()
    }
    fn set_minimum(&mut self) {
        *self = Self::minimum()
    }
}

macro_rules! bounded_num_impls {
    ($($t:ident)*) => {
        $(impl Bounded for $t {
            fn maximum() -> Self { std::$t::MAX }
            fn minimum() -> Self { std::$t::MIN }
        })*
    };
}
bounded_num_impls!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);

macro_rules! bounded_tuple_impls {
    (@impl $($T:ident)*) => {
        impl<$($T: Bounded),*> Bounded for ($($T,)*) {
            fn maximum() -> Self { ($(<$T as Bounded>::maximum(),)*) }
            fn minimum() -> Self { ($(<$T as Bounded>::minimum(),)*) }
        }
    };
    (@inner $($T:ident)*,) => {
        bounded_tuple_impls!(@impl $($T)*);
    };
    (@inner $($T:ident)*, $U:ident $($Rest:ident)*) => {
        bounded_tuple_impls!(@impl $($T)*);
        bounded_tuple_impls!(@inner $($T)* $U, $($Rest)*);
    };
    ($($T:ident)*) => {
        bounded_tuple_impls!(@inner , $($T)*);
    };
}
bounded_tuple_impls!(A B C D E F G H I J);

impl Bounded for bool {
    fn maximum() -> Self {
        true
    }
    fn minimum() -> Self {
        false
    }
}
impl<T> Bounded for Option<T>
where
    T: Bounded,
{
    fn maximum() -> Self {
        Some(<T as Bounded>::maximum())
    }
    fn minimum() -> Self {
        None
    }
}
impl<T> Bounded for std::cmp::Reverse<T>
where
    T: Bounded,
{
    fn maximum() -> Self {
        std::cmp::Reverse(<T as Bounded>::minimum())
    }
    fn minimum() -> Self {
        std::cmp::Reverse(<T as Bounded>::maximum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Reverse;

    fn assert_bounded<T: Bounded, I: Iterator<Item = T>>(iter: I) {
        assert!(T::minimum() <= T::maximum());
        for item in iter {
            assert!(T::minimum() <= item);
            assert!(item <= T::maximum());
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
