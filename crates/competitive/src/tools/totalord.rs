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
#[derive(PartialEq)]
pub struct TotalOrd<T>(pub T);
impl<T: PartialEq> Eq for TotalOrd<T> {}
impl<T: PartialOrd> PartialOrd for TotalOrd<T> {
    fn partial_cmp(&self, other: &TotalOrd<T>) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl<T: PartialOrd> Ord for TotalOrd<T> {
    fn cmp(&self, other: &TotalOrd<T>) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
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
