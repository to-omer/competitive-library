use std::ops::RangeInclusive;

/// fibonacci search helper
pub trait FibonacciSearch: Sized {
    fn fibonacci_search<T, F>(self, other: Self, f: F) -> (Self, T)
    where
        T: PartialOrd,
        F: FnMut(Self) -> T;
}
macro_rules! impl_fibonacci_search_unsigned {
    ($($t:ty)*) => {
        $(impl FibonacciSearch for $t {
            fn fibonacci_search<T, F>(self, other: Self, mut f: F) -> (Self, T)
            where
                T: PartialOrd,
                F: FnMut(Self) -> T,
            {
                let l = self;
                let r = other;
                assert!(l <= r);
                const W: usize = [12, 23, 46, 92, 185][<$t>::BITS.ilog2() as usize - 3];
                const FIB: [$t; W] = {
                    let mut fib = [0; W];
                    fib[0] = 1;
                    fib[1] = 2;
                    let mut i = 2;
                    while i < W {
                        fib[i] = fib[i - 1] + fib[i - 2];
                        i += 1;
                    }
                    fib
                };
                let mut s = l;
                let mut v0 = None;
                let mut v1 = None;
                let mut v2 = None;
                let mut v3 = None;
                for w in FIB[..FIB.partition_point(|&f| f < r - l)].windows(2).rev() {
                    let (w0, w1) = (w[0], w[1]);
                    if w1 > r - s || v1.get_or_insert_with(|| f(s + w0)) <= v2.get_or_insert_with(|| f(s + w1)) {
                        v3 = v2;
                        v2 = v1;
                        v1 = None;
                    } else {
                        v0 = v1;
                        v1 = v2;
                        v2 = None;
                        s += w0;
                    }
                }
                let mut kv = (s, v0.unwrap_or_else(|| f(s)));
                if s < r {
                    let v = v1.or(v2).unwrap_or_else(|| f(s + 1));
                    if v < kv.1 {
                        kv = (s + 1, v);
                    }
                    if s + 1 < r {
                        let v = v3.unwrap_or_else(|| f(s + 2));
                        if v < kv.1 {
                            kv = (s + 2, v);
                        }
                    }
                }
                kv
            }
        })*
    };
}
impl_fibonacci_search_unsigned!(u8 u16 u32 u64 u128 usize);

/// ternary search helper
pub trait Trisect: Clone {
    type Key: FibonacciSearch;
    fn trisect_key(self) -> Self::Key;
    fn trisect_unkey(key: Self::Key) -> Self;
}

macro_rules! impl_trisect_unsigned {
    ($($t:ty)*) => {
        $(impl Trisect for $t {
            type Key = $t;
            fn trisect_key(self) -> Self::Key {
                self
            }
            fn trisect_unkey(key: Self::Key) -> Self {
                key
            }
        })*
    };
}
macro_rules! impl_trisect_signed {
    ($({$i:ident $u:ident})*) => {
        $(impl Trisect for $i {
            type Key = $u;
            fn trisect_key(self) -> Self::Key {
                (self as $u) ^ (1 << <$u>::BITS - 1)
            }
            fn trisect_unkey(key: Self::Key) -> Self {
                (key ^ (1 << <$u>::BITS - 1)) as $i
            }
        })*
    };
}
macro_rules! impl_trisect_float {
    ($({$t:ident $u:ident $i:ident})*) => {
        $(impl Trisect for $t {
            type Key = $u;
            fn trisect_key(self) -> Self::Key {
                let a = self.to_bits() as $i;
                (a ^ (((a >> <$u>::BITS - 1) as $u) >> 1) as $i) as $u ^ (1 << <$u>::BITS - 1)
            }
            fn trisect_unkey(key: Self::Key) -> Self {
                let key = (key  ^ (1 << <$u>::BITS - 1)) as $i;
                $t::from_bits((key ^ (((key >> <$u>::BITS - 1) as $u) >> 1) as $i) as _)
            }
        })*
    };
}

impl_trisect_unsigned!(u8 u16 u32 u64 u128 usize);
impl_trisect_signed!({i8 u8} {i16 u16} {i32 u32} {i64 u64} {i128 u128} {isize usize});
impl_trisect_float!({f32 u32 i32} {f64 u64 i64});

/// Returns the element that gives the minimum value from the strictly concave up function and the minimum value.
pub fn ternary_search<K, V, F>(range: RangeInclusive<K>, mut f: F) -> (K, V)
where
    K: Trisect,
    V: PartialOrd,
    F: FnMut(K) -> V,
{
    let (l, r) = range.into_inner();
    let (k, v) =
        <K::Key as FibonacciSearch>::fibonacci_search(l.trisect_key(), r.trisect_key(), |x| {
            f(Trisect::trisect_unkey(x))
        });
    (K::trisect_unkey(k), v)
}

pub fn piecewise_ternary_search<const N: usize, K, V, F>(piece: [K; N], mut f: F) -> (K, V)
where
    K: Trisect,
    V: PartialOrd,
    F: FnMut(K) -> V,
{
    piece
        .windows(2)
        .map(|w| ternary_search(w[0].clone()..=w[1].clone(), &mut f))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap_or_else(|| (piece[0].clone(), f(piece[0].clone())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{num::DoubleDouble, tools::Xorshift};

    #[test]
    fn test_trisect_unsigned() {
        for p in 0u8..=u8::MAX {
            assert_eq!(p, u8::trisect_unkey(p.trisect_key()));
            for q in 0u8..=u8::MAX {
                assert_eq!(p.cmp(&q), p.trisect_key().cmp(&q.trisect_key()));
            }
        }
    }

    #[test]
    fn test_trisect_signed() {
        for p in i8::MIN..=i8::MAX {
            assert_eq!(p, i8::trisect_unkey(p.trisect_key()));
            for q in i8::MIN..=i8::MAX {
                assert_eq!(p.cmp(&q), p.trisect_key().cmp(&q.trisect_key()));
            }
        }
    }

    #[test]
    fn test_trisect_float() {
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let p = (rng.randf() - 0.5) * 200.;
            assert_eq!(p, f64::trisect_unkey(p.trisect_key()));
            let q = (rng.randf() - 0.5) * 200.;
            assert_eq!(
                p.partial_cmp(&q),
                p.trisect_key().partial_cmp(&q.trisect_key())
            );
        }
    }

    #[test]
    fn test_ternary_search_unsigned() {
        for p in 0u8..=u8::MAX {
            for l in 0u8..=u8::MAX {
                for r in l..=u8::MAX {
                    let f = |x| p.abs_diff(x);
                    assert_eq!(
                        f(l).min(f(r)).min(f(p.clamp(l, r))),
                        ternary_search(l..=r, f).1,
                    );
                }
            }
        }
    }

    #[test]
    fn test_ternary_search_signed() {
        for p in -20..=20 {
            assert_eq!(
                p.clamp(-10, 10),
                ternary_search(-10i64..=10, |x| 10 * (x - p).pow(2) + 5).0,
            );
        }
    }

    #[test]
    fn test_ternary_search_float() {
        assert_eq!(
            std::f64::consts::PI,
            ternary_search(f64::MIN..=f64::MAX, |x| (DoubleDouble::from(x)
                - DoubleDouble::from(std::f64::consts::PI))
            .abs())
            .0,
        );

        for a in 0..1000 {
            assert_eq!(
                0.0f64,
                piecewise_ternary_search([0.0, 1e-9, 1.0], |x| (x - (a as f64) / 1000.0).powi(2)).1,
            )
        }
    }
}
