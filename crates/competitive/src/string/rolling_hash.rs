#[codesnip::skip]
use crate::tools::Xorshift;

#[derive(Clone, Debug)]
pub struct RollingHash {
    base: u64,
    hash: Vec<u64>,
    pow: Vec<u64>,
}
impl RollingHash {
    const MASK30: u64 = (1 << 30) - 1;
    const MASK31: u64 = (1 << 31) - 1;
    const MASK61: u64 = (1 << 61) - 1;
    pub const MOD: u64 = Self::MASK61;
    #[inline]
    pub fn mersenne_mod(a: u64) -> u64 {
        let mut res = (a >> 61) + (a & Self::MASK61);
        if res >= Self::MASK61 {
            res -= Self::MASK61;
        }
        res
    }
    #[inline]
    pub fn mersenne_mul(a: u64, b: u64) -> u64 {
        let au = a >> 31;
        let ad = a & Self::MASK31;
        let bu = b >> 31;
        let bd = b & Self::MASK31;
        let mid = ad * bu + au * bd;
        let midu = mid >> 30;
        let midd = mid & Self::MASK30;
        au * bu * 2 + midu + (midd << 31) + ad * bd
    }
    #[inline]
    pub fn mersenne_mul_mod(a: u64, b: u64) -> u64 {
        Self::mersenne_mod(Self::mersenne_mul(a, b))
    }
    pub fn new(v: &[u64], base: u64) -> Self {
        let n = v.len();
        let mut hash = vec![0; n + 1];
        let mut pow = vec![1; n + 1];
        for i in 0..n {
            hash[i + 1] = Self::mersenne_mod(Self::mersenne_mul(hash[i], base) + v[i]);
            pow[i + 1] = Self::mersenne_mul_mod(pow[i], base);
        }
        Self { base, hash, pow }
    }
    pub fn hash_once(&self, v: &[u64]) -> u64 {
        let mut hash = 0;
        let mut pow = 1;
        for v in v.iter() {
            hash = Self::mersenne_mod(Self::mersenne_mul(hash, self.base) + v);
            pow = Self::mersenne_mul_mod(pow, self.base);
        }
        hash
    }
    /// S [l, r)
    pub fn find(&self, l: usize, r: usize) -> u64 {
        Self::mersenne_mod(
            self.hash[r] + Self::MOD - Self::mersenne_mul_mod(self.hash[l], self.pow[r - l]),
        )
    }
    pub fn concat(&self, h1: u64, h2: u64, l2: usize) -> u64 {
        Self::mersenne_mod(Self::mersenne_mul(h1, self.pow[l2]) + h2)
    }
}

#[derive(Clone, Debug)]
pub struct MultipleRollingHash {
    rh: Vec<RollingHash>,
}
impl MultipleRollingHash {
    pub fn new(v: &[u64], bases: &[u64]) -> Self {
        let rh = bases
            .iter()
            .map(|&base| RollingHash::new(v, base))
            .collect::<Vec<_>>();
        Self { rh }
    }
    pub fn new_rand(v: &[u64], n: usize) -> Self {
        let mut rand = Xorshift::time();
        let bases = rand
            .rands(RollingHash::MASK61 - 2, n)
            .into_iter()
            .map(|base| base + 2)
            .collect::<Vec<_>>();
        Self::new(v, &bases)
    }
    pub fn find(&self, l: usize, r: usize) -> Vec<u64> {
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
