use super::SliceBisectExt;
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    hash::Hash,
    iter::FromIterator,
};

pub trait Compressor<T>
where
    Self: FromIterator<T>,
    T: Ord,
{
    fn index_exact(&self, index: &T) -> Option<usize>;
    fn index_lower_bound(&self, index: &T) -> usize;
    fn size(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct VecCompress<T> {
    data: Vec<T>,
}

impl<T> FromIterator<T> for VecCompress<T>
where
    T: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut data: Vec<_> = iter.into_iter().collect();
        data.sort_unstable();
        data.dedup();
        Self { data }
    }
}

impl<T> Compressor<T> for VecCompress<T>
where
    T: Ord,
{
    fn index_exact(&self, index: &T) -> Option<usize> {
        self.data.binary_search(index).ok()
    }

    fn index_lower_bound(&self, index: &T) -> usize {
        self.data.position_bisect(|x| x >= index)
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

#[derive(Clone)]
pub struct HashCompress<T> {
    data: HashMap<T, usize>,
}

impl<T> Debug for HashCompress<T>
where
    T: Debug + Eq + Hash,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashCompress")
            .field("data", &self.data)
            .finish()
    }
}

impl<T> FromIterator<T> for HashCompress<T>
where
    T: Ord + Hash,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut data: Vec<_> = iter.into_iter().collect();
        data.sort_unstable();
        data.dedup();
        let data = data.into_iter().enumerate().map(|(i, t)| (t, i)).collect();
        Self { data }
    }
}

impl<T> Compressor<T> for HashCompress<T>
where
    T: Ord + Hash,
{
    fn index_exact(&self, index: &T) -> Option<usize> {
        self.data.get(index).copied()
    }

    fn index_lower_bound(&self, _index: &T) -> usize {
        panic!("HashCompress does not implement index_lower_bound")
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}
