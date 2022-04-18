//! binary operaions

use super::magma::*;
use crate::num::{Bounded, One, Zero};

#[codesnip::entry("MaxOperation")]
pub use self::max_operation_impl::MaxOperation;
#[codesnip::entry("MaxOperation", include("algebra", "bounded"))]
mod max_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    /// binary operation to select larger element
    pub struct MaxOperation<T: Clone + Ord + Bounded> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone + Ord + Bounded> Magma for MaxOperation<T> {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.max(y).clone()
        }
    }
    impl<T: Clone + Ord + Bounded> Unital for MaxOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            <T as Bounded>::minimum()
        }
    }
    impl<T: Clone + Ord + Bounded> Associative for MaxOperation<T> {}
    impl<T: Clone + Ord + Bounded> Commutative for MaxOperation<T> {}
    impl<T: Clone + Ord + Bounded> Idempotent for MaxOperation<T> {}
}

#[codesnip::entry("MinOperation")]
pub use self::min_operation_impl::MinOperation;
#[codesnip::entry("MinOperation", include("algebra", "bounded"))]
mod min_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    /// binary operation to select smaller element
    pub struct MinOperation<T: Clone + Ord + Bounded> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone + Ord + Bounded> Magma for MinOperation<T> {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.min(y).clone()
        }
    }
    impl<T: Clone + Ord + Bounded> Unital for MinOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            <T as Bounded>::maximum()
        }
    }
    impl<T: Clone + Ord + Bounded> Associative for MinOperation<T> {}
    impl<T: Clone + Ord + Bounded> Commutative for MinOperation<T> {}
    impl<T: Clone + Ord + Bounded> Idempotent for MinOperation<T> {}
}

#[codesnip::entry("FirstOperation")]
pub use self::first_operation_impl::FirstOperation;
#[codesnip::entry("FirstOperation", include("algebra"))]
mod first_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    /// retain the first element
    pub struct FirstOperation<T: Clone> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone> Magma for FirstOperation<T> {
        type T = Option<T>;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.as_ref().or(y.as_ref()).cloned()
        }
    }
    impl<T: Clone> Unital for FirstOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            None
        }
    }
    impl<T: Clone> Associative for FirstOperation<T> {}
    impl<T: Clone> Idempotent for FirstOperation<T> {}
}

#[codesnip::entry("LastOperation")]
pub use self::last_operation_impl::LastOperation;
#[codesnip::entry("LastOperation", include("algebra"))]
mod last_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    /// retain the last element
    pub struct LastOperation<T: Clone> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone> Magma for LastOperation<T> {
        type T = Option<T>;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            y.as_ref().or(x.as_ref()).cloned()
        }
    }
    impl<T: Clone> Unital for LastOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            None
        }
    }
    impl<T: Clone> Associative for LastOperation<T> {}
    impl<T: Clone> Idempotent for LastOperation<T> {}
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
    pub struct AdditiveOperation<T: Clone + Zero + Add<Output = T>> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone + Zero + Add<Output = T>> Magma for AdditiveOperation<T> {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() + y.clone()
        }
    }
    impl<T: Clone + Zero + Add<Output = T>> Unital for AdditiveOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            Zero::zero()
        }
    }
    impl<T: Clone + Zero + Add<Output = T>> Associative for AdditiveOperation<T> {}
    impl<T: Clone + Zero + Add<Output = T>> Commutative for AdditiveOperation<T> {}
    impl<T: Clone + Zero + Add<Output = T> + Sub<Output = T> + Neg<Output = T>> Invertible
        for AdditiveOperation<T>
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
    pub struct MultiplicativeOperation<T: Clone + One + Mul<Output = T>> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone + One + Mul<Output = T>> Magma for MultiplicativeOperation<T> {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() * y.clone()
        }
    }
    impl<T: Clone + One + Mul<Output = T>> Unital for MultiplicativeOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            One::one()
        }
    }
    impl<T: Clone + One + Mul<Output = T>> Associative for MultiplicativeOperation<T> {}
    impl<T: Clone + One + Mul<Output = T>> Commutative for MultiplicativeOperation<T> {}
    impl<T: Clone + One + Mul<Output = T> + Div<Output = T>> Invertible for MultiplicativeOperation<T> {
        #[inline]
        fn inverse(x: &Self::T) -> Self::T {
            Self::unit().div(x.clone())
        }
        #[inline]
        fn rinv_operate(x: &Self::T, y: &Self::T) -> Self::T {
            (x.clone()).div(y.clone())
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
    pub struct LinearOperation<T: Clone + Zero + Add<Output = T> + One + Mul<Output = T>> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>> Magma for LinearOperation<T> {
        type T = (T, T);
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            (
                y.0.clone() * x.0.clone(),
                y.0.clone() * x.1.clone() + y.1.clone(),
            )
        }
    }
    impl<T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>> Unital for LinearOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            (One::one(), Zero::zero())
        }
    }
    impl<T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>> Associative for LinearOperation<T> {}
    impl<
            T: Clone
                + Zero
                + One
                + Add<Output = T>
                + Sub<Output = T>
                + Neg<Output = T>
                + Mul<Output = T>
                + Div<Output = T>,
        > Invertible for LinearOperation<T>
    {
        fn inverse(x: &Self::T) -> Self::T {
            let y = <T as One>::one().div(x.0.clone());
            (y.clone(), -y.mul(x.1.clone()))
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
    pub struct BitAndOperation<T: Clone + BitAndIdentity> {
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
    impl_bitand_identity!(usize, std::usize::MAX);
    impl_bitand_identity!(u8, std::u8::MAX);
    impl_bitand_identity!(u16, std::u16::MAX);
    impl_bitand_identity!(u32, std::u32::MAX);
    impl_bitand_identity!(u64, std::u64::MAX);
    impl_bitand_identity!(isize, std::isize::MIN);
    impl_bitand_identity!(i8, std::i8::MIN);
    impl_bitand_identity!(i16, std::i16::MIN);
    impl_bitand_identity!(i32, std::i32::MIN);
    impl_bitand_identity!(i64, std::i64::MIN);
    impl<T: Clone + BitAndIdentity> Magma for BitAndOperation<T> {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() & y.clone()
        }
    }
    impl<T: Clone + BitAndIdentity> Unital for BitAndOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            BitAndIdentity::all_one()
        }
    }
    impl<T: Clone + BitAndIdentity> Associative for BitAndOperation<T> {}
    impl<T: Clone + BitAndIdentity> Commutative for BitAndOperation<T> {}
    impl<T: Clone + BitAndIdentity> Idempotent for BitAndOperation<T> {}
}

#[codesnip::entry("BitOrOperation")]
pub use self::bitor_operation_impl::{BitOrIdentity, BitOrOperation};
#[codesnip::entry("BitOrOperation", include("algebra"))]
mod bitor_operation_impl {
    use super::*;
    use std::{marker::PhantomData, ops::BitOr};
    /// |
    pub struct BitOrOperation<T: Clone + BitOrIdentity> {
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
    impl_bitor_identity!(usize, 0usize);
    impl_bitor_identity!(u8, 0u8);
    impl_bitor_identity!(u16, 0u16);
    impl_bitor_identity!(u32, 0u32);
    impl_bitor_identity!(u64, 0u64);
    impl_bitor_identity!(isize, 0isize);
    impl_bitor_identity!(i8, 0i8);
    impl_bitor_identity!(i16, 0i16);
    impl_bitor_identity!(i32, 0i32);
    impl_bitor_identity!(i64, 0i64);
    impl<T: Clone + BitOrIdentity> Magma for BitOrOperation<T> {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() | y.clone()
        }
    }
    impl<T: Clone + BitOrIdentity> Unital for BitOrOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            BitOrIdentity::all_zero()
        }
    }
    impl<T: Clone + BitOrIdentity> Associative for BitOrOperation<T> {}
    impl<T: Clone + BitOrIdentity> Commutative for BitOrOperation<T> {}
    impl<T: Clone + BitOrIdentity> Idempotent for BitOrOperation<T> {}
}

#[codesnip::entry("BitXorOperation")]
pub use self::bitxor_operation_impl::{BitXorIdentity, BitXorOperation};
#[codesnip::entry("BitXorOperation", include("algebra"))]
mod bitxor_operation_impl {
    use super::*;
    use std::{marker::PhantomData, ops::BitXor};
    /// ^
    pub struct BitXorOperation<T: Clone + BitXorIdentity> {
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
    impl_bitxor_identity!(usize, 0usize);
    impl_bitxor_identity!(u8, 0u8);
    impl_bitxor_identity!(u16, 0u16);
    impl_bitxor_identity!(u32, 0u32);
    impl_bitxor_identity!(u64, 0u64);
    impl_bitxor_identity!(isize, 0isize);
    impl_bitxor_identity!(i8, 0i8);
    impl_bitxor_identity!(i16, 0i16);
    impl_bitxor_identity!(i32, 0i32);
    impl_bitxor_identity!(i64, 0i64);
    impl<T: Clone + BitXorIdentity> Magma for BitXorOperation<T> {
        type T = T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            x.clone() ^ y.clone()
        }
    }
    impl<T: Clone + BitXorIdentity> Unital for BitXorOperation<T> {
        #[inline]
        fn unit() -> Self::T {
            BitXorIdentity::xor_zero()
        }
    }
    impl<T: Clone + BitXorIdentity> Associative for BitXorOperation<T> {}
    impl<T: Clone + BitXorIdentity> Commutative for BitXorOperation<T> {}
    impl<T: Clone + BitXorIdentity> Invertible for BitXorOperation<T> {
        fn inverse(x: &Self::T) -> Self::T {
            x.clone()
        }
    }
}

#[codesnip::entry("TupleOperation", include("algebra"))]
mod tuple_operation_impl {
    #![allow(unused_variables, clippy::unused_unit)]
    use super::*;
    macro_rules! impl_tuple_operation {
        (@impl $($T:ident)*, $($i:tt)*) => {
            impl<$($T: Magma),*> Magma for ($($T,)*) {
                type T = ($(<$T as Magma>::T,)*);
                #[inline]
                fn operate(x: &Self::T, y: &Self::T) -> Self::T {
                    ($(<$T as Magma>::operate(&x.$i, &y.$i),)*)
                }
            }
            impl<$($T: Unital),*> Unital for ($($T,)*) {
                #[inline]
                fn unit() -> Self::T {
                    ($(<$T as Unital>::unit(),)*)
                }
            }
            impl<$($T: Associative),*> Associative for ($($T,)*) {}
            impl<$($T: Commutative),*> Commutative for ($($T,)*) {}
            impl<$($T: Idempotent),*> Idempotent for ($($T,)*) {}
            impl<$($T: Invertible),*> Invertible for ($($T,)*) {
                #[inline]
                fn inverse(x: &Self::T) -> Self::T {
                    ($(<$T as Invertible>::inverse(&x.$i),)*)
                }
            }
        };
        (@inner [$($T:ident)*][] [$($i:tt)*][]) => {
            impl_tuple_operation!(@impl $($T)*, $($i)*);
        };
        (@inner [$($T:ident)*][$U:ident $($Rest:ident)*] [$($i:tt)*][$j:tt $($rest:tt)*]) => {
            impl_tuple_operation!(@impl $($T)*, $($i)*);
            impl_tuple_operation!(@inner [$($T)* $U][$($Rest)*] [$($i)* $j][$($rest)*]);
        };
        ($($T:ident)*, $($i:tt)*) => {
            impl_tuple_operation!(@inner [][$($T)*] [][$($i)*]);
        };
    }
    impl_tuple_operation!(A B C D E F G H I J, 0 1 2 3 4 5 6 7 8 9);
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
    impl<M: Magma> Magma for CountingOperation<M>
    where
        M::T: PartialEq,
    {
        type T = (M::T, usize);
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            if x.0 == y.0 {
                (x.0.clone(), x.1 + y.1)
            } else {
                let z = M::operate(&x.0, &y.0);
                if z == x.0 {
                    (z, x.1)
                } else if z == y.0 {
                    (z, y.1)
                } else {
                    (z, 1)
                }
            }
        }
    }
    impl<M: Unital> Unital for CountingOperation<M>
    where
        M::T: PartialEq,
    {
        #[inline]
        fn unit() -> Self::T {
            (M::unit(), 0)
        }
    }
    impl<M: Associative> Associative for CountingOperation<M> {}
    impl<M: Commutative> Commutative for CountingOperation<M> {}
    impl<M: Idempotent> Idempotent for CountingOperation<M> {}
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
    impl<M: Magma> Magma for ReverseOperation<M> {
        type T = M::T;
        #[inline]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            M::operate(y, x)
        }
    }
    impl<M: Unital> Unital for ReverseOperation<M> {
        #[inline]
        fn unit() -> Self::T {
            M::unit()
        }
    }
    impl<M: Associative> Associative for ReverseOperation<M> {}
    impl<M: Commutative> Commutative for ReverseOperation<M> {}
    impl<M: Invertible> Invertible for ReverseOperation<M> {
        #[inline]
        fn inverse(x: &Self::T) -> Self::T {
            M::inverse(x)
        }
    }
    impl<M: Idempotent> Idempotent for ReverseOperation<M> {}
}

#[codesnip::entry("Top2Operation")]
pub use self::top2_operation_impl::Top2Operation;
#[codesnip::entry("Top2Operation", include("algebra", "bounded"))]
mod top2_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    pub struct Top2Operation<T: Clone + Ord + Bounded> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone + Ord + Bounded> Magma for Top2Operation<T> {
        type T = (T, T);
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            if x.0 < y.0 {
                (y.0.clone(), if x.0 < y.1 { &y.1 } else { &x.0 }.clone())
            } else {
                (x.0.clone(), if x.1 < y.0 { &y.0 } else { &x.1 }.clone())
            }
        }
    }
    impl<T: Clone + Ord + Bounded> Unital for Top2Operation<T> {
        fn unit() -> Self::T {
            (<T as Bounded>::minimum(), <T as Bounded>::minimum())
        }
    }
    impl<T: Clone + Ord + Bounded> Associative for Top2Operation<T> {}
    impl<T: Clone + Ord + Bounded> Commutative for Top2Operation<T> {}
}

#[codesnip::entry("Bottom2Operation")]
pub use self::bottom2_operation_impl::Bottom2Operation;
#[codesnip::entry("Bottom2Operation", include("algebra", "bounded"))]
mod bottom2_operation_impl {
    use super::*;
    use std::marker::PhantomData;
    pub struct Bottom2Operation<T: Clone + Ord + Bounded> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T: Clone + Ord + Bounded> Magma for Bottom2Operation<T> {
        type T = (T, T);
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            if x.0 > y.0 {
                (y.0.clone(), if x.0 > y.1 { &y.1 } else { &x.0 }.clone())
            } else {
                (x.0.clone(), if x.1 > y.0 { &y.0 } else { &x.1 }.clone())
            }
        }
    }
    impl<T: Clone + Ord + Bounded> Unital for Bottom2Operation<T> {
        fn unit() -> Self::T {
            (<T as Bounded>::maximum(), <T as Bounded>::maximum())
        }
    }
    impl<T: Clone + Ord + Bounded> Associative for Bottom2Operation<T> {}
    impl<T: Clone + Ord + Bounded> Commutative for Bottom2Operation<T> {}
}

#[codesnip::entry("PermutationOperation")]
pub use self::permutation_operation_impl::PermutationOperation;
#[codesnip::entry("PermutationOperation", include("algebra"))]
mod permutation_operation_impl {
    use super::*;
    pub struct PermutationOperation;
    impl Magma for PermutationOperation {
        type T = Vec<usize>;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            y.iter()
                .map(|&y| if y < x.len() { x[y] } else { y })
                .collect()
        }
    }
    impl Associative for PermutationOperation {}
    impl Unital for PermutationOperation {
        fn unit() -> Self::T {
            Vec::new()
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
    impl<T: Clone + Eq> Magma for FindMajorityOperation<T> {
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
    impl<T: Clone + Eq> Unital for FindMajorityOperation<T> {
        fn unit() -> Self::T {
            (None, 0)
        }
    }
    impl<T> Associative for FindMajorityOperation<T> {}
}
