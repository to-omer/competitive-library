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
    fn size(&self) -> usize;
}

pub trait OrderedCompressor<T>: Compressor<T>
where
    T: Ord,
{
    fn index_lower_bound(&self, index: &T) -> usize;
}

#[derive(Debug, Clone)]
pub struct VecCompress<T> {
    data: Vec<T>,
}

impl<T> VecCompress<T> {
    pub fn from_sorted_unique(data: Vec<T>) -> Self {
        Self { data }
    }

    pub fn values(&self) -> &[T] {
        &self.data
    }
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

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl<T> OrderedCompressor<T> for VecCompress<T>
where
    T: Ord,
{
    fn index_lower_bound(&self, index: &T) -> usize {
        self.data.partition_point(|x| x < index)
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

    fn size(&self) -> usize {
        self.data.len()
    }
}
