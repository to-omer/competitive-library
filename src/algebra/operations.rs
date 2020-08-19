//! binary operaions

use super::magma::*;
use crate::num::{One, Zero};

/// binary operation to select larger element
#[cargo_snippet::snippet("MaxOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaxOperation<T: Clone + Ord + MinimumBounded> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("MaxOperation")]
pub trait MinimumBounded {
    fn minimum() -> Self;
}
#[cargo_snippet::snippet("MaxOperation")]
mod max_operation_impl {
    use super::*;
    macro_rules! impl_minimum_with_min {
        ([$($wh:tt)*], $t:ty, $min:expr) => {
            impl<$($wh)*> MinimumBounded for $t {
                #[inline]
                fn minimum() -> Self {
                    $min
                }
            }
        };
        ($t:ty, $min:expr) => {
            impl MinimumBounded for $t {
                #[inline]
                fn minimum() -> Self {
                    $min
                }
            }
        };
    }
    impl_minimum_with_min!(usize, std::usize::MIN);
    impl_minimum_with_min!(u8, std::u8::MIN);
    impl_minimum_with_min!(u16, std::u16::MIN);
    impl_minimum_with_min!(u32, std::u32::MIN);
    impl_minimum_with_min!(u64, std::u64::MIN);
    impl_minimum_with_min!(isize, std::isize::MIN);
    impl_minimum_with_min!(i8, std::i8::MIN);
    impl_minimum_with_min!(i16, std::i16::MIN);
    impl_minimum_with_min!(i32, std::i32::MIN);
    impl_minimum_with_min!(i64, std::i64::MIN);
    // impl_minimum_with_min!(f32, std::f32::MIN);
    // impl_minimum_with_min!(f64, std::f64::MIN);
    macro_rules! impl_minimum_tuple {
        ($($T:ident)*) => {
            impl<$($T: MinimumBounded),*> MinimumBounded for ($($T,)*) {
                #[inline]
                fn minimum() -> Self {
                    ($(<$T as MinimumBounded>::minimum(),)*)
                }
            }
        };
    }
    impl_minimum_tuple!();
    impl_minimum_tuple!(A);
    impl_minimum_tuple!(A B);
    impl_minimum_tuple!(A B C);
    impl_minimum_tuple!(A B C D);
    impl_minimum_tuple!(A B C D E);
    impl_minimum_tuple!(A B C D E F);
    impl_minimum_tuple!(A B C D E F G);
    impl_minimum_tuple!(A B C D E F G H);
    impl_minimum_tuple!(A B C D E F G H I);
    impl_minimum_tuple!(A B C D E F G H I J);
    impl_minimum_tuple!(A B C D E F G H I J K);
    impl<T: Clone + Ord + MinimumBounded> MaxOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Clone + Ord + MinimumBounded> Magma for MaxOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            std::cmp::max(x, y).clone()
        }
    }
    impl<T: Clone + Ord + MinimumBounded> Unital for MaxOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            MinimumBounded::minimum()
        }
    }
    impl<T: Clone + Ord + MinimumBounded> Associative for MaxOperation<T> {}
    impl<T: Clone + Ord + MinimumBounded> Commutative for MaxOperation<T> {}
    impl<T: Clone + Ord + MinimumBounded> Idempotent for MaxOperation<T> {}
}

/// binary operation to select smaller element
#[cargo_snippet::snippet("MinOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MinOperation<T: Clone + Ord + MaximumBounded> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("MinOperation")]
pub trait MaximumBounded {
    fn maximum() -> Self;
}
#[cargo_snippet::snippet("MinOperation")]
mod min_operation_impl {
    use super::*;
    macro_rules! impl_maximum_with_max {
        ([$($wh:tt)*], $t:ty, $max:expr) => {
            impl<$($wh)*> MaximumBounded for $t {
                #[inline]
                fn maximum() -> Self {
                    $max
                }
            }
        };
        ($t:ty, $max:expr) => {
            impl MaximumBounded for $t {
                #[inline]
                fn maximum() -> Self {
                    $max
                }
            }
        };
    }
    impl_maximum_with_max!(usize, std::usize::MAX);
    impl_maximum_with_max!(u8, std::u8::MAX);
    impl_maximum_with_max!(u16, std::u16::MAX);
    impl_maximum_with_max!(u32, std::u32::MAX);
    impl_maximum_with_max!(u64, std::u64::MAX);
    impl_maximum_with_max!(isize, std::isize::MAX);
    impl_maximum_with_max!(i8, std::i8::MAX);
    impl_maximum_with_max!(i16, std::i16::MAX);
    impl_maximum_with_max!(i32, std::i32::MAX);
    impl_maximum_with_max!(i64, std::i64::MAX);
    // impl_maximum_with_max!(f32, std::f32::MAX);
    // impl_maximum_with_max!(f64, std::f64::MAX);
    macro_rules! impl_maximum_tuple {
        ($($T:ident)*) => {
            impl<$($T: MaximumBounded),*> MaximumBounded for ($($T,)*) {
                #[inline]
                fn maximum() -> Self {
                    ($(<$T as MaximumBounded>::maximum(),)*)
                }
            }
        };
    }
    impl_maximum_tuple!();
    impl_maximum_tuple!(A);
    impl_maximum_tuple!(A B);
    impl_maximum_tuple!(A B C);
    impl_maximum_tuple!(A B C D);
    impl_maximum_tuple!(A B C D E);
    impl_maximum_tuple!(A B C D E F);
    impl_maximum_tuple!(A B C D E F G);
    impl_maximum_tuple!(A B C D E F G H);
    impl_maximum_tuple!(A B C D E F G H I);
    impl_maximum_tuple!(A B C D E F G H I J);
    impl_maximum_tuple!(A B C D E F G H I J K);
    impl<T: Clone + Ord + MaximumBounded> MinOperation<T> {
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
    impl<T: Clone + Ord + MaximumBounded> Magma for MinOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            std::cmp::min(x, y).clone()
        }
    }
    impl<T: Clone + Ord + MaximumBounded> Unital for MinOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            MaximumBounded::maximum()
        }
    }
    impl<T: Clone + Ord + MaximumBounded> Associative for MinOperation<T> {}
    impl<T: Clone + Ord + MaximumBounded> Commutative for MinOperation<T> {}
    impl<T: Clone + Ord + MaximumBounded> Idempotent for MinOperation<T> {}
}

/// retain the first element
#[cargo_snippet::snippet("FirstOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FirstOperation<T: Clone + PartialEq> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("FirstOperation")]
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
            x.as_ref().or(y.as_ref()).cloned()
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
#[cargo_snippet::snippet("LastOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LastOperation<T: Clone + PartialEq> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("LastOperation")]
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
            y.as_ref().or(x.as_ref()).cloned()
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
#[cargo_snippet::snippet("AdditiveOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdditiveOperation<T: Copy + Zero + std::ops::Add<Output = T>> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("AdditiveOperation")]
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
    impl<T: Copy + Zero + Add<Output = T> + Neg<Output = T>> Invertible for AdditiveOperation<T> {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            -*x
        }
    }
    impl<T: Copy + Zero + Add<Output = T> + Sub<Output = T>> RightInvertibleMagma
        for AdditiveOperation<T>
    {
        #[inline]
        fn rinv_operation(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x - *y
        }
    }
}

/// $\times$
#[cargo_snippet::snippet("MultiplicativeOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MultiplicativeOperation<T: Copy + One + std::ops::Mul<Output = T>> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("MultiplicativeOperation")]
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
    }
    impl<T: Copy + One + Mul<Output = T> + Div<Output = T>> RightInvertibleMagma
        for MultiplicativeOperation<T>
    {
        #[inline]
        fn rinv_operation(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (*x).div(*y)
        }
    }
}

/// $(a, b) \circ (c, d) = \lambda x. c \times (a \times x + b) + d$
#[cargo_snippet::snippet("LinearOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LinearOperation<
    T: Copy + PartialEq + Zero + std::ops::Add<Output = T> + One + std::ops::Mul<Output = T>,
> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("LinearOperation")]
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
#[cargo_snippet::snippet("BitAndOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitAndOperation<T: Copy + PartialEq + BitAndIdentity> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("BitAndOperation")]
pub trait BitAndIdentity: Sized + std::ops::BitAnd<Output = Self> {
    fn all_one() -> Self;
}
#[cargo_snippet::snippet("BitAndOperation")]
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
#[cargo_snippet::snippet("BitOrOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitOrOperation<T: Copy + PartialEq + BitOrIdentity> {
    _marker: std::marker::PhantomData<fn() -> T>,
}
pub trait BitOrIdentity: Sized + std::ops::BitOr<Output = Self> {
    fn all_zero() -> Self;
}
#[cargo_snippet::snippet("BitOrOperation")]
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

#[cargo_snippet::snippet("MonoidalOperation")]
#[derive(Clone, Debug)]
pub struct MonoidalOperation<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    e: T,
    op: F,
}
#[cargo_snippet::snippet("MonoidalOperation")]
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

#[cargo_snippet::snippet("GroupOperation")]
#[derive(Clone, Debug)]
pub struct GroupOperation<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> {
    e: T,
    op: F,
    inv: G,
}
#[cargo_snippet::snippet("GroupOperation")]
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

#[cargo_snippet::snippet("AssocoativeOperator")]
#[derive(Clone, Debug)]
pub struct AssocoativeOperator<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    operator: F,
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("AssocoativeOperator")]
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

#[cargo_snippet::snippet("AbsorbedAssocoativeOperator")]
#[derive(Clone, Debug)]
pub struct AbsorbedAssocoativeOperator<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    operator: F,
    _marker: std::marker::PhantomData<fn() -> T>,
}
#[cargo_snippet::snippet("AbsorbedAssocoativeOperator")]
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
                x.as_ref().or(y.as_ref()).cloned()
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
#[cargo_snippet::snippet("CartesianOperation")]
#[derive(Clone, Debug)]
pub struct CartesianOperation<M1, M2> {
    m1: M1,
    m2: M2,
}
#[cargo_snippet::snippet("CartesianOperation")]
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

#[cargo_snippet::snippet("CountingOperation")]
#[derive(Clone, Debug)]
pub struct CountingOperation<M> {
    m: M,
}
#[cargo_snippet::snippet("CountingOperation")]
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

#[cargo_snippet::snippet("ReverseOperation")]
#[derive(Clone, Debug)]
pub struct ReverseOperation<M> {
    m: M,
}
#[cargo_snippet::snippet("ReverseOperation")]
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
