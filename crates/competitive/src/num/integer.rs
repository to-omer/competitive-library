use crate::num::Zero;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct Saturating<T>(pub T);

impl std::ops::Add for Saturating<i64> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Saturating(self.0.saturating_add(rhs.0))
    }
}
impl Zero for Saturating<i64> {
    fn zero() -> Self {
        Saturating(0)
    }
}
