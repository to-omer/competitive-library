use super::{Gf2_63, Invertible, Mersenne61, Ring, SemiRing, Xorshift};
use std::{
    cmp::Ordering,
    marker::PhantomData,
    ops::{Bound, RangeBounds, RangeInclusive},
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
    fn hash_substr(hashed: &[Self::Hash]) -> Hashed<Self>;
    fn concat_hash(x: &Hashed<Self>, y: &Hashed<Self>) -> Hashed<Self>;
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
    fn new(hashed: Vec<Hasher::Hash>) -> Self {
        Self {
            hashed,
            _marker: PhantomData,
        }
    }
    pub fn length(&self) -> usize {
        self.hashed.len() - 1
    }
    pub fn range<R>(&self, range: R) -> HashedRange<'_, Hasher>
    where
        R: RangeBounds<usize>,
    {
        HashedRange::new(&self.hashed[to_range(range, self.length())])
    }
    pub fn hash_range<R>(&self, range: R) -> Hashed<Hasher>
    where
        R: RangeBounds<usize>,
    {
        self.range(range).hash()
    }
}

#[derive(Debug)]
pub struct HashedRange<'a, Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    hashed: &'a [Hasher::Hash],
    _marker: PhantomData<fn() -> Hasher>,
}

impl<Hasher> Clone for HashedRange<'_, Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            hashed: self.hashed,
            _marker: self._marker,
        }
    }
}

impl<Hasher> Copy for HashedRange<'_, Hasher> where Hasher: RollingHasher + ?Sized {}

impl<Hasher> PartialEq for HashedRange<'_, Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl<Hasher> Eq for HashedRange<'_, Hasher> where Hasher: RollingHasher + ?Sized {}

impl<Hasher> PartialOrd for HashedRange<'_, Hasher>
where
    Hasher: RollingHasher + ?Sized,
    Hasher::Hash: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let n = self.longest_common_prefix(other);
        match (self.length() > n, other.length() > n) {
            (true, true) => {
                let x = self.hash_range(n..=n);
                let y = other.hash_range(n..=n);
                x.hash.partial_cmp(&y.hash)
            }
            (x, y) => Some(x.cmp(&y)),
        }
    }
}

impl<Hasher> Ord for HashedRange<'_, Hasher>
where
    Hasher: RollingHasher + ?Sized,
    Hasher::Hash: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        let n = self.longest_common_prefix(other);
        match (self.length() > n, other.length() > n) {
            (true, true) => {
                let x = self.hash_range(n..=n);
                let y = other.hash_range(n..=n);
                x.hash.cmp(&y.hash)
            }
            (x, y) => x.cmp(&y),
        }
    }
}

impl<'a, Hasher> HashedRange<'a, Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    fn new(hashed: &'a [Hasher::Hash]) -> Self {
        Self {
            hashed,
            _marker: PhantomData,
        }
    }
    pub fn length(&self) -> usize {
        self.hashed.len() - 1
    }
    pub fn range<R>(&self, range: R) -> HashedRange<'a, Hasher>
    where
        R: RangeBounds<usize>,
    {
        HashedRange::new(&self.hashed[to_range(range, self.length())])
    }
    pub fn hash_range<R>(&self, range: R) -> Hashed<Hasher>
    where
        R: RangeBounds<usize>,
    {
        self.range(range).hash()
    }
    pub fn hash(&self) -> Hashed<Hasher> {
        Hasher::hash_substr(self.hashed)
    }
    pub fn longest_common_prefix(&self, other: &Self) -> usize {
        let n = self.length().min(other.length());
        let mut ok = 0usize;
        let mut err = n + 1;
        while ok + 1 < err {
            let mid = (ok + err) / 2;
            if self.range(..mid).hash() == other.range(..mid).hash() {
                ok = mid;
            } else {
                err = mid;
            }
        }
        ok
    }
}

fn to_range<R>(range: R, ub: usize) -> RangeInclusive<usize>
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
        Bound::Unbounded => ub,
    };
    l..=r
}

#[derive(Debug)]
pub struct Hashed<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    len: usize,
    hash: Hasher::Hash,
    _marker: PhantomData<fn() -> Hasher>,
}

impl<Hasher> std::hash::Hash for Hashed<Hasher>
where
    Hasher: RollingHasher + ?Sized,
    Hasher::Hash: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.hash.hash(state);
        self._marker.hash(state);
    }
}

impl<Hasher> Hashed<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    fn new(len: usize, hash: Hasher::Hash) -> Self {
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

impl<Hasher> Clone for Hashed<Hasher>
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

impl<Hasher> Copy for Hashed<Hasher> where Hasher: RollingHasher + ?Sized {}

impl<Hasher> PartialEq for Hashed<Hasher>
where
    Hasher: RollingHasher + ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.hash == other.hash
    }
}

impl<Hasher> Eq for Hashed<Hasher> where Hasher: RollingHasher + ?Sized {}

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
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

            fn hash_substr(hashed: &[Self::Hash]) -> Hashed<Self> {
                let len = hashed.len() - 1;
                let h = unsafe {
                    Self::__rolling_hash_local_key().with(|cell| {
                        let arr = &mut *cell.as_ptr();
                        [$(arr[$i].muln_sub(&hashed[0][$i], &hashed[len][$i], len),)+]
                    })
                };
                Hashed::new(len, h)
            }

            fn concat_hash(x: &Hashed<Self>, y: &Hashed<Self>) -> Hashed<Self> {
                let len = y.len;
                let hash = unsafe {
                    Self::__rolling_hash_local_key().with(|cell| {
                        let arr = &mut *cell.as_ptr();
                        [$(arr[$i].muln_add(&x.hash[$i], &y.hash[$i], len),)+]
                    })
                };
                Hashed::new(x.len + y.len, hash)
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
