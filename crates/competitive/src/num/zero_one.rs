pub trait Zero: Sized {
    fn zero() -> Self;
    #[inline]
    fn is_zero(&self) -> bool
    where
        Self: PartialEq,
    {
        self == &Self::zero()
    }
    #[inline]
    fn set_zero(&mut self) {
        *self = Self::zero();
    }
}
pub trait One: Sized {
    fn one() -> Self;
    #[inline]
    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        self == &Self::one()
    }
    #[inline]
    fn set_one(&mut self) {
        *self = Self::one();
    }
}
macro_rules! impl_zero_one {
    ($({$Trait:ident $method:ident $($t:ty)*, $e:expr})*) => {$($(
        impl $Trait for $t {
            fn $method() -> Self {
                $e
            }
        })*)*
    };
}
impl_zero_one!(
    {Zero zero u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128, 0}
    {Zero zero f32 f64, 0.}
    {One one u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128, 1}
    {One one f32 f64, 1.}
);
