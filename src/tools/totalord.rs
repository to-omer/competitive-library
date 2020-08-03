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
#[cargo_snippet::snippet("TotalOrd")]
#[derive(PartialEq, PartialOrd)]
pub struct TotalOrd<T>(pub T);
#[cargo_snippet::snippet("TotalOrd")]
impl<T: PartialEq> Eq for TotalOrd<T> {}
#[cargo_snippet::snippet("TotalOrd")]
impl<T: PartialOrd> Ord for TotalOrd<T> {
    fn cmp(&self, other: &TotalOrd<T>) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}
