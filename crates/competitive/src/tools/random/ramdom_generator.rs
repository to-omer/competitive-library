#[codesnip::skip]
use crate::num::Bounded;

use super::*;
use std::{
    marker::PhantomData,
    ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive},
};

pub trait RandomGenerator<T>: Sized {
    fn rand(&self, rng: &mut Xorshift) -> T;
    fn rand_iter(self, rng: &mut Xorshift) -> RandIter<'_, T, Self> {
        RandIter {
            gen: self,
            rng,
            _marker: PhantomData,
        }
    }
}

impl Xorshift {
    pub fn gen<T, G: RandomGenerator<T>>(&mut self, generator: G) -> T {
        generator.rand(self)
    }
    pub fn gen_iter<T, G: RandomGenerator<T>>(&mut self, generator: G) -> RandIter<'_, T, G> {
        generator.rand_iter(self)
    }
}

#[derive(Debug)]
pub struct RandIter<'r, T, G: RandomGenerator<T>> {
    gen: G,
    rng: &'r mut Xorshift,
    _marker: PhantomData<fn() -> T>,
}
impl<T, G: RandomGenerator<T>> Iterator for RandIter<'_, T, G> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.gen.rand(&mut self.rng))
    }
}

impl<T: NotEmptyStep64> RandomGenerator<T> for Range<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count = <T as NotEmptyStep64>::steps_between(&self.start, &self.end);
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&self.start, count)
    }
}
impl<T: NotEmptyStep64 + Bounded + Copy> RandomGenerator<T> for RangeFrom<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count =
            <T as NotEmptyStep64>::steps_between(&self.start, &<T as Bounded>::MAX).wrapping_add(1);
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&self.start, count)
    }
}
impl<T: NotEmptyStep64> RandomGenerator<T> for RangeInclusive<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count =
            <T as NotEmptyStep64>::steps_between(&self.start(), &self.end()).wrapping_add(1);
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&self.start(), count)
    }
}
impl<T: NotEmptyStep64 + Bounded + Copy> RandomGenerator<T> for RangeTo<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count = <T as NotEmptyStep64>::steps_between(&<T as Bounded>::MIN, &self.end);
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&<T as Bounded>::MIN, count)
    }
}
impl<T: NotEmptyStep64 + Bounded + Copy> RandomGenerator<T> for RangeToInclusive<T> {
    fn rand(&self, rng: &mut Xorshift) -> T {
        let count =
            <T as NotEmptyStep64>::steps_between(&<T as Bounded>::MIN, &self.end).wrapping_add(1);
        let count = randint_uniform(rng, count);
        <T as NotEmptyStep64>::forward_unchecked(&<T as Bounded>::MIN, count)
    }
}
macro_rules! random_generator_tuple_impls {
    ($($T:ident)*, $($G:ident)*, $($v:ident)*) => {
        impl<$($T),*, $($G),*> RandomGenerator<($($T,)*)> for ($($G,)*)
        where
            $($G: RandomGenerator<$T>),*
        {
            fn rand(&self, rng: &mut Xorshift) -> ($($T,)*) {
                let ($($v,)*) = self;
                ($(($v).rand(rng),)*)
            }
        }
    };
}
random_generator_tuple_impls!(A, GA, a);
random_generator_tuple_impls!(A B, GA GB, a b);
random_generator_tuple_impls!(A B C, GA GB GC, a b c);
random_generator_tuple_impls!(A B C D, GA GB GC GD, a b c d);
random_generator_tuple_impls!(A B C D E, GA GB GC GD GE, a b c d e);
random_generator_tuple_impls!(A B C D E F, GA GB GC GD GE GF, a b c d e f);
random_generator_tuple_impls!(A B C D E F G, GA GB GC GD GE GF GG, a b c d e f g);
random_generator_tuple_impls!(A B C D E F G H, GA GB GC GD GE GF GG GH, a b c d e f g h);
random_generator_tuple_impls!(A B C D E F G H I, GA GB GC GD GE GF GG GH GI, a b c d e f g h i);
random_generator_tuple_impls!(A B C D E F G H I J, GA GB GC GD GE GF GG GH GI GJ, a b c d e f g h i j);

pub trait NotEmptyStep64: Clone + PartialOrd {
    fn steps_between(start: &Self, end: &Self) -> u64;
    fn forward_unchecked(start: &Self, count: u64) -> Self;
}

macro_rules! step64_impls {
    ([$($u:ty),*],[$($i:ty),*]) => {
        $(impl NotEmptyStep64 for $u {
            fn steps_between(start: &Self, end: &Self) -> u64 {
                if *start < *end {
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
                if *start < *end {
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
        if start < end {
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
pub struct NotEmptySegment(pub usize);
impl RandomGenerator<(usize, usize)> for NotEmptySegment {
    fn rand(&self, rng: &mut Xorshift) -> (usize, usize) {
        let n = randint_uniform(rng, self.0 as u64);
        let l = randint_uniform(rng, self.0 as u64 - n) as usize;
        (l, l + n as usize + 1)
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
    ($rng:expr, [$g:expr; $len:expr]) => {
        ($rng).gen_iter($g).take($len).collect::<Vec<_>>()
    };
    ($rng:expr, [$($t:tt)*]) => {
        ::std::iter::repeat_with(|| $crate::rand_value!($rng, $($t)*))
    };
    ($rng:expr, {$g:expr}) => {
        ($rng).gen($g)
    };
    ($rng:expr, $g:expr) => {
        ($rng).gen($g)
    };
}
#[macro_export]
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
            _p: [(1..=10,2..=10); 2]
        );
    }
}
