use super::{AbelianGroup, AbelianMonoid, Group, Monoid, RangeBoundsExt};
use std::{
    fmt::{self, Debug, Formatter},
    iter::FromIterator,
    ops::RangeBounds,
};

/// Accumlated data
pub struct Accumulate<M>
where
    M: Monoid,
{
    data: Vec<M::T>,
}

impl<M> Debug for Accumulate<M>
where
    M: Monoid,
    M::T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Accumulate")
            .field("data", &self.data)
            .finish()
    }
}

impl<M> FromIterator<M::T> for Accumulate<M>
where
    M: Monoid,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = M::T>,
    {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut data = Vec::with_capacity(lower.saturating_add(1));
        let mut acc = M::unit();
        for x in iter {
            let y = M::operate(&acc, &x);
            data.push(acc);
            acc = y;
        }
        data.push(acc);
        Self { data }
    }
}

impl<M> Accumulate<M>
where
    M: Monoid,
{
    /// Return fold of \[0, k\)
    pub fn accumulate(&self, k: usize) -> M::T {
        assert!(
            k < self.data.len(),
            "index out of range: the len is {} but the index is {}",
            self.data.len(),
            k
        );
        unsafe { self.data.get_unchecked(k) }.clone()
    }
}

impl<M> Accumulate<M>
where
    M: Group,
{
    /// Return fold of range
    pub fn fold<R>(&self, range: R) -> M::T
    where
        R: RangeBounds<usize>,
    {
        let n = self.data.len() - 1;
        let range = range.to_range_bounded(0, n).expect("invalid range");
        let (l, r) = (range.start, range.end);
        assert!(l <= r, "bad range [{}, {})", l, r);
        M::operate(&M::inverse(unsafe { self.data.get_unchecked(l) }), unsafe {
            self.data.get_unchecked(r)
        })
    }
}

/// 2-dimensional accumlated data
pub struct Accumulate2d<M>
where
    M: AbelianMonoid,
{
    h: usize,
    w: usize,
    data: Vec<M::T>,
}

impl<M> Accumulate2d<M>
where
    M: AbelianMonoid,
{
    pub fn new(arr2d: &[Vec<M::T>]) -> Self {
        let h = arr2d.len();
        assert!(h > 0);
        let w = arr2d[0].len();
        assert!(w > 0);
        let w1 = w + 1;
        let mut data = Vec::with_capacity((h + 1) * w1);
        data.resize_with(w1, M::unit);
        for (i, arr) in arr2d.iter().enumerate() {
            assert_eq!(w, arr.len(), "expected 2d array");
            let mut acc = M::unit();
            for (j, x) in arr.iter().enumerate() {
                let y = M::operate(&acc, x);
                data.push(M::operate(&acc, unsafe { data.get_unchecked(w1 * i + j) }));
                acc = y;
            }
            data.push(M::operate(&acc, unsafe { data.get_unchecked(w1 * i + w) }));
        }
        Self { h, w, data }
    }
    pub fn from_fn<F>(h: usize, w: usize, mut f: F) -> Self
    where
        F: FnMut(usize, usize) -> M::T,
    {
        let w1 = w + 1;
        let mut data = Vec::with_capacity((h + 1) * w1);
        data.resize_with(w1, M::unit);
        for i in 0..h {
            let mut acc = M::unit();
            for j in 0..w {
                let y = M::operate(&acc, &f(i, j));
                data.push(M::operate(&acc, unsafe { data.get_unchecked(w1 * i + j) }));
                acc = y;
            }
            data.push(M::operate(&acc, unsafe { data.get_unchecked(w1 * i + w) }));
        }
        Self { h, w, data }
    }
}

impl<M> Accumulate2d<M>
where
    M: AbelianMonoid,
{
    /// Return fold of \[0, x\) × \[0, y\)
    pub fn accumulate(&self, x: usize, y: usize) -> M::T {
        let h1 = self.h + 1;
        let w1 = self.w + 1;
        assert!(
            x < h1,
            "index out of range: the first len is {} but the index is {}",
            h1,
            x
        );
        assert!(
            y < w1,
            "index out of range: the second len is {} but the index is {}",
            w1,
            y
        );
        unsafe { self.data.get_unchecked(w1 * x + y) }.clone()
    }
}

impl<M> Accumulate2d<M>
where
    M: AbelianGroup,
{
    /// Return fold of range
    pub fn fold<R0, R1>(&self, range0: R0, range1: R1) -> M::T
    where
        R0: RangeBounds<usize>,
        R1: RangeBounds<usize>,
    {
        let range0 = range0.to_range_bounded(0, self.h).expect("invalid range");
        let range1 = range1.to_range_bounded(0, self.w).expect("invalid range");
        let (xl, xr) = (range0.start, range0.end);
        let (yl, yr) = (range1.start, range1.end);
        assert!(xl <= xr, "bad range [{}, {})", xl, xr);
        assert!(yl <= yr, "bad range [{}, {})", yl, yr);
        let w1 = self.w + 1;
        unsafe {
            M::rinv_operate(
                &M::operate(
                    self.data.get_unchecked(w1 * xl + yl),
                    self.data.get_unchecked(w1 * xr + yr),
                ),
                &M::operate(
                    self.data.get_unchecked(w1 * xl + yr),
                    self.data.get_unchecked(w1 * xr + yl),
                ),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, LinearOperation, Magma, Unital},
        num::mint_basic::MInt1000000007,
        rand,
        tools::{RandomSpec, Xorshift},
    };
    type M = LinearOperation<MInt1000000007>;
    type A = AdditiveOperation<MInt1000000007>;
    struct D;
    impl RandomSpec<MInt1000000007> for D {
        fn rand(&self, rng: &mut Xorshift) -> MInt1000000007 {
            MInt1000000007::new_unchecked(rng.gen(..MInt1000000007::get_mod()))
        }
    }

    #[test]
    fn test_accumlate() {
        let mut rng = Xorshift::default();
        const Q: usize = 1_000;
        const N: usize = 50;
        for n in 0..Q {
            let n = n % N;
            rand!(rng, v: [(D, D); n]);
            let acc: Accumulate<M> = v.iter().cloned().collect();
            for r in 0..=n {
                assert_eq!(
                    v[..r].iter().fold(M::unit(), |x, y| M::operate(&x, y)),
                    acc.accumulate(r)
                );
                for l in 0..=r {
                    assert_eq!(
                        v[l..r].iter().fold(M::unit(), |x, y| M::operate(&x, y)),
                        acc.fold(l..r)
                    );
                }
            }
        }
    }

    #[test]
    fn test_accumlate2d() {
        let mut rng = Xorshift::default();
        const Q: usize = 1_000;
        const N: usize = 10;
        for i in 0..Q {
            let h = i % N + 1;
            let w = i / N % N + 1;
            rand!(rng, v: [[D; w]; h]);
            let acc2d = Accumulate2d::<A>::new(&v);
            for xr in 0..=h {
                for yr in 0..=w {
                    assert_eq!(
                        v[..xr]
                            .iter()
                            .flat_map(|v| v[..yr].iter())
                            .fold(A::unit(), |x, y| A::operate(&x, y)),
                        acc2d.accumulate(xr, yr)
                    );
                    for xl in 0..=xr {
                        for yl in 0..=yr {
                            assert_eq!(
                                v[xl..xr]
                                    .iter()
                                    .flat_map(|v| v[yl..yr].iter())
                                    .fold(A::unit(), |x, y| A::operate(&x, y)),
                                acc2d.fold(xl..xr, yl..yr)
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_accumlate2d_from_fn() {
        let mut rng = Xorshift::default();
        const Q: usize = 1_000;
        const N: usize = 10;
        for i in 0..Q {
            let h = i % N;
            let w = i / N % N;
            rand!(rng, v: [[D; w]; h]);
            let acc2d = Accumulate2d::<A>::from_fn(h, w, |i, j| v[i][j]);
            for xr in 0..=h {
                for yr in 0..=w {
                    assert_eq!(
                        v[..xr]
                            .iter()
                            .flat_map(|v| v[..yr].iter())
                            .fold(A::unit(), |x, y| A::operate(&x, y)),
                        acc2d.accumulate(xr, yr)
                    );
                    for xl in 0..=xr {
                        for yl in 0..=yr {
                            assert_eq!(
                                v[xl..xr]
                                    .iter()
                                    .flat_map(|v| v[yl..yr].iter())
                                    .fold(A::unit(), |x, y| A::operate(&x, y)),
                                acc2d.fold(xl..xr, yl..yr)
                            );
                        }
                    }
                }
            }
        }
    }
}
