use super::Xorshift;
use std::{
    marker::PhantomData,
    mem::swap,
    ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

/// Trait for spec of generating random value.
pub trait RandomSpec<T>: Sized {
    /// Return a random value.
    fn rand(&self, rng: &mut Xorshift) -> T;
    /// Return an iterator that generates random values.
    fn rand_iter(self, rng: &mut Xorshift) -> RandIter<'_, T, Self> {
        RandIter {
            spec: self,
            rng,
            _marker: PhantomData,
        }
    }
}

impl Xorshift {
    pub fn random<T, R>(&mut self, spec: R) -> T
    where
        R: RandomSpec<T>,
    {
        spec.rand(self)
    }
    pub fn random_iter<T, R>(&mut self, spec: R) -> RandIter<'_, T, R>
    where
        R: RandomSpec<T>,
    {
        spec.rand_iter(self)
    }
}

#[derive(Debug)]
pub struct RandIter<'r, T, R>
where
    R: RandomSpec<T>,
{
    spec: R,
    rng: &'r mut Xorshift,
    _marker: PhantomData<fn() -> T>,
}

impl<T, R> Iterator for RandIter<'_, T, R>
where
    R: RandomSpec<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.spec.rand(self.rng))
    }
}

macro_rules! impl_random_spec_range_full {
    ($($t:ty)*) => {
        $(impl RandomSpec<$t> for RangeFull {
            fn rand(&self, rng: &mut Xorshift) -> $t {
                rng.rand64() as _
            }
        })*
    };
}
impl_random_spec_range_full!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize);

impl RandomSpec<u128> for RangeFull {
    fn rand(&self, rng: &mut Xorshift) -> u128 {
        ((rng.rand64() as u128) << 64) | rng.rand64() as u128
    }
}
impl RandomSpec<i128> for RangeFull {
    fn rand(&self, rng: &mut Xorshift) -> i128 {
        rng.random::<u128, _>(..) as i128
    }
}

macro_rules! impl_random_spec_ranges {
    ($($u:ident $i:ident)*) => {
        $(
            impl RandomSpec<$u> for Range<$u> {
                fn rand(&self, rng: &mut Xorshift) -> $u {
                    assert!(self.start < self.end);
                    let len = self.end - self.start;
                    (self.start + rng.random::<$u, _>(..) % len)
                }
            }
            impl RandomSpec<$i> for Range<$i> {
                fn rand(&self, rng: &mut Xorshift) -> $i {
                    assert!(self.start < self.end);
                    let len = self.end.abs_diff(self.start);
                    self.start.wrapping_add_unsigned(rng.random::<$u, _>(..) % len)
                }
            }
            impl RandomSpec<$u> for RangeFrom<$u> {
                fn rand(&self, rng: &mut Xorshift) -> $u {
                    let len = ($u::MAX - self.start).wrapping_add(1);
                    let x = rng.random::<$u, _>(..);
                    self.start + if len != 0 { x % len } else { x }
                }
            }
            impl RandomSpec<$i> for RangeFrom<$i> {
                fn rand(&self, rng: &mut Xorshift) -> $i {
                    let len = ($i::MAX.abs_diff(self.start)).wrapping_add(1);
                    let x = rng.random::<$u, _>(..);
                    self.start.wrapping_add_unsigned(if len != 0 { x % len } else { x })
                }
            }
            impl RandomSpec<$u> for RangeInclusive<$u> {
                fn rand(&self, rng: &mut Xorshift) -> $u {
                    assert!(self.start() <= self.end());
                    let len = (self.end() - self.start()).wrapping_add(1);
                    let x = rng.random::<$u, _>(..);
                    self.start() + if len != 0 { x % len } else { x }
                }
            }
            impl RandomSpec<$i> for RangeInclusive<$i> {
                fn rand(&self, rng: &mut Xorshift) -> $i {
                    assert!(self.start() <= self.end());
                    let len = (self.end().abs_diff(*self.start())).wrapping_add(1);
                    let x = rng.random::<$u, _>(..);
                    self.start().wrapping_add_unsigned(if len != 0 { x % len } else { x })
                }
            }
            impl RandomSpec<$u> for RangeTo<$u> {
                fn rand(&self, rng: &mut Xorshift) -> $u {
                    let len = self.end;
                    rng.random::<$u, _>(..) % len
                }
            }
            impl RandomSpec<$i> for RangeTo<$i> {
                fn rand(&self, rng: &mut Xorshift) -> $i {
                    let len = self.end.abs_diff($i::MIN);
                    $i::MIN.wrapping_add_unsigned(rng.random::<$u, _>(..) % len)
                }
            }
            impl RandomSpec<$u> for RangeToInclusive<$u> {
                fn rand(&self, rng: &mut Xorshift) -> $u {
                    let len = (self.end).wrapping_add(1);
                    let x = rng.random::<$u, _>(..);
                    if len != 0 { x % len } else { x }
                }
            }
            impl RandomSpec<$i> for RangeToInclusive<$i> {
                fn rand(&self, rng: &mut Xorshift) -> $i {
                    let len = (self.end.abs_diff($i::MIN)).wrapping_add(1);
                    let x = rng.random::<$u, _>(..);
                    $i::MIN.wrapping_add_unsigned(if len != 0 { x % len } else { x })
                }
            }
        )*
    };
}
impl_random_spec_ranges!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

macro_rules! impl_random_spec_tuple {
    ($($T:ident)*, $($R:ident)*, $($v:ident)*) => {
        impl<$($T),*, $($R),*> RandomSpec<($($T,)*)> for ($($R,)*)
        where
            $($R: RandomSpec<$T>),*
        {
            fn rand(&self, rng: &mut Xorshift) -> ($($T,)*) {
                let ($($v,)*) = self;
                ($(($v).rand(rng),)*)
            }
        }
    };
}
impl_random_spec_tuple!(A, RA, a);
impl_random_spec_tuple!(A B, RA RB, a b);
impl_random_spec_tuple!(A B C, RA RB RC, a b c);
impl_random_spec_tuple!(A B C D, RA RB RC RD, a b c d);
impl_random_spec_tuple!(A B C D E, RA RB RC RD RE, a b c d e);
impl_random_spec_tuple!(A B C D E F, RA RB RC RD RE RF, a b c d e f);
impl_random_spec_tuple!(A B C D E F G, RA RB RC RD RE RF RG, a b c d e f g);
impl_random_spec_tuple!(A B C D E F G H, RA RB RC RD RE RF RG RH, a b c d e f g h);
impl_random_spec_tuple!(A B C D E F G H I, RA RB RC RD RE RF RG RH RI, a b c d e f g h i);
impl_random_spec_tuple!(A B C D E F G H I J, RA RB RC RD RE RF RG RH RI RJ, a b c d e f g h i j);

macro_rules! impl_random_spec_primitive {
    ($($t:ty)*) => {
        $(impl RandomSpec<$t> for $t {
            fn rand(&self, _rng: &mut Xorshift) -> $t {
                *self
            }
        })*
    };
}
impl_random_spec_primitive!(() u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize bool char);

impl<T, R> RandomSpec<T> for &R
where
    R: RandomSpec<T>,
{
    fn rand(&self, rng: &mut Xorshift) -> T {
        <R as RandomSpec<T>>::rand(self, rng)
    }
}
impl<T, R> RandomSpec<T> for &mut R
where
    R: RandomSpec<T>,
{
    fn rand(&self, rng: &mut Xorshift) -> T {
        <R as RandomSpec<T>>::rand(self, rng)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Left-close Right-open No Empty Segment
pub struct NotEmptySegment<T>(pub T);
impl<T> RandomSpec<(usize, usize)> for NotEmptySegment<T>
where
    T: RandomSpec<usize>,
{
    fn rand(&self, rng: &mut Xorshift) -> (usize, usize) {
        let n = rng.random(&self.0) as u64;
        let k = randint_uniform(rng, n);
        let l = randint_uniform(rng, n - k) as usize;
        (l, l + k as usize + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandRange<Q, T> {
    data: Q,
    _marker: PhantomData<fn() -> T>,
}
impl<Q, T> RandRange<Q, T> {
    pub fn new(data: Q) -> Self {
        Self {
            data,
            _marker: PhantomData,
        }
    }
}
impl<Q, T> RandomSpec<(Bound<T>, Bound<T>)> for RandRange<Q, T>
where
    Q: RandomSpec<T>,
    T: Ord,
{
    fn rand(&self, rng: &mut Xorshift) -> (Bound<T>, Bound<T>) {
        let mut l = rng.random(&self.data);
        let mut r = rng.random(&self.data);
        if l > r {
            swap(&mut l, &mut r);
        }
        (
            match rng.rand(3) {
                0 => Bound::Excluded(l),
                1 => Bound::Included(l),
                _ => Bound::Unbounded,
            },
            match rng.rand(3) {
                0 => Bound::Excluded(r),
                1 => Bound::Included(r),
                _ => Bound::Unbounded,
            },
        )
    }
}

#[inline]
fn randint_uniform(rng: &mut Xorshift, k: u64) -> u64 {
    let mut v = rng.rand64();
    if k > 0 {
        v %= k;
    }
    v
}

#[macro_export]
/// Return a random value using [`RandomSpec`].
macro_rules! rand_value {
    ($rng:expr, ($($e:expr),*)) => {
        ($($crate::rand_value!($rng, $e)),*)
    };
    ($rng:expr, ($($t:tt),*)) => {
        ($($crate::rand_value!($rng, $t)),*)
    };
    ($rng:expr, [$t:tt; $len:expr]) => {
        ::std::iter::repeat_with(|| $crate::rand_value!($rng, $t)).take($len).collect::<Vec<_>>()
    };
    ($rng:expr, [$s:expr; $len:expr]) => {
        ($rng).random_iter($s).take($len).collect::<Vec<_>>()
    };
    ($rng:expr, [$($t:tt)*]) => {
        ::std::iter::repeat_with(|| $crate::rand_value!($rng, $($t)*))
    };
    ($rng:expr, $s:expr) => {
        ($rng).random($s)
    };
}
#[macro_export]
/// Declare random values using [`RandomSpec`].
macro_rules! rand {
    (@assert $p:pat) => {};
    (@assert $($p:tt)*) => { ::std::compile_error!(::std::concat!("expected pattern, found `", ::std::stringify!($($p)*), "`")); };
    (@pat $rng:expr, [] [])                                          => {};
    (@pat $rng:expr, [] [] , $($t:tt)*)                              => { $crate::rand!(@pat $rng, [] [] $($t)*) };
    (@pat $rng:expr, [$($p:tt)*] [] $x:ident $($t:tt)*)              => { $crate::rand!(@pat $rng, [$($p)* $x] [] $($t)*) };
    (@pat $rng:expr, [$($p:tt)*] [] :: $($t:tt)*)                    => { $crate::rand!(@pat $rng, [$($p)* ::] [] $($t)*) };
    (@pat $rng:expr, [$($p:tt)*] [] & $($t:tt)*)                     => { $crate::rand!(@pat $rng, [$($p)* &] [] $($t)*) };
    (@pat $rng:expr, [$($p:tt)*] [] ($($x:tt)*) $($t:tt)*)           => { $crate::rand!(@pat $rng, [$($p)* ($($x)*)] [] $($t)*) };
    (@pat $rng:expr, [$($p:tt)*] [] [$($x:tt)*] $($t:tt)*)           => { $crate::rand!(@pat $rng, [$($p)* [$($x)*]] [] $($t)*) };
    (@pat $rng:expr, [$($p:tt)*] [] {$($x:tt)*} $($t:tt)*)           => { $crate::rand!(@pat $rng, [$($p)* {$($x)*}] [] $($t)*) };
    (@pat $rng:expr, [$($p:tt)*] [] : $($t:tt)*)                     => { $crate::rand!(@ty  $rng, [$($p)*] [] $($t)*) };
    (@ty  $rng:expr, [$($p:tt)*] [$($tt:tt)*] ($($x:tt)*) $($t:tt)*) => { $crate::rand!(@let $rng, [$($p)*] [$($tt)* ($($x)*)] $($t)*) };
    (@ty  $rng:expr, [$($p:tt)*] [$($tt:tt)*] [$($x:tt)*] $($t:tt)*) => { $crate::rand!(@let $rng, [$($p)*] [$($tt)* [$($x)*]] $($t)*) };
    (@ty  $rng:expr, [$($p:tt)*] [$($tt:tt)*] $e:expr)               => { $crate::rand!(@let $rng, [$($p)*] [$($tt)* $e]) };
    (@ty  $rng:expr, [$($p:tt)*] [$($tt:tt)*] $e:expr, $($t:tt)*)    => { $crate::rand!(@let $rng, [$($p)*] [$($tt)* $e], $($t)*) };
    (@ty  $rng:expr, [$($p:tt)*] [$($tt:tt)*] $e:tt)                 => { $crate::rand!(@let $rng, [$($p)*] [$($tt)* $e]) };
    (@ty  $rng:expr, [$($p:tt)*] [$($tt:tt)*] $e:tt, $($t:tt)*)      => { $crate::rand!(@let $rng, [$($p)*] [$($tt)* $e], $($t)*) };
    (@let $rng:expr, [$($p:tt)*] [$($tt:tt)*] $($t:tt)*) => {
        $crate::rand!{@assert $($p)*}
        let $($p)* = $crate::rand_value!($rng, $($tt)*);
        $crate::rand!(@pat $rng, [] [] $($t)*)
    };
    ($rng:expr) => {};
    ($rng:expr, $($t:tt)*) => { $crate::rand!(@pat $rng, [] [] $($t)*) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_range() {
        let mut rng = Xorshift::default();
        assert_eq!(rng.random(1i32..2), 1);
        assert_eq!(rng.random(1u32..2), 1);
        assert_eq!(rng.random(1i32..=1), 1);
        assert_eq!(rng.random(1u32..=1), 1);
        assert_eq!(rng.random(i32::MAX..), i32::MAX);
        assert_eq!(rng.random(u32::MAX..), u32::MAX);
        assert_eq!(rng.random(..=i32::MIN), i32::MIN);
        assert_eq!(rng.random(..=u32::MIN), u32::MIN);
    }

    #[test]
    fn test_random_segment() {
        let mut rng = Xorshift::default();
        for _ in 0..100_000 {
            let n = (1..=1_000_000).rand(&mut rng);
            let (l, r) = NotEmptySegment(n).rand(&mut rng);
            assert!(l < r);
            assert!(r <= n);
        }

        const N_SMALL: usize = 100;
        let mut set = std::collections::HashSet::new();
        for _ in 0..100_000 {
            let (l, r) = NotEmptySegment(N_SMALL).rand(&mut rng);
            assert!(l < r);
            assert!(r <= N_SMALL);
            set.insert((l, r));
        }
        assert!(set.len() == N_SMALL * (N_SMALL + 1) / 2);
    }

    #[test]
    fn test_rand_macro() {
        let mut rng = Xorshift::default();
        rand!(
            rng,
            _x: ..10,
            _lr: NotEmptySegment(10),
            _a: [..10; 10],
            _t: (..10,),
            _r: (&(..10),&mut (..10)),
            _p: [(1..=10,2..=10); 2]
        );
    }
}
