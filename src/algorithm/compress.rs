//! coordinate compression

#[cargo_snippet::snippet("Compress")]
#[derive(Clone, Debug)]
pub struct Compress<T> {
    v: Vec<T>,
}
#[cargo_snippet::snippet("Compress")]
#[cargo_snippet::snippet(include = "binary_search")]
impl<T: Ord> Compress<T> {
    pub fn get(&self, x: &T) -> usize {
        self.v.binary_search(&x).unwrap()
    }
}
#[cargo_snippet::snippet("Compress")]
impl<T> Compress<T> {
    pub fn len(&self) -> usize {
        self.v.len()
    }
}
#[cargo_snippet::snippet("Compress")]
impl<T> std::ops::Index<usize> for Compress<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    }
}
#[cargo_snippet::snippet("Compress")]
impl<T: Ord> std::iter::FromIterator<T> for Compress<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        use std::collections::BTreeSet;
        let v = iter
            .into_iter()
            .collect::<BTreeSet<T>>()
            .into_iter()
            .collect::<Vec<T>>();
        Self { v }
    }
}
