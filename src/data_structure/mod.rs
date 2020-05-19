//! data structures

pub mod binary_indexed_tree;
pub mod bitset;
pub mod disjoint_sparse_table;
pub mod lazy_segment_tree;
pub mod segment_tree;
pub mod sliding_winsow_aggregation;
pub mod union_find;

/// same as [Reverse](std::cmp::Reverse)
#[cargo_snippet::snippet("Reverse")]
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Rev<T>(pub T);
#[cargo_snippet::snippet("Reverse")]
impl<T: PartialOrd> PartialOrd for Rev<T> {
    fn partial_cmp(&self, other: &Rev<T>) -> Option<std::cmp::Ordering> {
        other.0.partial_cmp(&self.0)
    }
}
#[cargo_snippet::snippet("Reverse")]
impl<T: Ord> Ord for Rev<T> {
    fn cmp(&self, other: &Rev<T>) -> std::cmp::Ordering {
        other.0.cmp(&self.0)
    }
}

/// implement Ord by PartialOrd
///
/// # Example
///
/// ```
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
