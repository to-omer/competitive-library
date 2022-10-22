use super::{Invertible, Ring, SemiRing, Xorshift};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct RollingHash<R>
where
    R: SemiRing<T = u64>,
{
    base: u64,
    hash: Vec<u64>,
    pow: Vec<u64>,
    _marker: PhantomData<fn() -> R>,
}
impl<R> RollingHash<R>
where
    R: SemiRing<T = u64>,
{
    pub fn new(v: &[u64], base: u64) -> Self {
        let n = v.len();
        let mut hash = vec![R::zero(); n + 1];
        let mut pow = vec![R::one(); n + 1];
        for i in 0..n {
            hash[i + 1] = R::add(&R::mul(&hash[i], &base), &v[i]);
            pow[i + 1] = R::mul(&pow[i], &base);
        }
        Self {
            base,
            hash,
            pow,
            _marker: PhantomData,
        }
    }
    pub fn new_rand(v: &[u64], m: u64) -> Self {
        let mut rng = Xorshift::time();
        let base = rng.rand(m - 2) + 2;
        Self::new(v, base)
    }
    pub fn hash_once(&self, v: &[u64]) -> u64 {
        let mut hash = 0;
        let mut pow = 1;
        for v in v.iter() {
            hash = R::add(&R::mul(&hash, &self.base), v);
            pow = R::mul(&pow, &self.base);
        }
        hash
    }
    /// S [l, r)
    pub fn find(&self, l: usize, r: usize) -> u64
    where
        R::Additive: Invertible,
    {
        R::sub(&self.hash[r], &R::mul(&self.hash[l], &self.pow[r - l]))
    }
    pub fn concat(&self, h1: u64, h2: u64, l2: usize) -> u64 {
        R::add(&R::mul(&h1, &self.pow[l2]), &h2)
    }
}

#[derive(Clone, Debug)]
pub struct MultipleRollingHash<R>
where
    R: SemiRing<T = u64>,
{
    rh: Vec<RollingHash<R>>,
    _marker: PhantomData<fn() -> R>,
}
impl<R> MultipleRollingHash<R>
where
    R: SemiRing<T = u64>,
{
    pub fn new(v: &[u64], bases: &[u64]) -> Self {
        let rh = bases
            .iter()
            .map(|&base| RollingHash::new(v, base))
            .collect::<Vec<_>>();
        Self {
            rh,
            _marker: PhantomData,
        }
    }
    pub fn new_rand(v: &[u64], n: usize, m: u64) -> Self {
        let mut rng = Xorshift::time();
        let bases = rng
            .rands(m - 2, n)
            .into_iter()
            .map(|base| base + 2)
            .collect::<Vec<_>>();
        Self::new(v, &bases)
    }
    pub fn find(&self, l: usize, r: usize) -> Vec<u64>
    where
        R::Additive: Invertible,
    {
        self.rh.iter().map(|h| h.find(l, r)).collect::<Vec<_>>()
    }
    pub fn concat(&self, h1: &[u64], h2: &[u64], l2: usize) -> Vec<u64> {
        self.rh
            .iter()
            .zip(h1.iter().zip(h2.iter()))
            .map(|(h, (&hi1, &hi2))| h.concat(hi1, hi2, l2))
            .collect::<Vec<_>>()
    }
}
