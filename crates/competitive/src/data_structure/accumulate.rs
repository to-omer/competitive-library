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

impl<M> Debug for Accumulate2d<M>
where
    M: AbelianMonoid,
    M::T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Accumulate2d")
            .field("h", &self.h)
            .field("w", &self.w)
            .field("data", &self.data)
            .finish()
    }
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

pub struct AccumulateKd<const K: usize, M>
where
    M: AbelianMonoid,
{
    dim: [usize; K],
    offset: [usize; K],
    data: Vec<M::T>,
}

impl<const K: usize, M> Debug for AccumulateKd<K, M>
where
    M: AbelianMonoid,
    M::T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccumulateKd")
            .field("dim", &self.dim)
            .field("offset", &self.offset)
            .field("data", &self.data)
            .finish()
    }
}

impl<const K: usize, M> AccumulateKd<K, M>
where
    M: AbelianMonoid,
{
    pub fn from_fn(dim: [usize; K], mut f: impl FnMut([usize; K]) -> M::T) -> Self {
        fn fill<const K: usize, T>(
            dim: &[usize; K],
            offset: &[usize; K],
            data: &mut [T],
            f: &mut impl FnMut([usize; K]) -> T,
            mut index: [usize; K],
            pos: usize,
        ) {
            if pos < K {
                for i in 0..dim[pos] {
                    index[pos] = i;
                    fill(dim, offset, data, f, index, pos + 1);
                }
            } else {
                let i: usize = index.iter().zip(offset).map(|(x, y)| (x + 1) * y).sum();
                data[i] = f(index);
            }
        }

        let mut offset = [1; K];
        for d in (1..K).rev() {
            offset[d - 1] = offset[d] * (dim[d] + 1);
        }
        let size = offset[0] * (dim[0] + 1);
        let mut data = vec![M::unit(); size];
        fill(&dim, &offset, &mut data, &mut f, [0; K], 0);
        for d in 0..K {
            for i in 1..size {
                if i / offset[d] % (dim[d] + 1) != 0 {
                    data[i] = M::operate(&data[i], &data[i - offset[d]]);
                }
            }
        }
        Self { dim, offset, data }
    }
    pub fn accumulate(&self, x: [usize; K]) -> M::T {
        for (d, x) in x.into_iter().enumerate() {
            assert!(
                x <= self.dim[d],
                "index out of range: the len is {} but the index is {}",
                self.dim[d] + 1,
                x
            );
        }
        let p: usize = x.iter().zip(&self.offset).map(|(x, y)| x * y).sum();
        unsafe { self.data.get_unchecked(p) }.clone()
    }
}

impl<const K: usize, M> AccumulateKd<K, M>
where
    M: AbelianGroup,
{
    pub fn fold<R>(&self, ranges: [R; K]) -> M::T
    where
        R: RangeBounds<usize>,
    {
        let ranges: [_; K] = std::array::from_fn(|i| {
            let range = ranges[i]
                .to_range_bounded(0, self.dim[i])
                .expect("invalid range");
            let (l, r) = (range.start, range.end);
            assert!(l <= r, "bad range [{}, {})", l, r);
            [l, r]
        });
        let mut acc = M::unit();
        for bit in 0..1 << K {
            let p: usize = ranges
                .iter()
                .zip(&self.offset)
                .enumerate()
                .map(|(d, (range, offset))| range[(bit >> d) & 1 ^ 1] * offset)
                .sum();
            if bit.count_ones() & 1 == 0 {
                acc = M::operate(&acc, unsafe { self.data.get_unchecked(p) });
            } else {
                acc = M::rinv_operate(&acc, unsafe { self.data.get_unchecked(p) });
            }
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, LinearOperation, Magma, Unital},
        num::mint_basic::MInt1000000007,
        rand,
        tools::Xorshift,
    };
    type M = LinearOperation<MInt1000000007>;
    type A = AdditiveOperation<MInt1000000007>;

    #[test]
    fn test_accumlate() {
        let mut rng = Xorshift::default();
        const Q: usize = 1_000;
        const N: usize = 50;
        for n in 0..Q {
            let n = n % N;
            rand!(rng, v: [(.., ..); n]);
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
            rand!(rng, v: [[..; w]; h]);
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
            rand!(rng, v: [[..; w]; h]);
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

    #[test]
    fn test_accumlatekd_from_fn_3d() {
        let mut rng = Xorshift::default();
        const N: usize = 5;
        for i in 0..N * N * N {
            let dim = [i % N, i / N % N, i / N / N % N];
            rand!(rng, v: [[[..; dim[2]]; dim[1]]; dim[0]]);
            let acc = AccumulateKd::<3, A>::from_fn(dim, |[i, j, k]| v[i][j][k]);
            for xr in 0..=dim[0] {
                for yr in 0..=dim[1] {
                    for zr in 0..=dim[2] {
                        assert_eq!(
                            v[..xr]
                                .iter()
                                .flat_map(|v| v[..yr].iter().flat_map(|v| v[..zr].iter()))
                                .fold(A::unit(), |x, y| A::operate(&x, y)),
                            acc.accumulate([xr, yr, zr])
                        );
                        for xl in 0..=xr {
                            for yl in 0..=yr {
                                for zl in 0..=zr {
                                    assert_eq!(
                                        v[xl..xr]
                                            .iter()
                                            .flat_map(|v| v[yl..yr]
                                                .iter()
                                                .flat_map(|v| v[zl..zr].iter()))
                                            .fold(A::unit(), |x, y| A::operate(&x, y)),
                                        acc.fold([xl..xr, yl..yr, zl..zr])
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_accumlatekd_from_fn_4d() {
        let mut rng = Xorshift::default();
        const N: usize = 4;
        for i in 0..N * N * N * N {
            let dim = [i % N, i / N % N, i / N / N % N, i / N / N / N % N];
            rand!(rng, v: [[[[..; dim[3]]; dim[2]]; dim[1]]; dim[0]]);
            let acc = AccumulateKd::<4, A>::from_fn(dim, |[i, j, k, l]| v[i][j][k][l]);
            for xr in 0..=dim[0] {
                for yr in 0..=dim[1] {
                    for zr in 0..=dim[2] {
                        for wr in 0..=dim[3] {
                            assert_eq!(
                                v[..xr]
                                    .iter()
                                    .flat_map(|v| v[..yr]
                                        .iter()
                                        .flat_map(|v| v[..zr].iter().flat_map(|v| v[..wr].iter())))
                                    .fold(A::unit(), |x, y| A::operate(&x, y)),
                                acc.accumulate([xr, yr, zr, wr])
                            );
                            for xl in 0..=xr {
                                for yl in 0..=yr {
                                    for zl in 0..=zr {
                                        for wl in 0..=wr {
                                            assert_eq!(
                                                v[xl..xr]
                                                    .iter()
                                                    .flat_map(|v| v[yl..yr]
                                                        .iter()
                                                        .flat_map(|v| v[zl..zr]
                                                            .iter()
                                                            .flat_map(|v| v[wl..wr].iter())))
                                                    .fold(A::unit(), |x, y| A::operate(&x, y)),
                                                acc.fold([xl..xr, yl..yr, zl..zr, wl..wr])
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
