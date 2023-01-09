use super::{Gf2_63, Invertible, Mersenne61, Ring, SemiRing, Xorshift};
use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

pub trait RollingHasher {
    type T;
    type Hash: Copy + Eq;
    fn init(len: usize, rng: &mut Xorshift);
    fn init_with_time(len: usize) {
        let mut rng = Xorshift::time();
        Self::init(len, &mut rng);
    }
    fn ensure(len: usize);
    fn hash_sequence<I>(iter: I) -> HashedSequence<Self>
    where
        I: IntoIterator<Item = Self::T>;
    fn hash_range(seq: &HashedSequence<Self>, l: usize, r: usize) -> HashedRange<Self>;
    fn concat_hash(x: &HashedRange<Self>, y: &HashedRange<Self>) -> HashedRange<Self>;
}

#[derive(Debug)]
pub struct HashedSequence<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    hashed: Vec<Hasher::Hash>,
    _marker: PhantomData<fn() -> Hasher>,
}

impl<Hasher> HashedSequence<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    pub fn new(hashed: Vec<Hasher::Hash>) -> Self {
        Self {
            hashed,
            _marker: PhantomData,
        }
    }
    pub fn range<R>(&self, range: R) -> HashedRange<Hasher>
    where
        R: RangeBounds<usize>,
    {
        let l = match range.start_bound() {
            Bound::Included(l) => *l,
            Bound::Excluded(l) => l + 1,
            Bound::Unbounded => 0,
        };
        let r = match range.end_bound() {
            Bound::Included(r) => r + 1,
            Bound::Excluded(r) => *r,
            Bound::Unbounded => self.hashed.len() - 1,
        };
        Hasher::hash_range(self, l, r)
    }
}

#[derive(Debug)]
pub struct HashedRange<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    len: usize,
    hash: Hasher::Hash,
    _marker: PhantomData<fn() -> Hasher>,
}

impl<Hasher> HashedRange<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    pub fn new(len: usize, hash: Hasher::Hash) -> Self {
        Self {
            len,
            hash,
            _marker: PhantomData,
        }
    }
    pub fn concat(&self, other: &Self) -> Self {
        Hasher::concat_hash(self, other)
    }
}

impl<Hasher> Clone for HashedRange<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            len: self.len,
            hash: self.hash,
            _marker: self._marker,
        }
    }
}

impl<Hasher> Copy for HashedRange<Hasher> where Hasher: RollingHasher + ?Sized {}

impl<Hasher> PartialEq for HashedRange<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.hash == other.hash
    }
}

impl<Hasher> Eq for HashedRange<Hasher> where Hasher: RollingHasher + ?Sized {}

#[derive(Debug)]
struct RollingHashPrecalc<R>
where
    R: SemiRing,
{
    base: R::T,
    pow: Vec<R::T>,
}

impl<R> Default for RollingHashPrecalc<R>
where
    R: SemiRing,
    R::T: Default,
{
    fn default() -> Self {
        Self {
            base: Default::default(),
            pow: Default::default(),
        }
    }
}

impl<R> RollingHashPrecalc<R>
where
    R: SemiRing,
    R::Additive: Invertible,
{
    fn new(base: R::T) -> Self {
        Self {
            base,
            pow: vec![R::one()],
        }
    }
    fn ensure_pow(&mut self, len: usize) {
        if self.pow.len() <= len {
            self.pow.reserve(len - self.pow.len() + 1);
            if self.pow.is_empty() {
                self.pow.push(R::one());
            }
            for _ in 0..=len - self.pow.len() {
                self.pow.push(R::mul(self.pow.last().unwrap(), &self.base));
            }
        }
    }
    fn mul1_add(&self, x: &R::T, y: &R::T) -> R::T {
        R::add(&R::mul(x, &self.base), y)
    }
    fn muln_add(&mut self, x: &R::T, y: &R::T, n: usize) -> R::T {
        R::add(&R::mul(x, &self.pow[n]), y)
    }
    fn muln_sub(&mut self, l: &R::T, r: &R::T, n: usize) -> R::T {
        R::sub(r, &R::mul(l, &self.pow[n]))
    }
}

macro_rules! impl_rolling_hasher {
    (@inner $T:ident, $R:ty, [$($i:tt)*] [$($s:tt)*] [$a:tt $($tt:tt)*] [$k:tt $($j:tt)*]) => {
        impl_rolling_hasher!(@inner $T, $R, [$($i)* $k] [$($s)* ()] [$($tt)*] [$($j)*]);
    };
    (@inner $T:ident, $R:ty, [$($i:tt)+] [$($s:tt)+] [] [$len:tt $($j:tt)*]) => {
        pub enum $T {}

        impl $T {
            fn __rolling_hash_local_key() -> &'static ::std::thread::LocalKey<::std::cell::Cell<[RollingHashPrecalc<$R>; $len]>> {
                ::std::thread_local!(
                    static __LOCAL_KEY: ::std::cell::Cell<[RollingHashPrecalc<$R>; $len]> = ::std::cell::Cell::new(Default::default())
                );
                &__LOCAL_KEY
            }
        }

        impl RollingHasher for $T {
            type T = <$R as SemiRing>::T;

            type Hash = [<$R as SemiRing>::T; $len];

            fn init(len: usize, rng: &mut Xorshift) {
                Self::__rolling_hash_local_key().with(|cell| {
                    if unsafe{ (&*cell.as_ptr()).iter().all(|p| p.base == 0) } {
                        cell.set([$({ $s; RollingHashPrecalc::new(rng.rand(<$R>::MOD)) },)+]);
                    }
                });
                Self::ensure(len);
            }

            fn ensure(len: usize) {
                Self::__rolling_hash_local_key().with(|cell| {
                    unsafe {
                        let arr = &mut *cell.as_ptr();
                        $(arr[$i].ensure_pow(len);)+
                    }
                })
            }

            fn hash_sequence<I>(iter: I) -> HashedSequence<Self>
            where
                I: IntoIterator<Item = Self::T>,
            {
                let iter = iter.into_iter();
                let (lb, _) = iter.size_hint();
                let mut hashed = Vec::with_capacity(lb + 1);
                hashed.push([$({ $s; <$R>::zero() },)+]);
                unsafe {
                    Self::__rolling_hash_local_key().with(|cell| {
                        let arr = &*cell.as_ptr();
                        for item in iter {
                            let last = hashed.last().unwrap();
                            let h = [$(arr[$i].mul1_add(&last[$i], &item),)+];
                            hashed.push(h);
                        }
                    })
                };
                HashedSequence::new(hashed)
            }

            fn hash_range(seq: &HashedSequence<Self>, l: usize, r: usize) -> HashedRange<Self> {
                let len = r - l;
                let hash = unsafe {
                    Self::__rolling_hash_local_key().with(|cell| {
                        let arr = &mut *cell.as_ptr();
                        [$(arr[$i].muln_sub(&seq.hashed[l][$i], &seq.hashed[r][$i], len),)+]
                    })
                };
                HashedRange::new(len, hash)
            }

            fn concat_hash(x: &HashedRange<Self>, y: &HashedRange<Self>) -> HashedRange<Self> {
                let len = y.len;
                let hash = unsafe {
                    Self::__rolling_hash_local_key().with(|cell| {
                        let arr = &mut *cell.as_ptr();
                        [$(arr[$i].muln_add(&x.hash[$i], &y.hash[$i], len),)+]
                    })
                };
                HashedRange::new(x.len + y.len, hash)
            }
        }
    };
    ($T:ident, $R:ty, [$($tt:tt)+]) => {
        impl_rolling_hasher!(@inner $T, $R, [] [] [$($tt)+] [0 1 2 3 4 5 6 7 8 9]);
    };
}

impl_rolling_hasher!(Mersenne61x1, Mersenne61, [_]);
impl_rolling_hasher!(Mersenne61x2, Mersenne61, [_ _]);
impl_rolling_hasher!(Mersenne61x3, Mersenne61, [_ _ _]);
impl_rolling_hasher!(Gf2_63x1, Gf2_63, [_]);
impl_rolling_hasher!(Gf2_63x2, Gf2_63, [_ _]);
impl_rolling_hasher!(Gf2_63x3, Gf2_63, [_ _ _]);
