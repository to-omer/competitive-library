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

macro_rules! impl_bounded_num {
    ($($t:ident)*) => {
        $(impl Bounded for $t {
            fn maximum() -> Self { $t::MAX }
            fn minimum() -> Self { $t::MIN }
        })*
    };
}
impl_bounded_num!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);

macro_rules! impl_bounded_tuple {
    (@impl $($T:ident)*) => {
        impl<$($T: Bounded),*> Bounded for ($($T,)*) {
            fn maximum() -> Self { ($(<$T as Bounded>::maximum(),)*) }
            fn minimum() -> Self { ($(<$T as Bounded>::minimum(),)*) }
        }
    };
    (@inner $($T:ident)*,) => {
        impl_bounded_tuple!(@impl $($T)*);
    };
    (@inner $($T:ident)*, $U:ident $($Rest:ident)*) => {
        impl_bounded_tuple!(@impl $($T)*);
        impl_bounded_tuple!(@inner $($T)* $U, $($Rest)*);
    };
    ($T:ident $($Rest:ident)*) => {
        impl_bounded_tuple!(@inner $T, $($Rest)*);
    };
}
impl_bounded_tuple!(A B C D E F G H I J);

impl Bounded for () {
    fn maximum() -> Self {}
    fn minimum() -> Self {}
}
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
    use crate::tools::Xorshift;
    use std::cmp::Reverse;

    fn assert_bounded<T: Bounded>(item: T) {
        assert!(T::minimum() <= item);
        assert!(item <= T::maximum());
    }

    #[test]
    fn test_bounded() {
        let mut rng = Xorshift::default();
        let mut cases = Vec::new();
        for a in [0u32, 1, u32::MAX] {
            for b in [i64::MIN, -1, 0, 1, i64::MAX] {
                for c in [0usize, 1, usize::MAX] {
                    for d in [false, true] {
                        cases.push((a, b, c, d));
                    }
                }
            }
        }
        cases.extend((0..10_000).map(|_| {
            (
                rng.random(..),
                rng.random(..),
                rng.random(..),
                rng.random(0..2) == 0,
            )
        }));
        for (a, b, c, d) in cases {
            assert_bounded(a);
            assert_bounded(b);
            assert_bounded(c);
            assert_bounded(a as i32);
            assert_bounded(b as u64);
            assert_bounded(c as isize);
            assert_bounded(d);
            assert_bounded((a, b, c));
            assert_bounded(((), (a,), (b, c)));
            assert_bounded(if d { Some((d, b)) } else { None });
            assert_bounded(Reverse(b));
        }
    }
}
