//! binary operaions

use super::base::*;

/// binary operation to select larger element
#[cargo_snippet::snippet("MaxOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaxOperation<T: Clone + Ord + MinimumBounded> {
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("MaxOperation")]
pub trait MinimumBounded {
    fn minimum() -> Self;
}
#[cargo_snippet::snippet("MaxOperation")]
pub mod max_operation_impl {
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
    impl<T: Clone + Ord + MinimumBounded> MaxOperation<T> {
        pub fn new() -> Self {
            MaxOperation {
                phantom: std::marker::PhantomData,
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
    impl<T: Clone + Ord + MinimumBounded> SemiGroup for MaxOperation<T> {}
    impl<T: Clone + Ord + MinimumBounded> Monoid for MaxOperation<T> {}
    impl<T: Clone + Ord + MinimumBounded> Commutative for MaxOperation<T> {}
    impl<T: Clone + Ord + MinimumBounded> AbelianMonoid for MaxOperation<T> {}
    impl<T: Clone + Ord + MinimumBounded> Idempotent for MaxOperation<T> {}
    impl<T: Clone + Ord + MinimumBounded> IdempotentMonoid for MaxOperation<T> {}
}

/// binary operation to select smaller element
#[cargo_snippet::snippet("MinOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MinOperation<T: Clone + Ord + MaximumBounded> {
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("MinOperation")]
pub trait MaximumBounded {
    fn maximum() -> Self;
}
#[cargo_snippet::snippet("MinOperation")]
pub mod min_operation_impl {
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
    impl<T: Clone + Ord + MaximumBounded> MinOperation<T> {
        pub fn new() -> Self {
            MinOperation {
                phantom: std::marker::PhantomData,
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
    impl<T: Clone + Ord + MaximumBounded> SemiGroup for MinOperation<T> {}
    impl<T: Clone + Ord + MaximumBounded> Monoid for MinOperation<T> {}
    impl<T: Clone + Ord + MaximumBounded> Commutative for MinOperation<T> {}
    impl<T: Clone + Ord + MaximumBounded> AbelianMonoid for MinOperation<T> {}
    impl<T: Clone + Ord + MaximumBounded> Idempotent for MinOperation<T> {}
    impl<T: Clone + Ord + MaximumBounded> IdempotentMonoid for MinOperation<T> {}
}

/// retain the first element
#[cargo_snippet::snippet("FirstOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FirstOperation<T: Clone + PartialEq> {
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("FirstOperation")]
pub mod first_operation_impl {
    use super::*;
    impl<T: Clone + PartialEq> FirstOperation<T> {
        pub fn new() -> Self {
            FirstOperation {
                phantom: std::marker::PhantomData,
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
    impl<T: Clone + PartialEq> SemiGroup for FirstOperation<T> {}
    impl<T: Clone + PartialEq> Monoid for FirstOperation<T> {}
    impl<T: Clone + PartialEq> Idempotent for FirstOperation<T> {}
    impl<T: Clone + PartialEq> IdempotentMonoid for FirstOperation<T> {}
}

/// retain the last element
#[cargo_snippet::snippet("LastOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LastOperation<T: Clone + PartialEq> {
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("LastOperation")]
pub mod last_operation_impl {
    use super::*;
    impl<T: Clone + PartialEq> LastOperation<T> {
        pub fn new() -> Self {
            LastOperation {
                phantom: std::marker::PhantomData,
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
    impl<T: Clone + PartialEq> SemiGroup for LastOperation<T> {}
    impl<T: Clone + PartialEq> Monoid for LastOperation<T> {}
    impl<T: Clone + PartialEq> Idempotent for LastOperation<T> {}
    impl<T: Clone + PartialEq> IdempotentMonoid for LastOperation<T> {}
}

/// +
#[cargo_snippet::snippet("AdditiveOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdditiveOperation<T: Copy + PartialEq + AdditiveIdentity> {
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("AdditiveOperation")]
pub trait AdditiveIdentity: Sized + std::ops::Add<Output = Self> {
    fn zero() -> Self;
}
#[cargo_snippet::snippet("AdditiveOperation")]
#[macro_use]
pub mod additive_operation_impl {
    use super::*;
    #[macro_export(local_inner_macros)]
    macro_rules! impl_additive_identity {
        ([$($wh:tt)*], $t:ty, $zero:expr) => {
            impl<$($wh)*> AdditiveIdentity for $t {
                #[inline]
                fn zero() -> Self {
                    $zero
                }
            }
        };
        ($t:ty, $zero:expr) => {
            impl AdditiveIdentity for $t {
                #[inline]
                fn zero() -> Self {
                    $zero
                }
            }
        };
    }
    impl_additive_identity!(usize, 0usize);
    impl_additive_identity!(u8, 0u8);
    impl_additive_identity!(u16, 0u16);
    impl_additive_identity!(u32, 0u32);
    impl_additive_identity!(u64, 0u64);
    impl_additive_identity!(isize, 0isize);
    impl_additive_identity!(i8, 0i8);
    impl_additive_identity!(i16, 0i16);
    impl_additive_identity!(i32, 0i32);
    impl_additive_identity!(i64, 0i64);
    impl_additive_identity!(f32, 0.0f32);
    impl_additive_identity!(f64, 0.0f64);
    impl<T: Copy + PartialEq + AdditiveIdentity> AdditiveOperation<T> {
        pub fn new() -> Self {
            AdditiveOperation {
                phantom: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + PartialEq + AdditiveIdentity> Magma for AdditiveOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x + *y
        }
    }
    impl<T: Copy + PartialEq + AdditiveIdentity> Unital for AdditiveOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            AdditiveIdentity::zero()
        }
    }
    impl<T: Copy + PartialEq + AdditiveIdentity> Associative for AdditiveOperation<T> {}
    impl<T: Copy + PartialEq + AdditiveIdentity> SemiGroup for AdditiveOperation<T> {}
    impl<T: Copy + PartialEq + AdditiveIdentity> Monoid for AdditiveOperation<T> {}
    impl<T: Copy + PartialEq + AdditiveIdentity> Commutative for AdditiveOperation<T> {}
    impl<T: Copy + PartialEq + AdditiveIdentity> AbelianMonoid for AdditiveOperation<T> {}
    impl<T: Copy + PartialEq + AdditiveIdentity + std::ops::Neg<Output = T>> Invertible
        for AdditiveOperation<T>
    {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            -*x
        }
    }
    impl<T: Copy + PartialEq + AdditiveIdentity + std::ops::Neg<Output = T>> Group
        for AdditiveOperation<T>
    {
    }
    impl<T: Copy + PartialEq + AdditiveIdentity + std::ops::Neg<Output = T>> AbelianGroup
        for AdditiveOperation<T>
    {
    }
}

/// ×
#[cargo_snippet::snippet("MultiplicativeOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MultiplicativeOperation<T: Copy + PartialEq + MultiplicativeIdentity> {
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("MultiplicativeOperation")]
pub trait MultiplicativeIdentity: Sized + std::ops::Mul<Output = Self> {
    fn one() -> Self;
}
#[cargo_snippet::snippet("MultiplicativeOperation")]
#[macro_use]
pub mod multiplicative_operation_impl {
    use super::*;
    #[macro_export(local_inner_macros)]
    macro_rules! impl_multiplicative_identity {
        ([$($wh:tt)*], $t:ty, $one:expr) => {
            impl<$($wh)*> MultiplicativeIdentity for $t {
                #[inline]
                fn one() -> Self {
                    $one
                }
            }
        };
        ($t:ty, $one:expr) => {
            impl MultiplicativeIdentity for $t {
                #[inline]
                fn one() -> Self {
                    $one
                }
            }
        };
    }
    impl_multiplicative_identity!(usize, 1usize);
    impl_multiplicative_identity!(u8, 1u8);
    impl_multiplicative_identity!(u16, 1u16);
    impl_multiplicative_identity!(u32, 1u32);
    impl_multiplicative_identity!(u64, 1u64);
    impl_multiplicative_identity!(isize, 1isize);
    impl_multiplicative_identity!(i8, 1i8);
    impl_multiplicative_identity!(i16, 1i16);
    impl_multiplicative_identity!(i32, 1i32);
    impl_multiplicative_identity!(i64, 1i64);
    impl_multiplicative_identity!(f32, 1.0f32);
    impl_multiplicative_identity!(f64, 1.0f64);
    impl<T: Copy + PartialEq + MultiplicativeIdentity> MultiplicativeOperation<T> {
        pub fn new() -> Self {
            MultiplicativeOperation {
                phantom: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + PartialEq + MultiplicativeIdentity> Magma for MultiplicativeOperation<T> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            *x * *y
        }
    }
    impl<T: Copy + PartialEq + MultiplicativeIdentity> Unital for MultiplicativeOperation<T> {
        #[inline]
        fn unit(&self) -> Self::T {
            MultiplicativeIdentity::one()
        }
    }
    impl<T: Copy + PartialEq + MultiplicativeIdentity> Associative for MultiplicativeOperation<T> {}
    impl<T: Copy + PartialEq + MultiplicativeIdentity> SemiGroup for MultiplicativeOperation<T> {}
    impl<T: Copy + PartialEq + MultiplicativeIdentity> Monoid for MultiplicativeOperation<T> {}
    impl<T: Copy + PartialEq + MultiplicativeIdentity> Commutative for MultiplicativeOperation<T> {}
    impl<T: Copy + PartialEq + MultiplicativeIdentity> AbelianMonoid for MultiplicativeOperation<T> {}
    impl<T: Copy + PartialEq + MultiplicativeIdentity + std::ops::Div<Output = T>> Invertible
        for MultiplicativeOperation<T>
    {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            self.unit().div(*x)
        }
    }
    impl<T: Copy + PartialEq + MultiplicativeIdentity + std::ops::Div<Output = T>> Group
        for MultiplicativeOperation<T>
    {
    }
    impl<T: Copy + PartialEq + MultiplicativeIdentity + std::ops::Div<Output = T>> AbelianGroup
        for MultiplicativeOperation<T>
    {
    }
}

/// (a, b) ∘ (c, d) = λx. c × (a × x + b) + d
#[cargo_snippet::snippet("LinearOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LinearOperation<T: Copy + PartialEq + AdditiveIdentity + MultiplicativeIdentity> {
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("LinearOperation")]
pub mod linear_operation_impl {
    use super::*;
    impl<T: Copy + PartialEq + AdditiveIdentity + MultiplicativeIdentity> LinearOperation<T> {
        pub fn new() -> Self {
            LinearOperation {
                phantom: std::marker::PhantomData,
            }
        }
    }
    impl<T: Copy + PartialEq + AdditiveIdentity + MultiplicativeIdentity> Magma for LinearOperation<T> {
        type T = (T, T);
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (y.0 * x.0, y.0 * x.1 + y.1)
        }
    }
    impl<T: Copy + PartialEq + AdditiveIdentity + MultiplicativeIdentity> Unital
        for LinearOperation<T>
    {
        #[inline]
        fn unit(&self) -> Self::T {
            (MultiplicativeIdentity::one(), AdditiveIdentity::zero())
        }
    }
    impl<T: Copy + PartialEq + AdditiveIdentity + MultiplicativeIdentity> Associative
        for LinearOperation<T>
    {
    }
    impl<T: Copy + PartialEq + AdditiveIdentity + MultiplicativeIdentity> SemiGroup
        for LinearOperation<T>
    {
    }
    impl<T: Copy + PartialEq + AdditiveIdentity + MultiplicativeIdentity> Monoid
        for LinearOperation<T>
    {
    }
}

/// &
#[cargo_snippet::snippet("BitAndOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitAndOperation<T: Copy + PartialEq + BitAndIdentity> {
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("BitAndOperation")]
pub trait BitAndIdentity: Sized + std::ops::BitAnd<Output = Self> {
    fn all_one() -> Self;
}
#[cargo_snippet::snippet("BitAndOperation")]
pub mod bitand_operation_impl {
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
            BitAndOperation {
                phantom: std::marker::PhantomData,
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
    impl<T: Copy + PartialEq + BitAndIdentity> SemiGroup for BitAndOperation<T> {}
    impl<T: Copy + PartialEq + BitAndIdentity> Monoid for BitAndOperation<T> {}
    impl<T: Copy + PartialEq + BitAndIdentity> Commutative for BitAndOperation<T> {}
    impl<T: Copy + PartialEq + BitAndIdentity> AbelianMonoid for BitAndOperation<T> {}
    impl<T: Copy + PartialEq + BitAndIdentity> Idempotent for BitAndOperation<T> {}
    impl<T: Copy + PartialEq + BitAndIdentity> IdempotentMonoid for BitAndOperation<T> {}
}

/// |
#[cargo_snippet::snippet("BitOrOperation")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitOrOperation<T: Copy + PartialEq + BitOrIdentity> {
    phantom: std::marker::PhantomData<T>,
}
pub trait BitOrIdentity: Sized + std::ops::BitOr<Output = Self> {
    fn all_zero() -> Self;
}
#[cargo_snippet::snippet("BitOrOperation")]
pub mod bitor_operation_impl {
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
            BitOrOperation {
                phantom: std::marker::PhantomData,
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
    impl<T: Copy + PartialEq + BitOrIdentity> SemiGroup for BitOrOperation<T> {}
    impl<T: Copy + PartialEq + BitOrIdentity> Monoid for BitOrOperation<T> {}
    impl<T: Copy + PartialEq + BitOrIdentity> Commutative for BitOrOperation<T> {}
    impl<T: Copy + PartialEq + BitOrIdentity> AbelianMonoid for BitOrOperation<T> {}
    impl<T: Copy + PartialEq + BitOrIdentity> Idempotent for BitOrOperation<T> {}
    impl<T: Copy + PartialEq + BitOrIdentity> IdempotentMonoid for BitOrOperation<T> {}
}

#[cargo_snippet::snippet("MonoidalOperation")]
#[derive(Clone, Debug)]
pub struct MonoidalOperation<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    e: T,
    op: F,
}
#[cargo_snippet::snippet("MonoidalOperation")]
pub mod monoidal_operation_impl {
    use super::*;
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> MonoidalOperation<T, F> {
        pub fn new(e: T, op: F) -> Self {
            MonoidalOperation { e: e, op: op }
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
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> SemiGroup for MonoidalOperation<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Monoid for MonoidalOperation<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Commutative for MonoidalOperation<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> AbelianMonoid for MonoidalOperation<T, F> {}
}

#[cargo_snippet::snippet("GroupOperation")]
#[derive(Clone, Debug)]
pub struct GroupOperation<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> {
    e: T,
    op: F,
    inv: G,
}
#[cargo_snippet::snippet("GroupOperation")]
pub mod group_operation_impl {
    use super::*;
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> GroupOperation<T, F, G> {
        pub fn new(e: T, op: F, inv: G) -> Self {
            GroupOperation {
                e: e,
                op: op,
                inv: inv,
            }
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
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> SemiGroup
        for GroupOperation<T, F, G>
    {
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> Monoid for GroupOperation<T, F, G> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> Commutative
        for GroupOperation<T, F, G>
    {
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> AbelianMonoid
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
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> Group for GroupOperation<T, F, G> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T, G: Fn(&T) -> T> AbelianGroup
        for GroupOperation<T, F, G>
    {
    }
}

#[cargo_snippet::snippet("AssocoativeOperator")]
#[derive(Clone, Debug)]
pub struct AssocoativeOperator<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    operator: F,
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("AssocoativeOperator")]
pub mod assocoative_operator_impl {
    use super::*;
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Magma for AssocoativeOperator<T, F> {
        type T = T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            (self.operator)(x, y)
        }
    }
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Associative for AssocoativeOperator<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> SemiGroup for AssocoativeOperator<T, F> {}
}

#[cargo_snippet::snippet("AbsorbedAssocoativeOperator")]
#[derive(Clone, Debug)]
pub struct AbsorbedAssocoativeOperator<T: Clone + PartialEq, F: Fn(&T, &T) -> T> {
    operator: F,
    phantom: std::marker::PhantomData<T>,
}
#[cargo_snippet::snippet("AbsorbedAssocoativeOperator")]
pub mod absorbed_assocoative_operator_impl {
    use super::*;
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> AbsorbedAssocoativeOperator<T, F> {
        pub fn new(operator: F) -> Self {
            AbsorbedAssocoativeOperator {
                operator: operator,
                phantom: std::marker::PhantomData,
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
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> SemiGroup for AbsorbedAssocoativeOperator<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Monoid for AbsorbedAssocoativeOperator<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> Commutative for AbsorbedAssocoativeOperator<T, F> {}
    impl<T: Clone + PartialEq, F: Fn(&T, &T) -> T> AbelianMonoid for AbsorbedAssocoativeOperator<T, F> {}
}

/// (M1, M2)
#[cargo_snippet::snippet("CartesianOperation")]
#[derive(Clone, Debug)]
pub struct CartesianOperation<M1, M2> {
    m1: M1,
    m2: M2,
}
#[cargo_snippet::snippet("CartesianOperation")]
pub mod cartesian_operation_impl {
    use super::*;
    impl<M1, M2> CartesianOperation<M1, M2> {
        pub fn new(m1: M1, m2: M2) -> Self {
            CartesianOperation { m1: m1, m2: m2 }
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
    impl<M1: SemiGroup, M2: SemiGroup> SemiGroup for CartesianOperation<M1, M2> {}
    impl<M1: Monoid, M2: Monoid> Monoid for CartesianOperation<M1, M2> {}
    impl<M1: Commutative, M2: Commutative> Commutative for CartesianOperation<M1, M2> {}
    impl<M1: AbelianMonoid, M2: AbelianMonoid> AbelianMonoid for CartesianOperation<M1, M2> {}
    impl<M1: Invertible, M2: Invertible> Invertible for CartesianOperation<M1, M2> {
        #[inline]
        fn inverse(&self, x: &Self::T) -> Self::T {
            (self.m1.inverse(&x.0), self.m2.inverse(&x.1))
        }
    }
    impl<M1: Group, M2: Group> Group for CartesianOperation<M1, M2> {}
    impl<M1: AbelianGroup, M2: AbelianGroup> AbelianGroup for CartesianOperation<M1, M2> {}
}

#[cargo_snippet::snippet("CountingOperation")]
#[derive(Clone, Debug)]
pub struct CountingOperation<M> {
    m: M,
}
#[cargo_snippet::snippet("CountingOperation")]
pub mod counting_operation_impl {
    use super::*;
    impl<M> CountingOperation<M> {
        pub fn new(m: M) -> Self {
            CountingOperation { m: m }
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
    impl<M: SemiGroup> SemiGroup for CountingOperation<M> {}
    impl<M: Monoid> Monoid for CountingOperation<M> {}
    impl<M: Commutative> Commutative for CountingOperation<M> {}
    impl<M: AbelianMonoid> AbelianMonoid for CountingOperation<M> {}
    impl<M: Idempotent> Idempotent for CountingOperation<M> {}
    impl<M: IdempotentMonoid> IdempotentMonoid for CountingOperation<M> {}
}
