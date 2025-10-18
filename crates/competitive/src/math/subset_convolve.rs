use super::{BitwiseorConvolve, ConvolveSteps, Invertible, Ring};
use std::marker::PhantomData;

pub struct SubsetConvolve<M> {
    _marker: PhantomData<fn() -> M>,
}

impl<R> ConvolveSteps for SubsetConvolve<R>
where
    R: Ring<T: PartialEq, Additive: Invertible>,
{
    type T = Vec<R::T>;
    type F = Vec<Vec<R::T>>;

    fn length(t: &Self::T) -> usize {
        t.len()
    }

    fn transform(t: Self::T, len: usize) -> Self::F {
        let k = len.trailing_zeros() as usize;
        let mut f = vec![vec![R::zero(); len]; k + 1];
        for (i, t) in t.iter().enumerate() {
            let f = &mut f[i.count_ones() as usize][i];
            *f = R::add(f, t);
        }
        for f in f.iter_mut() {
            BitwiseorConvolve::<R::Additive>::zeta_transform(f);
        }
        f
    }

    fn inverse_transform(mut f: Self::F, len: usize) -> Self::T {
        for f in f.iter_mut() {
            BitwiseorConvolve::<R::Additive>::mobius_transform(f);
        }
        let mut t = vec![R::zero(); len];
        for (i, t) in t.iter_mut().enumerate() {
            *t = R::add(t, &f[i.count_ones() as usize][i]);
        }
        t
    }

    fn multiply(f: &mut Self::F, g: &Self::F) {
        for i in (0..f.len()).rev() {
            let (lf, rf) = f.split_at_mut(i);
            for (x, y) in rf[0].iter_mut().zip(&g[0]) {
                *x = R::mul(x, y);
            }
            for (x, y) in lf.iter().rev().zip(g.iter().skip(1)) {
                for ((x, y), z) in x.iter().zip(y).zip(&mut rf[0]) {
                    *z = R::add(z, &R::mul(x, y));
                }
            }
        }
    }

    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        assert_eq!(a.len(), b.len());
        let len = a.len();
        let same = a == b;
        let mut a = Self::transform(a, len);
        let b = if same {
            a.clone()
        } else {
            Self::transform(b, len)
        };
        Self::multiply(&mut a, &b);
        Self::inverse_transform(a, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algebra::AddMulOperation, rand, tools::Xorshift};

    const A: i64 = 100_000;

    #[test]
    fn test_subset_convolve() {
        let mut rng = Xorshift::default();

        for k in 0..12 {
            let n = 1 << k;
            rand!(rng, f: [-A..A; n], g: [-A..A; n]);
            let mut h = vec![0i64; n];
            for i in 0..n {
                for j in 0..n {
                    if i & j == 0 {
                        h[i | j] += f[i] * g[j];
                    }
                }
            }
            let i = SubsetConvolve::<AddMulOperation<i64>>::convolve(f, g);
            assert_eq!(h, i);
        }
    }
}
