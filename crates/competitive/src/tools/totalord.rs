use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Copy, Default)]
/// implement Ord by PartialOrd
///
/// # Example
///
/// ```
/// # use competitive::tools::TotalOrd;
/// let mut a = vec![3.1, 4.1, 5.9, 2.6];
/// a.sort_by_key(|&x| TotalOrd(x));
/// ```
///
pub struct TotalOrd<T>(pub T);
impl<T> PartialEq for TotalOrd<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for TotalOrd<T> where T: PartialEq {}
impl<T> PartialOrd for TotalOrd<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &TotalOrd<T>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl<T> Ord for TotalOrd<T>
where
    T: PartialOrd,
{
    fn cmp(&self, other: &TotalOrd<T>) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl<T> Hash for TotalOrd<T>
where
    T: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
    }
}
pub trait AsTotalOrd {
    fn as_total_ord(&self) -> TotalOrd<&Self>;
}
impl<T: PartialOrd> AsTotalOrd for T {
    fn as_total_ord(&self) -> TotalOrd<&Self> {
        TotalOrd(self)
    }
}
