use crate::algorithm::search::{lower_bound, Bisect};
use cargo_snippet::snippet;

#[snippet("Compress")]
#[derive(Clone, Debug)]
pub struct Compress<T> {
    v: Vec<T>,
}
#[snippet("Compress")]
#[snippet(include = "binary_search")]
impl<T: Bisect + Ord> Compress<T> {
    pub fn get(&self, x: T) -> usize {
        lower_bound(&self.v, x)
    }
}
#[snippet("Compress")]
impl<T> Compress<T> {
    pub fn len(&self) -> usize {
        self.v.len()
    }
}
#[snippet("Compress")]
impl<T> std::ops::Index<usize> for Compress<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    }
}
#[snippet("Compress")]
impl<T: Ord> std::iter::FromIterator<T> for Compress<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Compress<T> {
        use std::collections::BTreeSet;
        let v = iter
            .into_iter()
            .collect::<BTreeSet<T>>()
            .into_iter()
            .collect::<Vec<T>>();
        Compress { v: v }
    }
}
