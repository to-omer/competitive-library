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

pub struct Spec<T, F> {
    generate: F,
    _marker: PhantomData<fn() -> T>,
}

/// Wraps a random specification for composition.
pub fn spec<T, R>(source: R) -> Spec<T, impl Fn(&mut Xorshift) -> T>
where
    R: RandomSpec<T>,
{
    from_fn(move |rng| source.rand(rng))
}

/// Calls `generate` with the RNG whenever a value is requested.
pub fn from_fn<T, F>(generate: F) -> Spec<T, F>
where
    F: Fn(&mut Xorshift) -> T,
{
    Spec {
        generate,
        _marker: PhantomData,
    }
}

impl<T, F> Spec<T, F>
where
    F: Fn(&mut Xorshift) -> T,
{
    /// Samples the length once, then generates that many elements.
    pub fn vec<R>(self, length: R) -> Spec<Vec<T>, impl Fn(&mut Xorshift) -> Vec<T>>
    where
        R: RandomSpec<usize>,
    {
        from_fn(move |rng| {
            let n = rng.random(&length);
            rng.random_iter(&self).take(n).collect()
        })
    }
}

impl<T, F> RandomSpec<T> for Spec<T, F>
where
    F: Fn(&mut Xorshift) -> T,
{
    fn rand(&self, rng: &mut Xorshift) -> T {
        (self.generate)(rng)
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
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
/// Left-close Right-open Segment
pub struct WithEmptySegment<T>(pub T);
impl<T> RandomSpec<(usize, usize)> for WithEmptySegment<T>
where
    T: RandomSpec<usize>,
{
    fn rand(&self, rng: &mut Xorshift) -> (usize, usize) {
        let n = rng.random(&self.0) as u64;
        let k = randint_uniform(rng, n + 1);
        let l = randint_uniform(rng, n - k + 1) as usize;
        (l, l + k as usize)
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

pub struct WeightedSampler {
    n: usize,
    prob: Vec<f64>,
    alias: Vec<usize>,
}

impl WeightedSampler {
    pub fn new(weights: impl IntoIterator<Item = f64>) -> Self {
        let mut weights: Vec<_> = weights.into_iter().collect();
        let n = weights.len();
        assert!(n > 0, "weights must be non-empty");
        let mut prob = vec![0.0; n];
        let mut alias = vec![0; n];
        let mut small = vec![];
        let mut large = vec![];
        let sum: f64 = weights.iter().sum();
        assert!(sum > 0.0, "sum of weights must be positive");
        let mut scale = n as f64 / sum;
        if !sum.is_finite() || !scale.is_finite() {
            let max = weights.iter().fold(0.0f64, |max, &weight| {
                assert!(
                    weight.is_finite() && weight >= 0.0,
                    "weights must be finite and non-negative"
                );
                max.max(weight)
            });
            let sum: f64 = weights
                .iter_mut()
                .map(|weight| {
                    *weight /= max;
                    *weight
                })
                .sum();
            scale = n as f64 / sum;
        }
        for (i, weight) in weights.iter_mut().enumerate() {
            assert!(*weight >= 0.0, "weights must be non-negative");
            *weight *= scale;
            if *weight < 1.0 {
                small.push(i);
            } else {
                large.push(i);
            }
        }
        while let Some(&g) = large.last() {
            let Some(l) = small.pop() else {
                break;
            };
            prob[l] = weights[l];
            alias[l] = g;
            weights[g] -= 1.0 - weights[l];
            if weights[g] < 1.0 {
                large.pop();
                small.push(g);
            }
        }
        for g in small.into_iter().chain(large).rev() {
            prob[g] = 1.0;
            alias[g] = g;
        }
        Self { n, prob, alias }
    }
}

impl RandomSpec<usize> for WeightedSampler {
    fn rand(&self, rng: &mut Xorshift) -> usize {
        let i = rng.rand(self.n as u64) as usize;
        if rng.randf() < self.prob[i] {
            i
        } else {
            self.alias[i]
        }
    }
}

#[macro_export]
/// Return a random value using [`RandomSpec`].
///
/// ```
/// use competitive::tools::Xorshift;
///
/// let mut rng = Xorshift::new_with_seed(42);
/// let values: Vec<u32> = competitive::rand_value!(rng, [0..10; 4]);
/// assert_eq!(values.len(), 4);
/// assert!(values.into_iter().all(|value| value < 10));
/// ```
macro_rules! rand_value {
    (@repeat $rng:expr, [$($t:tt)*] $($len:expr)?)                                    => { ::std::iter::repeat_with(|| $crate::rand_value!(@inner $rng, [] $($t)*)) $(.take($len).collect::<::std::vec::Vec<_>>())? };
    (@array $rng:expr, [$($t:tt)*] $len:expr)                                         => {{ let values: [_; $len] = ::std::array::from_fn(|_| $crate::rand_value!(@inner $rng, [] $($t)*)); values }};
    (@group $rng:expr, [])                                                           => { () };
    (@group $rng:expr, [$($args:tt)*] , $($t:tt)*)                                   => { $crate::rand_value!(@tuple $rng, [$($args)*] $($t)*) };
    (@tuple $rng:expr, [$([$($args:tt)*])*])                                          => { ($($($args)*,)*) };
    (@$tag:ident $rng:expr, [[$($args:tt)*]])                                         => { $($args)* };
    (@$tag:ident $rng:expr, [$($args:tt)*] ($($tuple:tt)*) $(, $($t:tt)*)?)           => { $crate::rand_value!(@$tag $rng, [$($args)* [$crate::rand_value!(@group $rng, [] $($tuple)*)]] $(, $($t)*)?) };
    (@$tag:ident $rng:expr, [$($args:tt)*] [[$($tt:tt)*]; const $len:expr] $(, $($t:tt)*)?) => { $crate::rand_value!(@$tag $rng, [$($args)* [$crate::rand_value!(@array $rng, [[$($tt)*]] $len)]] $(, $($t)*)?) };
    (@$tag:ident $rng:expr, [$($args:tt)*] [[$($tt:tt)*]; $len:expr] $(, $($t:tt)*)?)       => { $crate::rand_value!(@$tag $rng, [$($args)* [$crate::rand_value!(@repeat $rng, [[$($tt)*]] $len)]] $(, $($t)*)?) };
    (@$tag:ident $rng:expr, [$($args:tt)*] [($($tt:tt)*); const $len:expr] $(, $($t:tt)*)?) => { $crate::rand_value!(@$tag $rng, [$($args)* [$crate::rand_value!(@array $rng, [($($tt)*)] $len)]] $(, $($t)*)?) };
    (@$tag:ident $rng:expr, [$($args:tt)*] [($($tt:tt)*); $len:expr] $(, $($t:tt)*)?)       => { $crate::rand_value!(@$tag $rng, [$($args)* [$crate::rand_value!(@repeat $rng, [($($tt)*)] $len)]] $(, $($t)*)?) };
    (@$tag:ident $rng:expr, [$($args:tt)*] [$ty:expr; const $len:expr] $(, $($t:tt)*)?)     => { $crate::rand_value!(@$tag $rng, [$($args)* [$crate::rand_value!(@array $rng, [$ty] $len)]] $(, $($t)*)?) };
    (@$tag:ident $rng:expr, [$($args:tt)*] [$ty:expr; $len:expr] $(, $($t:tt)*)?)           => { $crate::rand_value!(@$tag $rng, [$($args)* [$crate::rand_value!(@repeat $rng, [$ty] $len)]] $(, $($t)*)?) };
    (@$tag:ident $rng:expr, [$($args:tt)*] [$($tt:tt)*] $(, $($t:tt)*)?)                    => { $crate::rand_value!(@$tag $rng, [$($args)* [$crate::rand_value!(@repeat $rng, [$($tt)*])]] $(, $($t)*)?) };
    (@$tag:ident $rng:expr, [$($args:tt)*] $ty:expr)                                  => { $crate::rand_value!(@$tag $rng, [$($args)* [($rng).random($ty)]]) };
    (@$tag:ident $rng:expr, [$($args:tt)*] $ty:expr, $($t:tt)*)                       => { $crate::rand_value!(@$tag $rng, [$($args)* [($rng).random($ty)]] , $($t)*) };
    (@$tag:ident $rng:expr, [$($args:tt)*] , $($t:tt)*)                               => { $crate::rand_value!(@$tag $rng, [$($args)*] $($t)*) };
    (@$tag:ident $rng:expr, [$($args:tt)*])                                           => { ::std::compile_error!(::std::stringify!($($args)*)) };
    ($rng:expr, $($t:tt)*)                                                            => { $crate::rand_value!(@inner $rng, [] $($t)*) }
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
    (@ty  $rng:expr, [$($p:tt)*] [$($tt:tt)*] ($($x:tt)*) $(, $($t:tt)*)?) => { $crate::rand!(@let $rng, [$($p)*] [$($tt)* ($($x)*)] $(, $($t)*)?) };
    (@ty  $rng:expr, [$($p:tt)*] [$($tt:tt)*] [$($x:tt)*] $(, $($t:tt)*)?) => { $crate::rand!(@let $rng, [$($p)*] [$($tt)* [$($x)*]] $(, $($t)*)?) };
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
    use crate::num::mint_basic::MInt998244353;
    use crate::tools::testutil::integer_boundary_values;
    use std::cell::Cell;

    #[test]
    fn test_spec_vec() {
        let mut rng = Xorshift::default();
        let mut expected = rng.clone();
        for len in 0..=32 {
            let vectors = spec(-100i64..=100).vec(len);
            let actual = rng.random(&vectors);
            let expected: Vec<_> = (0..len).map(|_| expected.random(-100i64..=100)).collect();
            assert_eq!(actual, expected);
        }
        let vectors = spec(-100i64..=100).vec(0usize..=32);
        for _ in 0..1000 {
            let actual = rng.random(&vectors);
            let len = expected.random(0usize..=32);
            let values: Vec<_> = (0..len).map(|_| expected.random(-100i64..=100)).collect();
            assert_eq!(actual, values);
        }
        let values: Vec<i64> = rng.random(spec(..).vec(4));
        for value in values {
            assert_eq!(value, expected.random(..));
        }
        let values: Vec<MInt998244353> = rng.random(spec(..).vec(4));
        for value in values {
            assert_eq!(value, expected.random(..));
        }
        rand!(rng, values: spec(-100i64..=100).vec(4));
        for value in values {
            assert_eq!(value, expected.random(-100i64..=100));
        }
        assert_eq!(rng.rand64(), expected.rand64());
    }

    #[test]
    fn test_spec_vec_evaluation() {
        let mut rng = Xorshift::default();
        for len in 0..=32 {
            let lengths = Cell::new(0);
            let elements = Cell::new(0);
            let length = from_fn(|_| {
                lengths.set(lengths.get() + 1);
                len
            });
            let vectors = from_fn(|_| {
                let next = elements.get();
                elements.set(next + 1);
                next
            })
            .vec(length);
            assert_eq!((lengths.get(), elements.get()), (0, 0));
            for repetition in 0..3 {
                let actual = rng.random(&vectors);
                let start = repetition * len;
                assert_eq!(actual, (start..start + len).collect::<Vec<_>>());
                assert_eq!(lengths.get(), repetition + 1);
                assert_eq!(elements.get(), (repetition + 1) * len);
            }
        }
    }

    #[test]
    fn test_random_range() {
        let mut rng = Xorshift::default();
        macro_rules! check {
            ($($ty:ty),*) => {$(
                let values = integer_boundary_values!($ty);
                let pairs: Vec<_> = values.iter().flat_map(|&a| values.iter().map(move |&b| (a, b))).chain(rng.random_iter((.., ..)).take(10_000)).collect();
                for (a, b) in pairs {
                    let (lo, hi) = (a.min(b), a.max(b));
                    assert!((lo..=hi).contains(&rng.random(lo..=hi)));
                    if lo < hi {
                        assert!((lo..hi).contains(&rng.random(lo..hi)));
                        assert!(rng.random(..hi) < hi);
                    }
                    assert!(rng.random(lo..) >= lo);
                    assert!(rng.random(..=hi) <= hi);
                    assert_eq!(rng.random(a..=a), a);
                }
            )*};
        }
        check!(
            i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize
        );
    }

    #[test]
    fn test_random_segment() {
        let mut rng = Xorshift::default();
        for n in 1..=16usize {
            let mut counts = vec![vec![0; n + 1]; n];
            for _ in 0..10_000 {
                let (l, r) = NotEmptySegment(n).rand(&mut rng);
                assert!(l < r && r <= n);
                counts[l][r] += 1;
            }
            for (l, row) in counts.iter().enumerate() {
                assert!(row[l + 1..].iter().all(|&count| count > 0));
            }
        }
    }

    #[test]
    fn test_rand_macro() {
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let n = rng.random(1..=100);
            let lo = rng.random(-100..=100);
            let hi = lo + rng.random(1..=100);
            rand!(rng, x: lo..hi, lr: NotEmptySegment(n), a: [lo..hi; n], t: (lo..hi,), r: (&(lo..hi), &mut (lo..hi)), p: [(lo..hi, lo..hi); 2]);
            assert!(lr.0 < lr.1 && lr.1 <= n);
            assert_eq!(a.len(), n);
            assert!(
                [x, t.0, r.0, r.1]
                    .into_iter()
                    .chain(a)
                    .chain(p.into_iter().flat_map(|(a, b)| [a, b]))
                    .all(|x| (lo..hi).contains(&x))
            );
        }
    }

    #[test]
    fn test_rand_iter() {
        let mut rng = Xorshift::default();
        let mut expected = rng.clone();
        for len in [0, 1, 32] {
            let values = rng.random_iter(0u32..100);
            assert_eq!(values.size_hint(), (usize::MAX, None));
            let mut values = values.take(len);
            for remaining in (1..=len).rev() {
                assert_eq!(values.size_hint(), (remaining, Some(remaining)));
                assert_eq!(values.next(), Some(expected.random(0u32..100)));
            }
            assert_eq!(values.size_hint(), (0, Some(0)));
            assert_eq!(values.next(), None);
        }
        assert_eq!(rng.rand64(), expected.rand64());
    }

    #[test]
    fn test_rand_value_macro() {
        let mut rng = Xorshift::default();
        let mut expected = rng.clone();
        let range = 0u32..100;
        let (): () = rand_value!(rng, ());
        assert_eq!(rand_value!(rng, (range.clone())), expected.random(&range));
        assert_eq!(
            rand_value!(rng, (range.clone(),)),
            (expected.random(&range),)
        );
        assert_eq!(
            rand_value!(rng, (0u32..100).clone()),
            expected.random(&range)
        );
        assert_eq!(
            rand_value!(rng, [range.clone()][0].clone()),
            expected.random(&range)
        );
        let nested =
            rand_value!(rng, ([[range.clone()][0].clone(); const 2], [(range.clone(),); 2]));
        let expected_nested = (
            std::array::from_fn(|_| expected.random(&range)),
            (0..2)
                .map(|_| (expected.random(&range),))
                .collect::<Vec<_>>(),
        );
        assert_eq!(nested, expected_nested);
        rand!(rng, x: (0u32..100).clone(), (y,): (range.clone(),), z: ([range.clone()][0].clone(), range.clone()), w: [range.clone()][0].clone());
        for actual in [x, y, z.0, z.1, w] {
            assert_eq!(actual, expected.random(&range));
        }
        assert_eq!(rng.rand64(), expected.rand64());
    }

    #[test]
    fn test_rand_value_repetition() {
        let mut rng = Xorshift::default();
        let mut expected = rng.clone();
        for len in [0, 1, 8] {
            let spec_calls = Cell::new(0);
            let len_calls = Cell::new(0);
            let values: Vec<u32> = rand_value!(
                rng,
                [{
                    spec_calls.set(spec_calls.get() + 1);
                    0u32..100
                }; {
                    len_calls.set(len_calls.get() + 1);
                    len
                }]
            );
            assert_eq!(len_calls.get(), 1);
            assert_eq!(spec_calls.get(), len);
            assert_eq!(
                values,
                expected
                    .random_iter(0u32..100)
                    .take(len)
                    .collect::<Vec<_>>()
            );
        }
        let spec_calls = Cell::new(0);
        let values = rand_value!(
            &mut rng,
            [{
                spec_calls.set(spec_calls.get() + 1);
                0u32..100
            }]
        );
        assert_eq!(spec_calls.get(), 0);
        let values: Vec<_> = values.take(8).collect();
        assert_eq!(spec_calls.get(), 8);
        assert_eq!(
            values,
            expected.random_iter(0u32..100).take(8).collect::<Vec<_>>()
        );
        let empty: [u32; 0] = rand_value!(rng, [0u32..100; const 0]);
        assert!(empty.is_empty());
        assert_eq!(rng.rand64(), expected.rand64());
    }

    #[test]
    fn test_weighted_sampler_distribution() {
        for (case, (weights, scale)) in weighted_cases(32_768, 1024).enumerate() {
            let n = weights.len();
            let sampler =
                WeightedSampler::new(weights.iter().map(|&weight| f64::from(weight) * scale));
            let mut distribution = vec![0.0; n];
            for (i, (&prob, &alias)) in sampler.prob.iter().zip(&sampler.alias).enumerate() {
                assert!((0.0..=1.0).contains(&prob));
                assert!(alias < n);
                distribution[i] += prob / n as f64;
                distribution[alias] += (1.0 - prob) / n as f64;
            }
            let total = f64::from(weights.iter().sum::<u32>());
            for (&weight, &actual) in weights.iter().zip(&distribution) {
                let expected = f64::from(weight) / total;
                assert!(
                    (expected - actual).abs() < 1e-12,
                    "case={case}, weights={weights:?}, scale={scale}: {distribution:?}"
                );
                if weight == 0 {
                    assert_eq!(actual, 0.0);
                }
            }
        }
    }

    #[test]
    fn test_weighted_sampler() {
        const DRAWS: usize = 100_000;
        let mut rng = Xorshift::default();
        for (case, (weights, scale)) in weighted_cases(1024, 256).enumerate() {
            let sampler =
                WeightedSampler::new(weights.iter().map(|&weight| f64::from(weight) * scale));
            let mut counts = vec![0usize; weights.len()];
            for _ in 0..DRAWS {
                counts[sampler.rand(&mut rng)] += 1;
            }
            let total = f64::from(weights.iter().sum::<u32>());
            for (&weight, &count) in weights.iter().zip(&counts) {
                let probability = f64::from(weight) / total;
                let expected = probability * DRAWS as f64;
                let deviation = (expected * (1.0 - probability)).sqrt();
                // Eight standard deviations plus eight counts cover rare-event noise.
                assert!(
                    (count as f64 - expected).abs() <= 8.0 * deviation + 8.0,
                    "case={case}, weights={weights:?}, scale={scale}: {counts:?}"
                );
                if weight == 0 {
                    assert_eq!(count, 0);
                }
            }
        }
    }

    fn weighted_cases(cases: usize, max_len: usize) -> impl Iterator<Item = (Vec<u32>, f64)> {
        let mut rng = Xorshift::default();
        (0..cases).map(move |case| {
            let n = match case {
                0 => 1,
                1 => 2,
                2 => max_len,
                _ => rng.random(if case % 4 == 2 { 3 } else { 2 }..=max_len),
            };
            let weights = match case % 4 {
                0 => vec![rng.random(1u32..=256); n],
                1 => {
                    let mut weights = vec![0; n];
                    weights[rng.random(0..n)] = 1;
                    weights
                }
                2 => {
                    let max = rng.random(1u32..=256);
                    let mut weights = rng.random(spec(1..=max).vec(n));
                    weights[..2].fill(max);
                    let positive = rng.random(2..n);
                    weights[positive..].fill(0);
                    rng.shuffle(&mut weights);
                    weights
                }
                _ => {
                    let mut weights = rng.random(spec(1u32..=256).vec(n));
                    let dominant = rng.random(0..n);
                    weights[dominant] = weights.iter().sum::<u32>() * rng.random(2u32..=16);
                    weights
                }
            };
            let max = *weights.iter().max().unwrap();
            let width = max.ilog2() as i32;
            let exponent = match case / 4 % 4 {
                0 => -1074,
                1 => -1023 - width + (case / 16 % 3) as i32,
                2 => rng.random(-1073..1023 - width),
                _ => 1023 - width,
            };
            // Power-of-two scales keep the original integer ratios at floating-point boundaries.
            let scale = if exponent < -1022 {
                f64::from_bits(1 << (exponent + 1074))
            } else {
                f64::from_bits(((exponent + 1023) as u64) << 52)
            };
            (weights, scale)
        })
    }
}
