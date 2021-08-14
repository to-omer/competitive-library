#[codesnip::skip]
use crate::num::Bounded;

use super::*;
use std::{
    marker::PhantomData,
    ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive},
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
    pub fn gen<T, R: RandomSpec<T>>(&mut self, spec: R) -> T {
        spec.rand(self)
    }
    pub fn gen_iter<T, R: RandomSpec<T>>(&mut self, spec: R) -> RandIter<'_, T, R> {
        spec.rand_iter(self)
    }
}

#[derive(Debug)]
pub struct RandIter<'r, T, R: RandomSpec<T>> {
    spec: R,
    rng: &'r mut Xorshift,
    _marker: PhantomData<fn() -> T>,
}
impl<T, R: RandomSpec<T>> Iterator for RandIter<'_, T, R> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.spec.rand(&mut self.rng))
    }
}

impl<T: NotEmptyStep64> RandomSpec<T> for Range<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count = <T as NotEmptyStep64>::steps_between(&self.start, &self.end);
        assert_ne!(count, 0, "empty range in `RandomSpec<T> for Range<T>`");
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&self.start, count)
    }
}
impl<T: NotEmptyStep64 + Bounded> RandomSpec<T> for RangeFrom<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count = <T as NotEmptyStep64>::steps_between(&self.start, &<T as Bounded>::maximum())
            .wrapping_add(1);
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&self.start, count)
    }
}
impl<T: NotEmptyStep64> RandomSpec<T> for RangeInclusive<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count = <T as NotEmptyStep64>::steps_between(self.start(), self.end()).wrapping_add(1);
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(self.start(), count)
    }
}
impl<T: NotEmptyStep64 + Bounded> RandomSpec<T> for RangeTo<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count = <T as NotEmptyStep64>::steps_between(&<T as Bounded>::minimum(), &self.end);
        assert_ne!(count, 0, "empty range in `RandomSpec<T> for RangeTo<T>`");
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&<T as Bounded>::minimum(), count)
    }
}
impl<T: NotEmptyStep64 + Bounded> RandomSpec<T> for RangeToInclusive<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count = <T as NotEmptyStep64>::steps_between(&<T as Bounded>::minimum(), &self.end)
            .wrapping_add(1);
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&<T as Bounded>::minimum(), count)
    }
}
macro_rules! random_spec_tuple_impls {
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
random_spec_tuple_impls!(A, RA, a);
random_spec_tuple_impls!(A B, RA RB, a b);
random_spec_tuple_impls!(A B C, RA RB RC, a b c);
random_spec_tuple_impls!(A B C D, RA RB RC RD, a b c d);
random_spec_tuple_impls!(A B C D E, RA RB RC RD RE, a b c d e);
random_spec_tuple_impls!(A B C D E F, RA RB RC RD RE RF, a b c d e f);
random_spec_tuple_impls!(A B C D E F G, RA RB RC RD RE RF RG, a b c d e f g);
random_spec_tuple_impls!(A B C D E F G H, RA RB RC RD RE RF RG RH, a b c d e f g h);
random_spec_tuple_impls!(A B C D E F G H I, RA RB RC RD RE RF RG RH RI, a b c d e f g h i);
random_spec_tuple_impls!(A B C D E F G H I J, RA RB RC RD RE RF RG RH RI RJ, a b c d e f g h i j);

macro_rules! random_spec_primitive_impls {
    ($($t:ty)*) => {
        $(impl RandomSpec<$t> for $t {
            fn rand(&self, _rng: &mut Xorshift) -> $t {
                *self
            }
        })*
    };
}
random_spec_primitive_impls!(() u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize bool char);

impl<T, R: RandomSpec<T>> RandomSpec<T> for &R {
    fn rand(&self, rng: &mut Xorshift) -> T {
        <R as RandomSpec<T>>::rand(self, rng)
    }
}
impl<T, R: RandomSpec<T>> RandomSpec<T> for &mut R {
    fn rand(&self, rng: &mut Xorshift) -> T {
        <R as RandomSpec<T>>::rand(self, rng)
    }
}

pub trait NotEmptyStep64: Clone + PartialOrd {
    fn steps_between(start: &Self, end: &Self) -> u64;
    fn forward_unchecked(start: &Self, count: u64) -> Self;
}

macro_rules! step64_impls {
    ([$($u:ty),*],[$($i:ty),*]) => {
        $(impl NotEmptyStep64 for $u {
            fn steps_between(start: &Self, end: &Self) -> u64 {
                if *start <= *end {
                    (*end - *start) as u64
                } else {
                    panic!("empty range in `NotEmptyStep64`");
                }
            }
            fn forward_unchecked(start: &Self, count: u64) -> Self {
                start + count as Self
            }
        })*
        $(impl NotEmptyStep64 for $i {
            fn steps_between(start: &Self, end: &Self) -> u64 {
                if *start <= *end {
                    ((*end as i64).wrapping_sub(*start as i64)) as u64
                } else {
                    panic!("empty range in `NotEmptyStep64`");
                }
            }
            fn forward_unchecked(start: &Self, count: u64) -> Self {
                start + count as Self
            }
        })*
    };
}
step64_impls!([u8, u16, u32, u64, usize], [i8, i16, i32, i64, isize]);
impl NotEmptyStep64 for char {
    fn steps_between(start: &Self, end: &Self) -> u64 {
        let start = *start as u8;
        let end = *end as u8;
        if start <= end {
            (end - start) as u64
        } else {
            panic!("empty range in `NotEmptyStep64`");
        }
    }
    fn forward_unchecked(start: &Self, count: u64) -> Self {
        NotEmptyStep64::forward_unchecked(&(*start as u8), count) as char
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Left-close Right-open No Empty Segment
pub struct NotEmptySegment<T>(pub T);
impl<T: RandomSpec<usize>> RandomSpec<(usize, usize)> for NotEmptySegment<T> {
    fn rand(&self, rng: &mut Xorshift) -> (usize, usize) {
        let n = rng.gen(&self.0) as u64;
        let k = randint_uniform(rng, n);
        let l = randint_uniform(rng, n - k) as usize;
        (l, l + k as usize + 1)
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
        ($rng).gen_iter($s).take($len).collect::<Vec<_>>()
    };
    ($rng:expr, [$($t:tt)*]) => {
        ::std::iter::repeat_with(|| $crate::rand_value!($rng, $($t)*))
    };
    ($rng:expr, {$s:expr}) => {
        ($rng).gen($s)
    };
    ($rng:expr, $s:expr) => {
        ($rng).gen($s)
    };
}
#[macro_export]
/// Declare random values using [`RandomSpec`].
macro_rules! rand {
    ($rng:expr) => {};
    ($rng:expr,) => {};
    ($rng:expr, $var:tt: $t:tt) => {
        let $var = $crate::rand_value!($rng, $t);
    };
    ($rng:expr, mut $var:tt: $t:tt) => {
        let mut $var = $crate::rand_value!($rng, $t);
    };
    ($rng:expr, $var:tt: $t:tt, $($rest:tt)*) => {
        let $var = $crate::rand_value!($rng, $t);
        rand!($rng, $($rest)*)
    };
    ($rng:expr, mut $var:tt: $t:tt, $($rest:tt)*) => {
        let mut $var = $crate::rand_value!($rng, $t);
        rand!($rng, $($rest)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_random_segment() {
        let mut rng = Xorshift::default();
        for _ in 0..100_000 {
            let n = (0..1_000_000).rand(&mut rng) + 1;
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
            &mut rng,
            _x: (..10),
            _lr: (NotEmptySegment(10)),
            _a: [..10; 10],
            _t: (..10,),
            _r: (&(..10),&mut (..10)),
            _p: [(1..=10,2..=10); 2]
        );
    }
}
