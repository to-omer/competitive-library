//! binary operaions

use super::{Bounded, One, Zero, magma::*};

#[codesnip::entry("MaxOperation")]
pub use self::max_operation_impl::MaxOperation;
#[codesnip::entry("MaxOperation", include("algebra", "bounded"))]
mod max_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    /// binary operation to select larger element
    pub struct MaxOperation<T>
    where
        T: Clone + Ord + Bounded,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for MaxOperation<T>
    where
        T: Clone + Ord + Bounded,
    {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.max(y).clone()
        }
    }
    impl<T> Unital for MaxOperation<T>
    where
        T: Clone + Ord + Bounded,
    {
        #[inline]
        fn unit() -> Self::T {
            <T as Bounded>::minimum()
        }
    }
    impl<T> Associative for MaxOperation<T> where T: Clone + Ord + Bounded {}
    impl<T> Commutative for MaxOperation<T> where T: Clone + Ord + Bounded {}
    impl<T> Idempotent for MaxOperation<T> where T: Clone + Ord + Bounded {}

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_max_operation() {
            type M = MaxOperation<i32>;
            assert_eq!(M::operate(&1, &2), 2);
            assert_eq!(M::operate(&2, &1), 2);
            assert_eq!(M::operate(&2, &2), 2);
            for a in -10..=10 {
                assert!(M::check_unital(&a));
                assert!(M::check_idempotent(&a));
                for b in -10..=10 {
                    assert!(M::check_commutative(&a, &b));
                    for c in -10..=10 {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("MinOperation")]
pub use self::min_operation_impl::MinOperation;
#[codesnip::entry("MinOperation", include("algebra", "bounded"))]
mod min_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    /// binary operation to select smaller element
    pub struct MinOperation<T>
    where
        T: Clone + Ord + Bounded,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for MinOperation<T>
    where
        T: Clone + Ord + Bounded,
    {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.min(y).clone()
        }
    }
    impl<T> Unital for MinOperation<T>
    where
        T: Clone + Ord + Bounded,
    {
        #[inline]
        fn unit() -> Self::T {
            <T as Bounded>::maximum()
        }
    }
    impl<T> Associative for MinOperation<T> where T: Clone + Ord + Bounded {}
    impl<T> Commutative for MinOperation<T> where T: Clone + Ord + Bounded {}
    impl<T> Idempotent for MinOperation<T> where T: Clone + Ord + Bounded {}

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_min_operation() {
            type M = MinOperation<i32>;
            assert_eq!(M::operate(&1, &2), 1);
            assert_eq!(M::operate(&2, &1), 1);
            assert_eq!(M::operate(&2, &2), 2);
            for a in -10..=10 {
                assert!(M::check_unital(&a));
                assert!(M::check_idempotent(&a));
                for b in -10..=10 {
                    assert!(M::check_commutative(&a, &b));
                    for c in -10..=10 {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("FirstOperation")]
pub use self::first_operation_impl::FirstOperation;
#[codesnip::entry("FirstOperation", include("algebra"))]
mod first_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    /// retain the first element
    pub struct FirstOperation<T>
    where
        T: Clone,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for FirstOperation<T>
    where
        T: Clone,
    {
        type T = Option<T>;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.as_ref().or(y.as_ref()).cloned()
        }
    }
    impl<T> Unital for FirstOperation<T>
    where
        T: Clone,
    {
        #[inline]
        fn unit() -> Self::T {
            None
        }
    }
    impl<T> Associative for FirstOperation<T> where T: Clone {}
    impl<T> Idempotent for FirstOperation<T> where T: Clone {}

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_first_operation() {
            type M = FirstOperation<i32>;
            assert_eq!(M::operate(&Some(1), &Some(2)), Some(1));
            assert_eq!(M::operate(&Some(2), &Some(1)), Some(2));
            assert_eq!(M::operate(&Some(1), &None), Some(1));
            assert_eq!(M::operate(&None, &Some(1)), Some(1));
            assert_eq!(M::operate(&None, &None), None);
            let iter = [Some(1), Some(2), Some(3), None];
            for a in iter {
                assert!(M::check_unital(&a));
                assert!(M::check_idempotent(&a));
                for b in iter {
                    for c in iter {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("LastOperation")]
pub use self::last_operation_impl::LastOperation;
#[codesnip::entry("LastOperation", include("algebra"))]
mod last_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    /// retain the last element
    pub struct LastOperation<T>
    where
        T: Clone,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for LastOperation<T>
    where
        T: Clone,
    {
        type T = Option<T>;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            y.as_ref().or(x.as_ref()).cloned()
        }
    }
    impl<T> Unital for LastOperation<T>
    where
        T: Clone,
    {
        #[inline]
        fn unit() -> Self::T {
            None
        }
    }
    impl<T> Associative for LastOperation<T> where T: Clone {}
    impl<T> Idempotent for LastOperation<T> where T: Clone {}

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_last_operation() {
            type M = LastOperation<i32>;
            assert_eq!(M::operate(&Some(1), &Some(2)), Some(2));
            assert_eq!(M::operate(&Some(2), &Some(1)), Some(1));
            assert_eq!(M::operate(&Some(1), &None), Some(1));
            assert_eq!(M::operate(&None, &Some(1)), Some(1));
            assert_eq!(M::operate(&None, &None), None);
            let iter = [Some(1), Some(2), Some(3), None];
            for a in iter {
                assert!(M::check_unital(&a));
                assert!(M::check_idempotent(&a));
                for b in iter {
                    for c in iter {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("AdditiveOperation")]
pub use self::additive_operation_impl::AdditiveOperation;
#[codesnip::entry("AdditiveOperation", include("algebra", "zero_one"))]
mod additive_operation_impl {
    use super::*;
    use std::{
        marker::PhantomData,
        ops::{Add, Neg, Sub},
    };
    /// $+$
    pub struct AdditiveOperation<T>
    where
        T: Clone + Zero + Add<Output = T>,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for AdditiveOperation<T>
    where
        T: Clone + Zero + Add<Output = T>,
    {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() + y.clone()
        }
    }
    impl<T> Unital for AdditiveOperation<T>
    where
        T: Clone + Zero + Add<Output = T>,
    {
        #[inline]
        fn unit() -> Self::T {
            Zero::zero()
        }
    }
    impl<T> Associative for AdditiveOperation<T> where T: Clone + Zero + Add<Output = T> {}
    impl<T> Commutative for AdditiveOperation<T> where T: Clone + Zero + Add<Output = T> {}
    impl<T> Invertible for AdditiveOperation<T>
    where
        T: Clone + Zero + Add<Output = T> + Sub<Output = T> + Neg<Output = T>,
    {
        #[inline]
        fn inverse(x: &Self::T) -> Self::T {
            -x.clone()
        }
        #[inline]
        fn rinv_operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() - y.clone()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_additive_operation() {
            type M = AdditiveOperation<i32>;
            assert_eq!(M::operate(&1, &2), 3);
            assert_eq!(M::operate(&2, &1), 3);
            assert_eq!(M::operate(&1, &0), 1);
            assert_eq!(M::operate(&0, &1), 1);
            assert_eq!(M::operate(&0, &0), 0);
            for a in -10..=10 {
                assert!(M::check_unital(&a));
                assert!(M::check_invertible(&a));
                for b in -10..=10 {
                    assert!(M::check_commutative(&a, &b));
                    for c in -10..=10 {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("MultiplicativeOperation")]
pub use self::multiplicative_operation_impl::MultiplicativeOperation;
#[codesnip::entry("MultiplicativeOperation", include("algebra", "zero_one"))]
mod multiplicative_operation_impl {
    use super::*;
    use std::{
        marker::PhantomData,
        ops::{Div, Mul},
    };
    /// $\times$
    pub struct MultiplicativeOperation<T>
    where
        T: Clone + One + Mul<Output = T>,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for MultiplicativeOperation<T>
    where
        T: Clone + One + Mul<Output = T>,
    {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() * y.clone()
        }
    }
    impl<T> Unital for MultiplicativeOperation<T>
    where
        T: Clone + One + Mul<Output = T>,
    {
        #[inline]
        fn unit() -> Self::T {
            One::one()
        }
    }
    impl<T> Associative for MultiplicativeOperation<T> where T: Clone + One + Mul<Output = T> {}
    impl<T> Commutative for MultiplicativeOperation<T> where T: Clone + One + Mul<Output = T> {}
    impl<T> Invertible for MultiplicativeOperation<T>
    where
        T: Clone + One + Mul<Output = T> + Div<Output = T>,
    {
        #[inline]
        fn inverse(x: &Self::T) -> Self::T {
            Self::unit().div(x.clone())
        }
        #[inline]
        fn rinv_operate(x: &Self::T, y: &Self::T) -> Self::T {
            (x.clone()).div(y.clone())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::num::mint_basic::MInt998244353;

        #[test]
        fn test_multiplicative_operation() {
            type MInt = MInt998244353;
            type M = MultiplicativeOperation<MInt>;
            assert_eq!(M::operate(&MInt::new(1), &MInt::new(2)), MInt::new(2));
            assert_eq!(M::operate(&MInt::new(2), &MInt::new(1)), MInt::new(2));
            assert_eq!(M::operate(&MInt::new(1), &MInt::new(1)), MInt::new(1));
            assert_eq!(M::operate(&MInt::new(1), &MInt::new(0)), MInt::new(0));
            assert_eq!(M::operate(&MInt::new(0), &MInt::new(1)), MInt::new(0));
            assert_eq!(M::operate(&MInt::new(0), &MInt::new(0)), MInt::new(0));
            let iter = (-10..=10).map(MInt::from);
            for a in iter.clone() {
                assert!(M::check_unital(&a));
                if !a.is_zero() {
                    assert!(M::check_invertible(&a));
                }
                for b in iter.clone() {
                    assert!(M::check_commutative(&a, &b));
                    for c in iter.clone() {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("LinearOperation")]
pub use self::linear_operation_impl::LinearOperation;
#[codesnip::entry("LinearOperation", include("algebra", "zero_one"))]
mod linear_operation_impl {
    use super::*;
    use std::{
        marker::PhantomData,
        ops::{Add, Div, Mul, Neg, Sub},
    };
    /// $(a, b) \circ (c, d) = \lambda x. c \times (a \times x + b) + d$
    pub struct LinearOperation<T>
    where
        T: Clone + Zero + Add<Output = T> + One + Mul<Output = T>,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> LinearOperation<T>
    where
        T: Clone + Zero + Add<Output = T> + One + Mul<Output = T>,
    {
        pub fn apply(f: &(T, T), x: &T) -> T {
            f.0.clone() * x.clone() + f.1.clone()
        }
    }
    impl<T> Magma for LinearOperation<T>
    where
        T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>,
    {
        type T = (T, T);
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            (
                y.0.clone() * x.0.clone(),
                y.0.clone() * x.1.clone() + y.1.clone(),
            )
        }
    }
    impl<T> Unital for LinearOperation<T>
    where
        T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>,
    {
        #[inline]
        fn unit() -> Self::T {
            (One::one(), Zero::zero())
        }
    }
    impl<T> Associative for LinearOperation<T> where
        T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>
    {
    }
    impl<T> Invertible for LinearOperation<T>
    where
        T: Clone
            + Zero
            + One
            + Add<Output = T>
            + Sub<Output = T>
            + Neg<Output = T>
            + Mul<Output = T>
            + Div<Output = T>,
    {
        fn inverse(x: &Self::T) -> Self::T {
            let y = <T as One>::one().div(x.0.clone());
            (y.clone(), -y.mul(x.1.clone()))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::num::mint_basic::MInt998244353;

        #[test]
        fn test_linear_operation() {
            type MInt = MInt998244353;
            type M = LinearOperation<MInt>;
            let iter = (-5..=5).flat_map(|x| (-5..=5).map(move |y| (MInt::from(x), MInt::from(y))));
            for a in iter.clone() {
                assert!(M::check_unital(&a));
                if !a.0.is_zero() {
                    assert!(M::check_invertible(&a));
                }
                for b in iter.clone() {
                    for c in iter.clone() {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                    for x in (-5..=5).map(MInt::from) {
                        assert_eq!(
                            M::apply(&M::operate(&a, &b), &x),
                            M::apply(&b, &M::apply(&a, &x))
                        );
                    }
                }
            }
        }
    }
}

#[codesnip::entry("BitAndOperation")]
pub use self::bitand_operation_impl::{BitAndIdentity, BitAndOperation};
#[codesnip::entry("BitAndOperation", include("algebra"))]
mod bitand_operation_impl {
    use super::*;
    use std::{marker::PhantomData, ops::BitAnd};
    /// &
    pub struct BitAndOperation<T>
    where
        T: Clone + BitAndIdentity,
    {
        _marker: PhantomData<fn() -> T>,
    }
    pub trait BitAndIdentity: Sized + BitAnd<Output = Self> {
        fn all_one() -> Self;
    }
    #[macro_export]
    macro_rules! impl_bitand_identity {
        ([$($wh:tt)*], $t:ty, $all_one:expr) => {
            impl<$($wh)*> BitAndIdentity for $t {
                #[inline]
                fn all_one() -> Self {
                    $all_one
                }
            }
        };
        ($t:ty, $all_one:expr) => {
            impl BitAndIdentity for $t {
                #[inline]
                fn all_one() -> Self {
                    $all_one
                }
            }
        };
    }
    impl_bitand_identity!(bool, true);
    impl_bitand_identity!(usize, usize::MAX);
    impl_bitand_identity!(u8, u8::MAX);
    impl_bitand_identity!(u16, u16::MAX);
    impl_bitand_identity!(u32, u32::MAX);
    impl_bitand_identity!(u64, u64::MAX);
    impl_bitand_identity!(u128, u128::MAX);
    impl_bitand_identity!(isize, -1);
    impl_bitand_identity!(i8, -1);
    impl_bitand_identity!(i16, -1);
    impl_bitand_identity!(i32, -1);
    impl_bitand_identity!(i64, -1);
    impl_bitand_identity!(i128, -1);
    impl<T> Magma for BitAndOperation<T>
    where
        T: Clone + BitAndIdentity,
    {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() & y.clone()
        }
    }
    impl<T> Unital for BitAndOperation<T>
    where
        T: Clone + BitAndIdentity,
    {
        #[inline]
        fn unit() -> Self::T {
            BitAndIdentity::all_one()
        }
    }
    impl<T> Associative for BitAndOperation<T> where T: Clone + BitAndIdentity {}
    impl<T> Commutative for BitAndOperation<T> where T: Clone + BitAndIdentity {}
    impl<T> Idempotent for BitAndOperation<T> where T: Clone + BitAndIdentity {}

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_bitand_operation() {
            macro_rules! impl_test_bitand_operation {
                ($ty:ty, $array:expr) => {{
                    type M = BitAndOperation<$ty>;
                    for a in $array {
                        assert!(M::check_unital(&a));
                        assert!(M::check_idempotent(&a));
                        for b in $array {
                            assert!(M::check_commutative(&a, &b));
                            for c in $array {
                                assert!(M::check_associative(&a, &b, &c));
                            }
                        }
                    }
                }};
            }
            impl_test_bitand_operation!(bool, [true, false]);
            impl_test_bitand_operation!(usize, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitand_operation!(u8, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitand_operation!(u16, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitand_operation!(u32, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitand_operation!(u64, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitand_operation!(u128, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitand_operation!(isize, [0, 1, 2, 3, 4, 5, -1, -2, isize::MIN, isize::MAX]);
            impl_test_bitand_operation!(i8, [0, 1, 2, 3, 4, 5, -1, -2, i8::MIN, i8::MAX]);
            impl_test_bitand_operation!(i16, [0, 1, 2, 3, 4, 5, -1, -2, i16::MIN, i16::MAX]);
            impl_test_bitand_operation!(i32, [0, 1, 2, 3, 4, 5, -1, -2, i32::MIN, i32::MAX]);
            impl_test_bitand_operation!(i64, [0, 1, 2, 3, 4, 5, -1, -2, i64::MIN, i64::MAX]);
            impl_test_bitand_operation!(i128, [0, 1, 2, 3, 4, 5, -1, -2, i128::MIN, i128::MAX]);
        }
    }
}

#[codesnip::entry("BitOrOperation")]
pub use self::bitor_operation_impl::{BitOrIdentity, BitOrOperation};
#[codesnip::entry("BitOrOperation", include("algebra"))]
mod bitor_operation_impl {
    use super::*;
    use std::{marker::PhantomData, ops::BitOr};
    /// |
    pub struct BitOrOperation<T>
    where
        T: Clone + BitOrIdentity,
    {
        _marker: PhantomData<fn() -> T>,
    }
    pub trait BitOrIdentity: Sized + BitOr<Output = Self> {
        fn all_zero() -> Self;
    }
    #[macro_export]
    macro_rules! impl_bitor_identity {
        ([$($wh:tt)*], $t:ty, $all_zero:expr) => {
            impl<$($wh)*> BitOrIdentity for $t {
                #[inline]
                fn all_zero() -> Self {
                    $all_zero
                }
            }
        };
        ($t:ty, $all_zero:expr) => {
            impl BitOrIdentity for $t {
                #[inline]
                fn all_zero() -> Self {
                    $all_zero
                }
            }
        };
    }
    impl_bitor_identity!(bool, false);
    impl_bitor_identity!(usize, 0);
    impl_bitor_identity!(u8, 0);
    impl_bitor_identity!(u16, 0);
    impl_bitor_identity!(u32, 0);
    impl_bitor_identity!(u64, 0);
    impl_bitor_identity!(u128, 0);
    impl_bitor_identity!(isize, 0);
    impl_bitor_identity!(i8, 0);
    impl_bitor_identity!(i16, 0);
    impl_bitor_identity!(i32, 0);
    impl_bitor_identity!(i64, 0);
    impl_bitor_identity!(i128, 0);
    impl<T> Magma for BitOrOperation<T>
    where
        T: Clone + BitOrIdentity,
    {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() | y.clone()
        }
    }
    impl<T> Unital for BitOrOperation<T>
    where
        T: Clone + BitOrIdentity,
    {
        #[inline]
        fn unit() -> Self::T {
            BitOrIdentity::all_zero()
        }
    }
    impl<T> Associative for BitOrOperation<T> where T: Clone + BitOrIdentity {}
    impl<T> Commutative for BitOrOperation<T> where T: Clone + BitOrIdentity {}
    impl<T> Idempotent for BitOrOperation<T> where T: Clone + BitOrIdentity {}

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_bitor_operation() {
            macro_rules! impl_test_bitor_operation {
                ($ty:ty, $array:expr) => {{
                    type M = BitOrOperation<$ty>;
                    for a in $array {
                        assert!(M::check_unital(&a));
                        assert!(M::check_idempotent(&a));
                        for b in $array {
                            assert!(M::check_commutative(&a, &b));
                            for c in $array {
                                assert!(M::check_associative(&a, &b, &c));
                            }
                        }
                    }
                }};
            }
            impl_test_bitor_operation!(bool, [true, false]);
            impl_test_bitor_operation!(usize, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitor_operation!(u8, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitor_operation!(u16, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitor_operation!(u32, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitor_operation!(u64, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitor_operation!(u128, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitor_operation!(isize, [0, 1, 2, 3, 4, 5, -1, -2, isize::MIN, isize::MAX]);
            impl_test_bitor_operation!(i8, [0, 1, 2, 3, 4, 5, -1, -2, i8::MIN, i8::MAX]);
            impl_test_bitor_operation!(i16, [0, 1, 2, 3, 4, 5, -1, -2, i16::MIN, i16::MAX]);
            impl_test_bitor_operation!(i32, [0, 1, 2, 3, 4, 5, -1, -2, i32::MIN, i32::MAX]);
            impl_test_bitor_operation!(i64, [0, 1, 2, 3, 4, 5, -1, -2, i64::MIN, i64::MAX]);
            impl_test_bitor_operation!(i128, [0, 1, 2, 3, 4, 5, -1, -2, i128::MIN, i128::MAX]);
        }
    }
}

#[codesnip::entry("BitXorOperation")]
pub use self::bitxor_operation_impl::{BitXorIdentity, BitXorOperation};
#[codesnip::entry("BitXorOperation", include("algebra"))]
mod bitxor_operation_impl {
    use super::*;
    use std::{marker::PhantomData, ops::BitXor};
    /// ^
    pub struct BitXorOperation<T>
    where
        T: Clone + BitXorIdentity,
    {
        _marker: PhantomData<fn() -> T>,
    }
    pub trait BitXorIdentity: Sized + BitXor<Output = Self> {
        fn xor_zero() -> Self;
    }
    #[macro_export]
    macro_rules! impl_bitxor_identity {
        ([$($wh:tt)*], $t:ty, $xor_zero:expr) => {
            impl<$($wh)*> BitXorIdentity for $t {
                #[inline]
                fn xor_zero() -> Self { $xor_zero }
            }
        };
        ($t:ty, $xor_zero:expr) =>{
            impl BitXorIdentity for $t {
                #[inline]
                fn xor_zero() -> Self { $xor_zero }
            }
        };
    }
    impl_bitxor_identity!(bool, false);
    impl_bitxor_identity!(usize, 0);
    impl_bitxor_identity!(u8, 0);
    impl_bitxor_identity!(u16, 0);
    impl_bitxor_identity!(u32, 0);
    impl_bitxor_identity!(u64, 0);
    impl_bitxor_identity!(u128, 0);
    impl_bitxor_identity!(isize, 0);
    impl_bitxor_identity!(i8, 0);
    impl_bitxor_identity!(i16, 0);
    impl_bitxor_identity!(i32, 0);
    impl_bitxor_identity!(i64, 0);
    impl_bitxor_identity!(i128, 0);
    impl<T> Magma for BitXorOperation<T>
    where
        T: Clone + BitXorIdentity,
    {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() ^ y.clone()
        }
    }
    impl<T> Unital for BitXorOperation<T>
    where
        T: Clone + BitXorIdentity,
    {
        #[inline]
        fn unit() -> Self::T {
            BitXorIdentity::xor_zero()
        }
    }
    impl<T> Associative for BitXorOperation<T> where T: Clone + BitXorIdentity {}
    impl<T> Commutative for BitXorOperation<T> where T: Clone + BitXorIdentity {}
    impl<T> Invertible for BitXorOperation<T>
    where
        T: Clone + BitXorIdentity,
    {
        fn inverse(x: &Self::T) -> Self::T {
            x.clone()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_bitxor_operation() {
            macro_rules! impl_test_bitxor_operation {
                ($ty:ty, $array:expr) => {{
                    type M = BitXorOperation<$ty>;
                    for a in $array {
                        assert!(M::check_unital(&a));
                        assert!(M::check_invertible(&a));
                        for b in $array {
                            assert!(M::check_commutative(&a, &b));
                            for c in $array {
                                assert!(M::check_associative(&a, &b, &c));
                            }
                        }
                    }
                }};
            }
            impl_test_bitxor_operation!(bool, [true, false]);
            impl_test_bitxor_operation!(usize, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitxor_operation!(u8, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitxor_operation!(u16, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitxor_operation!(u32, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitxor_operation!(u64, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitxor_operation!(u128, [0, 1, 2, 3, 4, 5, !0 - 1, !0]);
            impl_test_bitxor_operation!(isize, [0, 1, 2, 3, 4, 5, -1, -2, isize::MIN, isize::MAX]);
            impl_test_bitxor_operation!(i8, [0, 1, 2, 3, 4, 5, -1, -2, i8::MIN, i8::MAX]);
            impl_test_bitxor_operation!(i16, [0, 1, 2, 3, 4, 5, -1, -2, i16::MIN, i16::MAX]);
            impl_test_bitxor_operation!(i32, [0, 1, 2, 3, 4, 5, -1, -2, i32::MIN, i32::MAX]);
            impl_test_bitxor_operation!(i64, [0, 1, 2, 3, 4, 5, -1, -2, i64::MIN, i64::MAX]);
            impl_test_bitxor_operation!(i128, [0, 1, 2, 3, 4, 5, -1, -2, i128::MIN, i128::MAX]);
        }
    }
}

#[codesnip::entry("LogicalLinearOperation")]
pub use self::logical_linear_operation_impl::LogicalLinearOperation;
#[codesnip::entry(
    "LogicalLinearOperation",
    include("algebra", "BitXorOperation", "BitAndOperation")
)]
mod logical_linear_operation_impl {
    use super::*;
    use std::{
        marker::PhantomData,
        ops::{BitAnd, BitXor},
    };
    /// $(a, b) \circ (c, d) = \lambda x. c \wedge (a \wedge x \oplus b) \oplus d$
    pub struct LogicalLinearOperation<T>
    where
        T: Clone + BitXorIdentity + BitAndIdentity + BitXor<Output = T> + BitAnd<Output = T>,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> LogicalLinearOperation<T>
    where
        T: Clone + BitXorIdentity + BitAndIdentity + BitXor<Output = T> + BitAnd<Output = T>,
    {
        pub fn eval((a, b): &<Self as Magma>::T, x: &T) -> T {
            a.clone() & x.clone() ^ b.clone()
        }
    }
    impl<T> Magma for LogicalLinearOperation<T>
    where
        T: Clone + BitXorIdentity + BitAndIdentity + BitXor<Output = T> + BitAnd<Output = T>,
    {
        type T = (T, T);
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            (
                y.0.clone() & x.0.clone(),
                y.0.clone() & x.1.clone() ^ y.1.clone(),
            )
        }
    }
    impl<T> Unital for LogicalLinearOperation<T>
    where
        T: Clone + BitXorIdentity + BitAndIdentity + BitXor<Output = T> + BitAnd<Output = T>,
    {
        #[inline]
        fn unit() -> Self::T {
            (BitAndIdentity::all_one(), BitXorIdentity::xor_zero())
        }
    }
    impl<T> Associative for LogicalLinearOperation<T> where
        T: Clone + BitXorIdentity + BitAndIdentity + BitXor<Output = T> + BitAnd<Output = T>
    {
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_logical_linear_operation() {
            type M = LogicalLinearOperation<i32>;
            let iter = (-3..=3).flat_map(|x| (-3..=3).map(move |y| (x, y)));
            for a in iter.clone() {
                assert!(M::check_unital(&a));
                for b in iter.clone() {
                    for c in iter.clone() {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                    for x in -3..=3 {
                        assert_eq!(
                            M::eval(&M::operate(&a, &b), &x),
                            M::eval(&b, &M::eval(&a, &x))
                        );
                    }
                }
            }
        }
    }
}

#[codesnip::entry("TupleOperation", include("algebra"))]
mod tuple_operation_impl {
    use super::*;
    macro_rules! impl_tuple_operation {
        (@impl) => {
            impl Magma for () {
                type T = ();
                fn operate(_x: &Self::T, _y: &Self::T) -> Self::T {}
            }
            impl Unital for () {
                fn unit() -> Self::T {}
            }
            impl Associative for () {}
            impl Commutative for () {}
            impl Idempotent for () {}
            impl Invertible for () {
                fn inverse(_x: &Self::T) -> Self::T {}
            }
        };
        (@impl $($T:ident $i:tt)*) => {
            impl<$($T: Magma),*> Magma for ($($T,)*) {
                type T = ($(<$T as Magma>::T,)*);
                fn operate(x: &Self::T, y: &Self::T) -> Self::T {
                    ($(<$T as Magma>::operate(&x.$i, &y.$i),)*)
                }
            }
            impl<$($T: Unital),*> Unital for ($($T,)*) {
                fn unit() -> Self::T {
                    ($(<$T as Unital>::unit(),)*)
                }
            }
            impl<$($T: Associative),*> Associative for ($($T,)*) {}
            impl<$($T: Commutative),*> Commutative for ($($T,)*) {}
            impl<$($T: Idempotent),*> Idempotent for ($($T,)*) {}
            impl<$($T: Invertible),*> Invertible for ($($T,)*) {
                fn inverse(x: &Self::T) -> Self::T {
                    ($(<$T as Invertible>::inverse(&x.$i),)*)
                }
            }
        };
        (@inner $($T:ident $i:tt)*; $U:ident $j:tt $($t:tt)*) => {
            impl_tuple_operation!(@impl $($T $i)*);
            impl_tuple_operation!(@inner $($T $i)* $U $j; $($t)*);
        };
        (@inner $($T:ident $i:tt)*;) => {
            impl_tuple_operation!(@impl $($T $i)*);
        };
        ($($t:tt)*) => {
            impl_tuple_operation!(@inner ; $($t)*);
        };
    }
    impl_tuple_operation!(A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9);

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::num::mint_basic::MInt998244353;

        #[test]
        fn test_tuple_operation() {
            type MInt = MInt998244353;
            type M = (AdditiveOperation<MInt>, MultiplicativeOperation<MInt>);
            let iter = (-5..=5).flat_map(|x| (-5..=5).map(move |y| (MInt::from(x), MInt::from(y))));
            for a in iter.clone() {
                assert!(M::check_unital(&a));
                if !a.1.is_zero() {
                    assert!(M::check_invertible(&a));
                }
                for b in iter.clone() {
                    assert!(M::check_commutative(&a, &b));
                    for c in iter.clone() {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("ArrayOperation")]
pub use self::array_operation_impl::ArrayOperation;
#[codesnip::entry("ArrayOperation", include("algebra", "array"))]
mod array_operation_impl {
    use super::*;
    use crate::array;
    use std::marker::PhantomData;
    pub struct ArrayOperation<M, const N: usize> {
        _marker: PhantomData<fn() -> M>,
    }
    impl<M, const N: usize> Magma for ArrayOperation<M, N>
    where
        M: Magma,
    {
        type T = [M::T; N];
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            array!(|i| M::operate(&x[i], &y[i]); N)
        }
    }
    impl<M, const N: usize> Unital for ArrayOperation<M, N>
    where
        M: Unital,
    {
        #[inline]
        fn unit() -> Self::T {
            array!(|| M::unit(); N)
        }
    }
    impl<M, const N: usize> Associative for ArrayOperation<M, N> where M: Associative {}
    impl<M, const N: usize> Commutative for ArrayOperation<M, N> where M: Commutative {}
    impl<M, const N: usize> Idempotent for ArrayOperation<M, N> where M: Idempotent {}
    impl<M, const N: usize> Invertible for ArrayOperation<M, N>
    where
        M: Invertible,
    {
        #[inline]
        fn inverse(x: &Self::T) -> Self::T {
            array!(|i| M::inverse(&x[i]); N)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_array_operation() {
            type M = ArrayOperation<AdditiveOperation<i32>, 2>;

            let iter = (-5..=5).flat_map(|x| (-5..=5).map(move |y| [x, y]));
            for a in iter.clone() {
                assert!(M::check_unital(&a));
                assert!(M::check_invertible(&a));
                for b in iter.clone() {
                    assert!(M::check_commutative(&a, &b));
                    for c in iter.clone() {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("CountingOperation")]
pub use self::counting_operation_impl::CountingOperation;
#[codesnip::entry("CountingOperation", include("algebra"))]
mod counting_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    pub struct CountingOperation<M> {
        _marker: PhantomData<fn() -> M>,
    }
    impl<M> Magma for CountingOperation<M>
    where
        M: Magma<T: PartialEq> + Idempotent,
    {
        type T = (M::T, usize);
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let z = M::operate(&x.0, &y.0);
            match (z == x.0, z == y.0) {
                (true, true) => (z, x.1 + y.1),
                (true, false) => (z, x.1),
                (false, true) => (z, y.1),
                (false, false) => (z, 1),
            }
        }
    }
impl<M> Unital for CountingOperation<M>
where
    M: Unital<T: PartialEq> + Idempotent,
    {
        #[inline]
        fn unit() -> Self::T {
            (M::unit(), 0)
        }
    }
impl<M> Associative for CountingOperation<M>
where
    M: Associative<T: PartialEq> + Idempotent,
    {
    }
impl<M> Commutative for CountingOperation<M>
where
    M: Commutative<T: PartialEq> + Idempotent,
    {
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_counting_operation() {
            type M = CountingOperation<MaxOperation<i32>>;
            let iter = (-5..=5).flat_map(|x| (1..=5).map(move |y| (x, y)));
            for a in iter.clone() {
                assert!(M::check_unital(&a));
                for b in iter.clone() {
                    assert!(M::check_commutative(&a, &b));
                    for c in iter.clone() {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("ReverseOperation")]
pub use self::reverse_operation_impl::ReverseOperation;
#[codesnip::entry("ReverseOperation", include("algebra"))]
mod reverse_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    pub struct ReverseOperation<M> {
        _marker: PhantomData<fn() -> M>,
    }
    impl<M> Magma for ReverseOperation<M>
    where
        M: Magma,
    {
        type T = M::T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            M::operate(y, x)
        }
    }
    impl<M> Unital for ReverseOperation<M>
    where
        M: Unital,
    {
        #[inline]
        fn unit() -> Self::T {
            M::unit()
        }
    }
    impl<M> Associative for ReverseOperation<M> where M: Associative {}
    impl<M> Commutative for ReverseOperation<M> where M: Commutative {}
    impl<M> Invertible for ReverseOperation<M>
    where
        M: Invertible,
    {
        #[inline]
        fn inverse(x: &Self::T) -> Self::T {
            M::inverse(x)
        }
    }
    impl<M> Idempotent for ReverseOperation<M> where M: Idempotent {}

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::num::mint_basic::MInt998244353;

        #[test]
        fn test_reverse_operation() {
            type MInt = MInt998244353;
            type M = ReverseOperation<LinearOperation<MInt>>;
            let iter = (-3..=3).flat_map(|x| (-3..=3).map(move |y| (MInt::from(x), MInt::from(y))));
            for a in iter.clone() {
                assert!(M::check_unital(&a));
                if !a.0.is_zero() {
                    assert!(M::check_invertible(&a));
                }
                for b in iter.clone() {
                    for c in iter.clone() {
                        assert!(M::check_associative(&a, &b, &c));
                    }
                }
            }
        }
    }
}

#[codesnip::entry("TopkOperation")]
pub use self::topk_operation_impl::TopkOperation;
#[codesnip::entry("TopkOperation", include("algebra", "bounded", "array"))]
mod topk_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    pub struct TopkOperation<const K: usize, T>
    where
        T: Clone + Ord + Bounded,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<const K: usize, T> Magma for TopkOperation<K, T>
    where
        T: Clone + Ord + Bounded,
    {
        type T = [T; K];
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let mut i = 0;
            let mut j = 0;
            crate::array![|| if i == K || j != K && x[i] < y[j] {
                let t = &y[j];
                j += 1;
                t.clone()
            } else {
                let t = &x[i];
                i += 1;
                t.clone()
            }; K]
        }
    }
    impl<const K: usize, T> Unital for TopkOperation<K, T>
    where
        T: Clone + Ord + Bounded,
    {
        fn unit() -> Self::T {
            crate::array![|| <T as Bounded>::minimum(); K]
        }
    }
    impl<const K: usize, T> Associative for TopkOperation<K, T> where T: Clone + Ord + Bounded {}
    impl<const K: usize, T> Commutative for TopkOperation<K, T> where T: Clone + Ord + Bounded {}

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{array, tools::Xorshift};

        #[test]
        fn test_topk_operation() {
            type M = TopkOperation<4, i64>;
            let mut rng = Xorshift::default();
            for _ in 0..100 {
                let mut x = [i64::MIN; 4];
                for _ in 0..100 {
                    let mut y = [i64::MIN; 4];
                    for y in &mut y {
                        *y = rng.random(0..1000);
                    }
                    y.sort_unstable();
                    y.reverse();
                    let z = {
                        let mut x = x.to_vec();
                        x.extend(&y);
                        x.sort_unstable();
                        x.reverse();
                        x.truncate(4);
                        x
                    };
                    let zz = M::operate(&x, &y);
                    for (z, zz) in z.iter().zip(&zz) {
                        assert_eq!(z, zz);
                    }
                    x = zz;
                }

                let mut g = || {
                    if rng.random(0..3) == 0 {
                        i64::MIN
                    } else {
                        rng.random(0..10)
                    }
                };
                let mut a = array![|| g(); 4];
                a.sort_unstable();
                a.reverse();
                assert!(M::check_unital(&a));
                let mut b = array![|| g(); 4];
                b.sort_unstable();
                b.reverse();
                assert!(M::check_commutative(&a, &b));
                let mut c = array![|| g(); 4];
                c.sort_unstable();
                c.reverse();
                assert!(M::check_associative(&a, &b, &c));
            }
        }
    }
}

#[codesnip::entry("BottomkOperation")]
pub use self::bottomk_operation_impl::BottomkOperation;
#[codesnip::entry("BottomkOperation", include("algebra", "bounded", "array"))]
mod bottomk_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    pub struct BottomkOperation<const K: usize, T>
    where
        T: Clone + Ord + Bounded,
    {
        _marker: PhantomData<fn() -> T>,
    }
    impl<const K: usize, T> Magma for BottomkOperation<K, T>
    where
        T: Clone + Ord + Bounded,
    {
        type T = [T; K];
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let mut i = 0;
            let mut j = 0;
            crate::array![|| if i == K || j != K && x[i] > y[j] {
                let t = &y[j];
                j += 1;
                t.clone()
            } else {
                let t = &x[i];
                i += 1;
                t.clone()
            }; K]
        }
    }
    impl<const K: usize, T> Unital for BottomkOperation<K, T>
    where
        T: Clone + Ord + Bounded,
    {
        fn unit() -> Self::T {
            crate::array![|| <T as Bounded>::maximum(); K]
        }
    }
    impl<const K: usize, T> Associative for BottomkOperation<K, T> where T: Clone + Ord + Bounded {}
    impl<const K: usize, T> Commutative for BottomkOperation<K, T> where T: Clone + Ord + Bounded {}

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{array, tools::Xorshift};

        #[test]
        fn test_bottomk_operation() {
            type M = BottomkOperation<4, i64>;
            let mut rng = Xorshift::default();
            for _ in 0..100 {
                let mut x = [i64::MAX; 4];
                for _ in 0..100 {
                    let mut y = [i64::MAX; 4];
                    for y in &mut y {
                        *y = rng.random(0..1000);
                    }
                    y.sort_unstable();
                    let z = {
                        let mut x = x.to_vec();
                        x.extend(&y);
                        x.sort_unstable();
                        x.truncate(4);
                        x
                    };
                    let zz = M::operate(&x, &y);
                    for (z, zz) in z.iter().zip(&zz) {
                        assert_eq!(z, zz);
                    }
                    x = zz;
                }

                let mut g = || {
                    if rng.random(0..3) == 0 {
                        i64::MAX
                    } else {
                        rng.random(0..10)
                    }
                };
                let mut a = array![|| g(); 4];
                a.sort_unstable();
                assert!(M::check_unital(&a));
                let mut b = array![|| g(); 4];
                b.sort_unstable();
                assert!(M::check_commutative(&a, &b));
                let mut c = array![|| g(); 4];
                c.sort_unstable();
                assert!(M::check_associative(&a, &b, &c));
            }
        }
    }
}

#[codesnip::entry("PermutationOperation")]
pub use self::permutation_operation_impl::PermutationOperation;
#[codesnip::entry("PermutationOperation", include("algebra"))]
mod permutation_operation_impl {
    use super::*;
    pub enum PermutationOperation {}
    impl Magma for PermutationOperation {
        type T = Vec<usize>;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let n = x.len().max(y.len());
            let z: Vec<_> = (0..n)
                .map(|i| {
                    let j = y.get(i).cloned().unwrap_or(i);
                    x.get(j).cloned().unwrap_or(j)
                })
                .collect();
            z
        }
    }
    impl Associative for PermutationOperation {}
    impl Unital for PermutationOperation {
        fn unit() -> Self::T {
            Vec::new()
        }

        fn is_unit(x: &Self::T) -> bool
        where
            Self::T: PartialEq,
        {
            x.iter().enumerate().all(|(i, &x)| i == x)
        }
    }
    impl Invertible for PermutationOperation {
        fn inverse(x: &Self::T) -> Self::T {
            let mut y = vec![0; x.len()];
            for (i, x) in x.iter().enumerate() {
                y[*x] = i;
            }
            y
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::tools::Xorshift;

        #[test]
        fn test_permutation_operation() {
            type M = PermutationOperation;
            let mut rng = Xorshift::default();
            for _ in 0..100 {
                let mut a: Vec<usize> = (0..rng.random(0..20)).collect();
                let mut b: Vec<usize> = (0..rng.random(0..20)).collect();
                let mut c: Vec<usize> = (0..rng.random(0..20)).collect();
                rng.shuffle(&mut a);
                rng.shuffle(&mut b);
                rng.shuffle(&mut c);
                assert!(M::check_unital(&a));
                assert!(M::check_invertible(&a));
                assert!(M::check_associative(&a, &b, &c));
            }
        }
    }
}

#[codesnip::entry("FindMajorityOperation")]
pub use self::find_majority_operation_impl::FindMajorityOperation;
#[codesnip::entry("FindMajorityOperation", include("algebra"))]
mod find_majority_operation_impl {
    use super::*;
    use std::{cmp::Ordering, marker::PhantomData};
    /// Find majority(strict) of a sequence.
    ///
    /// fold $x \in S$ with `(Some(x), 1)`
    ///
    /// `(Some(m), _)` represents `m` may be a majority of $S$.
    ///
    /// `(None, _)` represents that there is no majority value.
    pub struct FindMajorityOperation<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for FindMajorityOperation<T>
    where
        T: Clone + Eq,
    {
        type T = (Option<T>, usize);
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            if y.0.is_none() {
                x.clone()
            } else if x.0.is_none() {
                y.clone()
            } else {
                match (x.0.eq(&y.0), x.1.cmp(&y.1)) {
                    (true, _) => (x.0.clone(), x.1 + y.1),
                    (_, Ordering::Less) => (y.0.clone(), y.1 - x.1),
                    (_, Ordering::Equal) => (None, 0),
                    (_, Ordering::Greater) => (x.0.clone(), x.1 - y.1),
                }
            }
        }
    }
    impl<T> Unital for FindMajorityOperation<T>
    where
        T: Clone + Eq,
    {
        fn unit() -> Self::T {
            (None, 0)
        }
    }
    impl<T> Associative for FindMajorityOperation<T> where T: Clone + Eq {}

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::{collections::HashMap, iter::once};

        #[test]
        fn test_find_majority_operation() {
            type M = FindMajorityOperation<i32>;
            let iter = (-5..=5)
                .flat_map(|x| (1..=5).map(move |y| (Some(x), y)))
                .chain(once((None, 0)));
            for a in iter.clone() {
                assert!(M::check_unital(&a));
                for b in iter.clone() {
                    for c in iter.clone() {
                        // no associativity
                        // assert!(M::check_associative(&a, &b, &c));
                        let mut count = HashMap::<_, usize>::new();
                        for (key, cnt) in [a, b, c] {
                            if let Some(key) = key {
                                *count.entry(key).or_default() += cnt;
                            }
                        }
                        let max = count.values().max().cloned().unwrap_or_default();
                        let sum: usize = count.values().sum();
                        if max * 2 > sum {
                            assert_eq!(
                                M::operate(&M::operate(&a, &b), &c).0,
                                count.into_iter().find(|&(_, v)| v == max).map(|(k, _)| k)
                            );
                        }
                    }
                }
            }
        }
    }
}

pub use self::concatenate_operation::{ConcatenateOperation, SortedConcatenateOperation};
mod concatenate_operation {
    use super::*;
    use std::marker::PhantomData;
    pub struct ConcatenateOperation<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for ConcatenateOperation<T>
    where
        T: Clone,
    {
        type T = Vec<T>;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.iter().chain(y.iter()).cloned().collect()
        }
    }
    impl<T> Unital for ConcatenateOperation<T>
    where
        T: Clone,
    {
        fn unit() -> Self::T {
            Vec::new()
        }
    }
    impl<T> Associative for ConcatenateOperation<T> where T: Clone {}

    pub struct SortedConcatenateOperation<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> Magma for SortedConcatenateOperation<T>
    where
        T: Clone + Ord,
    {
        type T = Vec<T>;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let mut xit = x.iter().cloned().peekable();
            let mut yit = y.iter().cloned().peekable();
            let mut z = Vec::with_capacity(x.len() + y.len());
            loop {
                match (xit.peek(), yit.peek()) {
                    (None, None) => break,
                    (Some(_), None) => z.push(xit.next().unwrap()),
                    (Some(x), Some(y)) if x <= y => z.push(xit.next().unwrap()),
                    _ => z.push(yit.next().unwrap()),
                }
            }
            z
        }
    }
    impl<T> Unital for SortedConcatenateOperation<T>
    where
        T: Clone + Ord,
    {
        fn unit() -> Self::T {
            Vec::new()
        }
    }
    impl<T> Associative for SortedConcatenateOperation<T> where T: Clone + Ord {}
    impl<T> Commutative for SortedConcatenateOperation<T> where T: Clone + Ord {}

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{rand, tools::Xorshift};

        #[test]
        fn test_concatenate_operation() {
            type M = ConcatenateOperation<i32>;
            let mut rng = Xorshift::default();
            for _ in 0..100 {
                rand!(rng, n: 0..4, a: [0..10; n], m: 0..4, b: [0..10; m], l: 0..4, c: [0..10; l]);
                assert!(M::check_unital(&a));
                assert!(M::check_associative(&a, &b, &c));

                let ab: Vec<_> = a.iter().chain(b.iter()).cloned().collect();
                assert_eq!(M::operate(&a, &b), ab);
            }
        }

        #[test]
        fn test_sorted_concatenate_operation() {
            type M = SortedConcatenateOperation<i32>;
            let mut rng = Xorshift::default();
            for _ in 0..100 {
                rand!(rng, n: 0..4, mut a: [0..10; n], m: 0..4, mut b: [0..10; m], l: 0..4, mut c: [0..10; l]);
                a.sort_unstable();
                b.sort_unstable();
                c.sort_unstable();
                assert!(M::check_unital(&a));
                assert!(M::check_commutative(&a, &b));
                assert!(M::check_associative(&a, &b, &c));

                let mut ab: Vec<_> = a.iter().chain(b.iter()).cloned().collect();
                ab.sort_unstable();
                assert_eq!(M::operate(&a, &b), ab);
            }
        }
    }
}

#[codesnip::entry("MinimumIntervalMovementOperation")]
pub use self::minimum_interval_movement_impl::{
    MinimumIntervalMovement, MinimumIntervalMovementOperation,
};
#[codesnip::entry(
    "MinimumIntervalMovementOperation",
    include("algebra", "bounded", "zero_one")
)]
mod minimum_interval_movement_impl {
    use super::*;
    use std::{
        marker::PhantomData,
        ops::{Add, Sub},
    };

    pub struct MinimumIntervalMovementOperation<T> {
        _marker: PhantomData<fn() -> T>,
    }
    #[derive(Debug, Clone)]
    pub struct MinimumIntervalMovement<T> {
        pos_range: (T, T),
        move_range: (T, T),
        cost: T,
    }
    impl<T> MinimumIntervalMovement<T>
    where
        T: Clone + Zero,
    {
        pub fn new(l: T, r: T) -> Self {
            Self {
                pos_range: (l.clone(), r.clone()),
                move_range: (l, r),
                cost: T::zero(),
            }
        }
    }
    impl<T> MinimumIntervalMovement<T>
    where
        T: Clone + Ord + Zero,
    {
        pub fn position(&self, x: &T) -> T {
            x.clamp(&self.pos_range.0, &self.pos_range.1).clone()
        }
    }
    impl<T> MinimumIntervalMovement<T>
    where
        T: Clone + Ord + Add<Output = T> + Sub<Output = T> + Zero,
    {
        pub fn move_cost(&self, x: &T) -> T {
            x.max(&self.move_range.0).clone() - x.min(&self.move_range.1).clone()
                + self.cost.clone()
        }
    }
    impl<T> Magma for MinimumIntervalMovementOperation<T>
    where
        T: Clone + Ord + Add<Output = T> + Sub<Output = T> + Zero,
    {
        type T = MinimumIntervalMovement<T>;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let pos_range = (
                (&x.pos_range.0)
                    .clamp(&y.pos_range.0, &y.pos_range.1)
                    .clone(),
                (&x.pos_range.1)
                    .clamp(&y.pos_range.0, &y.pos_range.1)
                    .clone(),
            );
            let move_range = (
                (&y.move_range.0)
                    .clamp(&x.move_range.0, &x.move_range.1)
                    .clone(),
                (&y.move_range.1)
                    .clamp(&x.move_range.0, &x.move_range.1)
                    .clone(),
            );
            let cost = x.cost.clone() + y.move_cost(&x.position(&move_range.0));
            MinimumIntervalMovement {
                pos_range,
                move_range,
                cost,
            }
        }
    }
    impl<T> Associative for MinimumIntervalMovementOperation<T> where
        T: Clone + Ord + Add<Output = T> + Sub<Output = T> + Zero
    {
    }
    impl<T> Unital for MinimumIntervalMovementOperation<T>
    where
        T: Clone + Ord + Add<Output = T> + Sub<Output = T> + Zero + Bounded,
    {
        fn unit() -> Self::T {
            MinimumIntervalMovement::new(T::minimum(), T::maximum())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{chmin, tools::Xorshift};
        use std::collections::HashMap;

        #[test]
        fn test_minimum_interval_movement_operation() {
            type M = MinimumIntervalMovementOperation<i32>;
            let mut rng = Xorshift::default();
            for _ in 0..100 {
                let mut min = M::unit();
                let mut cost = HashMap::new();
                let s: i32 = rng.random(-100..100);
                cost.insert(s, 0i32);
                for _ in 0..10 {
                    let l = rng.random(-100..100);
                    let r = rng.random(l..=100);
                    let m = MinimumIntervalMovement::new(l, r);
                    min = M::operate(&min, &m);
                    let mut ncost = HashMap::new();
                    for (x, c) in cost {
                        for nx in l..=r {
                            let nc = c + (x - nx).abs();
                            chmin!(*ncost.entry(nx).or_insert(nc), nc);
                        }
                    }
                    cost = ncost;
                    let x = min.position(&s);
                    let c = min.move_cost(&s);
                    assert_eq!(Some(&c), cost.get(&x));
                }
            }
        }
    }
}
