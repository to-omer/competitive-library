use std::cmp::Ordering;

pub trait ApproxOrd {
    fn approx_eq(&self, other: &Self) -> bool;
    fn approx_cmp(&self, other: &Self) -> Ordering;
}
macro_rules! impl_approx_zero_for_int {
    ($($t:ty)*) => {
        $(impl ApproxOrd for $t {
            fn approx_eq(&self, other: &Self) -> bool {
                self.eq(other)
            }
            fn approx_cmp(&self, other: &Self) -> Ordering {
                self.cmp(other)
            }
        })*
    };
}
impl_approx_zero_for_int!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl ApproxOrd for f32 {
    fn approx_eq(&self, other: &Self) -> bool {
        const EPS_F32: f32 = 1e-8;
        (self - other).abs() < EPS_F32
    }
    fn approx_cmp(&self, other: &Self) -> Ordering {
        if self.approx_eq(other) {
            Ordering::Equal
        } else {
            let mut left = self.to_bits() as i32;
            let mut right = other.to_bits() as i32;
            left ^= (((left >> 31) as u32) >> 1) as i32;
            right ^= (((right >> 31) as u32) >> 1) as i32;
            left.cmp(&right)
        }
    }
}
impl ApproxOrd for f64 {
    fn approx_eq(&self, other: &Self) -> bool {
        const EPS_F64: f64 = 1e-8;
        (self - other).abs() < EPS_F64
    }
    fn approx_cmp(&self, other: &Self) -> Ordering {
        if self.approx_eq(other) {
            Ordering::Equal
        } else {
            let mut left = self.to_bits() as i64;
            let mut right = other.to_bits() as i64;
            left ^= (((left >> 63) as u64) >> 1) as i64;
            right ^= (((right >> 63) as u64) >> 1) as i64;
            left.cmp(&right)
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct Approx<T>(pub T)
where
    T: ApproxOrd;
impl<T> PartialEq for Approx<T>
where
    T: ApproxOrd,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.approx_eq(&other.0)
    }
}
impl<T> Eq for Approx<T> where T: ApproxOrd {}
impl<T> PartialOrd for Approx<T>
where
    T: ApproxOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.approx_cmp(&other.0))
    }
}
impl<T> Ord for Approx<T>
where
    T: ApproxOrd,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.approx_cmp(&other.0)
    }
}
