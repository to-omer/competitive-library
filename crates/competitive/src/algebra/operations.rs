//! binary operaions

use super::magma::*;
use crate::num::{Bounded, One, Zero};

/// binary operation to select larger element
#[codesnip::entry("MaxOperation", include("algebra", "bounded"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaxOperation<T: Clone + Ord + Bounded> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("MaxOperation")]
mod max_operation_impl {
    use super::*;
    impl<T: Clone + Ord + Bounded> MaxOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Clone + Ord + Bounded> Magma for MaxOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            x.max(y).clone()
        }
    }
    impl<T: Clone + Ord + Bounded> Unital for MaxOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            <T as Bounded>::MIN
        }
    }
    impl<T: Clone + Ord + Bounded> Associative for MaxOperation<T> {}
    impl<T: Clone + Ord + Bounded> Commutative for MaxOperation<T> {}
    impl<T: Clone + Ord + Bounded> Idempotent for MaxOperation<T> {}
}

/// binary operation to select smaller element
#[codesnip::entry("MinOperation", include("algebra", "bounded"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MinOperation<T: Clone + Ord + Bounded> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("MinOperation")]
mod min_operation_impl {
    use super::*;
    impl<T: Clone + Ord + Bounded> MinOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Clone + Ord + Bounded> Magma for MinOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            x.min(y).clone()
        }
    }
    impl<T: Clone + Ord + Bounded> Unital for MinOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            <T as Bounded>::MAX
        }
    }
    impl<T: Clone + Ord + Bounded> Associative for MinOperation<T> {}
    impl<T: Clone + Ord + Bounded> Commutative for MinOperation<T> {}
    impl<T: Clone + Ord + Bounded> Idempotent for MinOperation<T> {}
}

/// retain the first element
#[codesnip::entry("FirstOperation", include("algebra"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FirstOperation<T: Clone + PartialEq> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("FirstOperation")]
mod first_operation_impl {
    use super::*;
    impl<T: Clone + PartialEq> FirstOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Clone + PartialEq> Magma for FirstOperation<T> {
        type T = Option<T>;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            x.as_ref().or_else(|| y.as_ref()).cloned()
        }
    }
    impl<T: Clone + PartialEq> Unital for FirstOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            None
        }
    }
    impl<T: Clone + PartialEq> Associative for FirstOperation<T> {}
    impl<T: Clone + PartialEq> Idempotent for FirstOperation<T> {}
}

/// retain the last element
#[codesnip::entry("LastOperation", include("algebra"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LastOperation<T: Clone + PartialEq> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("LastOperation")]
mod last_operation_impl {
    use super::*;
    impl<T: Clone + PartialEq> LastOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Clone + PartialEq> Magma for LastOperation<T> {
        type T = Option<T>;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            y.as_ref().or_else(|| x.as_ref()).cloned()
        }
    }
    impl<T: Clone + PartialEq> Unital for LastOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            None
        }
    }
    impl<T: Clone + PartialEq> Associative for LastOperation<T> {}
    impl<T: Clone + PartialEq> Idempotent for LastOperation<T> {}
}

/// $+$
#[codesnip::entry("AdditiveOperation", include("algebra", "zero_one"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdditiveOperation<T: Copy + Zero + std::ops::Add<Output = T>> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("AdditiveOperation")]
mod additive_operation_impl {
    use super::*;
    use std::ops::{Add, Neg, Sub};
    impl<T: Copy + Zero + Add<Output = T>> AdditiveOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + Zero + Add<Output = T>> Magma for AdditiveOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x + *y
        }
    }
    impl<T: Copy + Zero + Add<Output = T>> Unital for AdditiveOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            Zero::zero()
        }
    }
    impl<T: Copy + Zero + Add<Output = T>> Associative for AdditiveOperation<T> {}
    impl<T: Copy + Zero + Add<Output = T>> Commutative for AdditiveOperation<T> {}
    impl<T: Copy + Zero + Add<Output = T> + Sub<Output = T> + Neg<Output = T>> Invertible
        for AdditiveOperation<T>
    {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            -*x
        }
        #[inline]
        fn rinv_operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x - *y
        }
    }
}

/// $\times$
#[codesnip::entry("MultiplicativeOperation", include("algebra", "zero_one"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MultiplicativeOperation<T: Copy + One + std::ops::Mul<Output = T>> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("MultiplicativeOperation")]
mod multiplicative_operation_impl {
    use super::*;
    use std::ops::{Div, Mul};
    impl<T: Copy + One + Mul<Output = T>> MultiplicativeOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + One + Mul<Output = T>> Magma for MultiplicativeOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x * *y
        }
    }
    impl<T: Copy + One + Mul<Output = T>> Unital for MultiplicativeOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            One::one()
        }
    }
    impl<T: Copy + One + Mul<Output = T>> Associative for MultiplicativeOperation<T> {}
    impl<T: Copy + One + Mul<Output = T>> Commutative for MultiplicativeOperation<T> {}
    impl<T: Copy + One + Mul<Output = T> + Div<Output = T>> Invertible for MultiplicativeOperation<T> {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            self.unit().div(*x)
        }
        #[inline]
        fn rinv_operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (*x).div(*y)
        }
    }
}

/// $(a, b) \circ (c, d) = \lambda x. c \times (a \times x + b) + d$
#[codesnip::entry("LinearOperation", include("algebra", "zero_one"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LinearOperation<
    T: Copy + PartialEq + Zero + std::ops::Add<Output = T> + One + std::ops::Mul<Output = T>,
> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("LinearOperation")]
mod linear_operation_impl {
    use super::*;
    use std::ops::{Add, Mul};
    impl<T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>> LinearOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>> Magma for LinearOperation<T> {
        type T = (T, T);
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (y.0 * x.0, y.0 * x.1 + y.1)
        }
    }
    impl<T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>> Unital for LinearOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            (One::one(), Zero::zero())
        }
    }
    impl<T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>> Associative for LinearOperation<T> {}
}

/// &
#[codesnip::entry("BitAndOperation", include("algebra"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitAndOperation<T: Copy + PartialEq + BitAndIdentity> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("BitAndOperation")]
pub trait BitAndIdentity: Sized + std::ops::BitAnd<Output = Self> {
    fn all_one() -> Self;
}
#[codesnip::entry("BitAndOperation")]
mod bitand_operation_impl {
    use super::*;
    #[macro_export(local_inner_macros)]
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
    impl<T: Copy + PartialEq + BitAndIdentity> BitAndOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + PartialEq + BitAndIdentity> Magma for BitAndOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x & *y
        }
    }
    impl<T: Copy + PartialEq + BitAndIdentity> Unital for BitAndOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            BitAndIdentity::all_one()
        }
    }
    impl<T: Copy + PartialEq + BitAndIdentity> Associative for BitAndOperation<T> {}
    impl<T: Copy + PartialEq + BitAndIdentity> Commutative for BitAndOperation<T> {}
    impl<T: Copy + PartialEq + BitAndIdentity> Idempotent for BitAndOperation<T> {}
}

/// |
#[codesnip::entry("BitOrOperation", include("algebra"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitOrOperation<T: Copy + PartialEq + BitOrIdentity> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("BitOrOperation")]
pub trait BitOrIdentity: Sized + std::ops::BitOr<Output = Self> {
    fn all_zero() -> Self;
}
#[codesnip::entry("BitOrOperation")]
mod bitor_operation_impl {
    use super::*;
    #[macro_export(local_inner_macros)]
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
    impl<T: Copy + PartialEq + BitOrIdentity> BitOrOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + PartialEq + BitOrIdentity> Magma for BitOrOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x | *y
        }
    }
    impl<T: Copy + PartialEq + BitOrIdentity> Unital for BitOrOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            BitOrIdentity::all_zero()
        }
    }
    impl<T: Copy + PartialEq + BitOrIdentity> Associative for BitOrOperation<T> {}
    impl<T: Copy + PartialEq + BitOrIdentity> Commutative for BitOrOperation<T> {}
    impl<T: Copy + PartialEq + BitOrIdentity> Idempotent for BitOrOperation<T> {}
}

/// ^
#[codesnip::entry("BitXorOperation", include("algebra"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitXorOperation<T: Copy + PartialEq + BitXorIdentity> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("BitXorOperation")]
pub trait BitXorIdentity: Sized + std::ops::BitXor<Output = Self> {
    fn xor_zero() -> Self;
}
#[codesnip::entry("BitXorOperation")]
mod bitxor_operation_impl {
    use super::*;
    #[macro_export(local_inner_macros)]
    macro_rules !impl_bitxor_identity {([$($wh :tt ) *] ,$t :ty ,$xor_zero :expr ) =>{impl <$($wh ) *>BitXorIdentity for $t {#[inline ] fn xor_zero () ->Self {$xor_zero } } } ;($t :ty ,$xor_zero :expr ) =>{impl BitXorIdentity for $t {#[inline ] fn xor_zero () ->Self {$xor_zero } } } ;}
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
    impl<T: Copy + PartialEq + BitXorIdentity> BitXorOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + PartialEq + BitXorIdentity> Magma for BitXorOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x ^ *y
        }
    }
    impl<T: Copy + PartialEq + BitXorIdentity> Unital for BitXorOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            BitXorIdentity::xor_zero()
        }
    }
    impl<T: Copy + PartialEq + BitXorIdentity> Associative for BitXorOperation<T> {}
    impl<T: Copy + PartialEq + BitXorIdentity> Commutative for BitXorOperation<T> {}
    impl<T: Copy + PartialEq + BitXorIdentity> Invertible for BitXorOperation<T> {
        fn inverse(&self, x: &Self::T) -> Self::T {
            *x
        }
    }
}

#[codesnip::entry("MonoidalOperation", include("algebra"))]
#[derive(Clone, Debug)]
pub struct MonoidalOperation<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    e: T,
    op: F,
}
#[codesnip::entry("MonoidalOperation")]
mod monoidal_operation_impl {
    use super::*;
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> MonoidalOperation<T, F> {
        pub fn new(e: T, op: F) -> Self {
            Self { e, op }
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Magma for MonoidalOperation<T, F> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (self.op)(x, y)
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Unital for MonoidalOperation<T, F> {
        #[inline]
        fn unit(&self) -> Self::T {
            self.e.clone()
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Associative for MonoidalOperation<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Commutative for MonoidalOperation<T, F> {}
}

#[codesnip::entry("GroupOperation", include("algebra"))]
#[derive(Clone, Debug)]
pub struct GroupOperation<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> {
    e: T,
    op: F,
    inv: G,
}
#[codesnip::entry("GroupOperation")]
mod group_operation_impl {
    use super::*;
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> GroupOperation<T, F, G> {
        pub fn new(e: T, op: F, inv: G) -> Self {
            Self { e, op, inv }
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> Magma for GroupOperation<T, F, G> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (self.op)(x, y)
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> Unital for GroupOperation<T, F, G> {
        #[inline]
        fn unit(&self) -> Self::T {
            self.e.clone()
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> Associative
        for GroupOperation<T, F, G>
    {
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> Commutative
        for GroupOperation<T, F, G>
    {
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> Invertible
        for GroupOperation<T, F, G>
    {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            (self.inv)(x)
        }
    }
}

#[codesnip::entry("AssocoativeOperator", include("algebra"))]
#[derive(Clone, Debug)]
pub struct AssocoativeOperator<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    operator: F,
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("AssocoativeOperator")]
mod assocoative_operator_impl {
    use super::*;
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Magma for AssocoativeOperator<T, F> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (self.operator)(x, y)
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Associative for AssocoativeOperator<T, F> {}
}

#[codesnip::entry("AbsorbedAssocoativeOperator", include("algebra"))]
#[derive(Clone, Debug)]
pub struct AbsorbedAssocoativeOperator<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    operator: F,
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("AbsorbedAssocoativeOperator")]
mod absorbed_assocoative_operator_impl {
    use super::*;
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> AbsorbedAssocoativeOperator<T, F> {
        pub fn new(operator: F) -> Self {
            Self {
                operator,
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Magma for AbsorbedAssocoativeOperator<T, F> {
        type T = Option<T>;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            if let (Some(a), Some(b)) = (x, y) {
                Some((self.operator)(a, b))
            } else {
                x.as_ref().or_else(|| y.as_ref()).cloned()
            }
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Unital for AbsorbedAssocoativeOperator<T, F> {
        #[inline]
        fn unit(&self) -> Self::T {
            None
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Associative for AbsorbedAssocoativeOperator<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Commutative for AbsorbedAssocoativeOperator<T, F> {}
}

/// $(M_1, M_2)$
#[codesnip::entry("CartesianOperation", include("algebra"))]
#[derive(Clone, Debug)]
pub struct CartesianOperation<M1, M2> {
    m1: M1,
    m2: M2,
}
#[codesnip::entry("CartesianOperation")]
mod cartesian_operation_impl {
    use super::*;
    impl<M1, M2> CartesianOperation<M1, M2> {
        pub fn new(m1: M1, m2: M2) -> Self {
            Self { m1, m2 }
        }
    }
    impl<M1: Magma, M2: Magma> Magma for CartesianOperation<M1, M2> {
        type T = (M1::T, M2::T);
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (self.m1.operate(&x.0, &y.0), self.m2.operate(&x.1, &y.1))
        }
    }
    impl<M1: Unital, M2: Unital> Unital for CartesianOperation<M1, M2> {
        #[inline]
        fn unit(&self) -> Self::T {
            (self.m1.unit(), self.m2.unit())
        }
    }
    impl<M1: Associative, M2: Associative> Associative for CartesianOperation<M1, M2> {}
    impl<M1: Commutative, M2: Commutative> Commutative for CartesianOperation<M1, M2> {}
    impl<M1: Invertible, M2: Invertible> Invertible for CartesianOperation<M1, M2> {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            (self.m1.inverse(&x.0), self.m2.inverse(&x.1))
        }
    }
}

#[codesnip::entry("CountingOperation", include("algebra"))]
#[derive(Clone, Debug)]
pub struct CountingOperation<M> {
    m: M,
}
#[codesnip::entry("CountingOperation")]
mod counting_operation_impl {
    use super::*;
    impl<M> CountingOperation<M> {
        pub fn new(m: M) -> Self {
            Self { m }
        }
    }
    impl<M: Magma> Magma for CountingOperation<M> {
        type T = (M::T, usize);
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            if x.0 == y.0 {
                (x.0.clone(), x.1 + y.1)
            } else {
                let z = self.m.operate(&x.0, &y.0);
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
    impl<M: Unital> Unital for CountingOperation<M> {
        #[inline]
        fn unit(&self) -> Self::T {
            (self.m.unit(), 0)
        }
    }
    impl<M: Associative> Associative for CountingOperation<M> {}
    impl<M: Commutative> Commutative for CountingOperation<M> {}
    impl<M: Idempotent> Idempotent for CountingOperation<M> {}
}

#[codesnip::entry("ReverseOperation", include("algebra"))]
#[derive(Clone, Debug)]
pub struct ReverseOperation<M> {
    m: M,
}
#[codesnip::entry("ReverseOperation")]
mod reverse_operation_impl {
    use super::*;
    impl<M> ReverseOperation<M> {
        pub fn new(m: M) -> Self {
            Self { m }
        }
    }
    impl<M: Magma> Magma for ReverseOperation<M> {
        type T = M::T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            self.m.operate(&y, &x)
        }
    }
    impl<M: Unital> Unital for ReverseOperation<M> {
        #[inline]
        fn unit(&self) -> Self::T {
            self.m.unit()
        }
    }
    impl<M: Associative> Associative for ReverseOperation<M> {}
    impl<M: Commutative> Commutative for ReverseOperation<M> {}
    impl<M: Invertible> Invertible for ReverseOperation<M> {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            self.m.inverse(&x)
        }
    }
    impl<M: Idempotent> Idempotent for ReverseOperation<M> {}
}

#[codesnip::entry("Top2Operation", include("algebra", "bounded"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Top2Operation<T: Clone + Ord + Bounded> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[codesnip::entry("Top2Operation")]
mod top2_operation_impl {
    use super::*;
    impl<T: Clone + Ord + Bounded> Top2Operation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Clone + Ord + Bounded> Magma for Top2Operation<T> {
        type T = (T, T);
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            if x.0 < y.0 {
                (y.0.clone(), if x.0 < y.1 { &y.1 } else { &x.0 }.clone())
            } else {
                (x.0.clone(), if x.1 < y.0 { &y.0 } else { &x.1 }.clone())
            }
        }
    }
    impl<T: Clone + Ord + Bounded> Unital for Top2Operation<T> {
        fn unit(&self) -> Self::T {
            (<T as Bounded>::MIN, <T as Bounded>::MIN)
        }
    }
    impl<T: Clone + Ord + Bounded> Associative for Top2Operation<T> {}
    impl<T: Clone + Ord + Bounded> Commutative for Top2Operation<T> {}
}

#[codesnip::entry("PermutationOperation", include("algebra"))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PermutationOperation {
    size: usize,
}
#[codesnip::entry("PermutationOperation")]
mod permutation_operation_impl {
    use super::*;
    impl PermutationOperation {
        pub fn new(size: usize) -> Self {
            Self { size }
        }
    }
    impl Magma for PermutationOperation {
        type T = Vec<usize>;
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            assert!(x.len() == self.size);
            assert!(y.len() == self.size);
            y.iter().map(|y| x[*y]).collect()
        }
    }
    impl Associative for PermutationOperation {}
    impl Unital for PermutationOperation {
        fn unit(&self) -> Self::T {
            (0..self.size).collect()
        }
    }
    impl Invertible for PermutationOperation {
        fn inverse(&self, x: &Self::T) -> Self::T {
            assert!(x.len() == self.size);
            let mut y = vec![0; self.size];
            for (i, x) in x.iter().enumerate() {
                y[*x] = i;
            }
            y
        }
    }
}
