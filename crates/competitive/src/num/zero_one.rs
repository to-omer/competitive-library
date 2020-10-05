#[snippet::entry("zero_one")]
pub trait Zero: PartialEq + Sized {
    fn zero() -> Self;
    #[inline]
    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}
#[snippet::entry("zero_one")]
pub trait One: PartialEq + Sized {
    fn one() -> Self;
    #[inline]
    fn is_one(&self) -> bool {
        self == &Self::one()
    }
}
#[snippet::entry("zero_one")]
macro_rules! zero_one_impls {
    ($({$Trait:ident $method:ident $($t:ty)*, $e:expr})*) => {$($(
        impl $Trait for $t {
            #[inline]
            fn $method() -> Self {
                $e
            }
        })*)*
    };
}
#[snippet::entry("zero_one")]
zero_one_impls!(
    {Zero zero u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128, 0}
    {Zero zero f32 f64, 0.}
    {One one u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128, 1}
    {One one f32 f64, 1.}
);
