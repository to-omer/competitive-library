use super::{ConvolveSteps, Field, Group, Invertible};
use std::{fmt::Debug, marker::PhantomData};

trait FromLength<const EXACT_DIVISION: bool> {
    fn from_length(len: usize) -> Self;
}

impl<T> FromLength<false> for T
where
    T: From<usize>,
{
    fn from_length(len: usize) -> Self {
        T::from(len)
    }
}

impl<T> FromLength<true> for T
where
    T: TryFrom<usize>,
    T::Error: Debug,
{
    fn from_length(len: usize) -> Self {
        T::try_from(len).unwrap()
    }
}

/// `EXACT_DIVISION` normalizes with division instead of a multiplicative inverse.
pub struct BitwisexorConvolve<M, const EXACT_DIVISION: bool = false> {
    _marker: PhantomData<fn() -> M>,
}

impl<G, const EXACT_DIVISION: bool> BitwisexorConvolve<G, EXACT_DIVISION>
where
    G: Group,
{
    pub fn hadamard_transform(f: &mut [G::T]) {
        let k = f.len().trailing_zeros() as usize;
        assert!(f.len() == 1 << k);
        if k & 1 != 0 {
            for [x, y] in f.as_chunks_mut::<2>().0 {
                let t = G::operate(x, y);
                *y = G::rinv_operate(x, y);
                *x = t;
            }
        }
        for i in (k & 1..k).step_by(2) {
            for chunk in f.chunks_exact_mut(4 << i) {
                let (left, right) = chunk.split_at_mut(2 << i);
                let (a, b) = left.split_at_mut(1 << i);
                let (c, d) = right.split_at_mut(1 << i);
                for (((a, b), c), d) in a.iter_mut().zip(b).zip(c).zip(d) {
                    let x = G::operate(a, b);
                    let y = G::rinv_operate(a, b);
                    let z = G::operate(c, d);
                    let w = G::rinv_operate(c, d);
                    *a = G::operate(&x, &z);
                    *b = G::operate(&y, &w);
                    *c = G::rinv_operate(&x, &z);
                    *d = G::rinv_operate(&y, &w);
                }
            }
        }
    }
}

impl<R, const EXACT_DIVISION: bool> ConvolveSteps for BitwisexorConvolve<R, EXACT_DIVISION>
where
    R: Field<
            T: PartialEq + FromLength<EXACT_DIVISION>,
            Additive: Invertible,
            Multiplicative: Invertible,
        >,
{
    type T = Vec<R::T>;
    type F = Vec<R::T>;

    fn length(t: &Self::T) -> usize {
        t.len()
    }

    fn transform(mut t: Self::T, _len: usize) -> Self::F {
        BitwisexorConvolve::<R::Additive, EXACT_DIVISION>::hadamard_transform(&mut t);
        t
    }

    fn inverse_transform(mut f: Self::F, len: usize) -> Self::T {
        BitwisexorConvolve::<R::Additive, EXACT_DIVISION>::hadamard_transform(&mut f);
        let len = R::T::from_length(len);
        if EXACT_DIVISION {
            for f in &mut f {
                *f = R::div(f, &len);
            }
        } else if !f.is_empty() {
            let inv_len = R::inv(&len);
            for f in &mut f {
                *f = R::mul(f, &inv_len);
            }
        }
        f
    }

    fn multiply(f: &mut Self::F, g: &Self::F) {
        for (f, g) in f.iter_mut().zip(g) {
            *f = R::mul(f, g);
        }
    }

    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        assert_eq!(a.len(), b.len());
        let len = a.len();
        let same = a == b;
        let mut a = Self::transform(a, len);
        if same {
            for a in a.iter_mut() {
                *a = R::mul(a, a);
            }
        } else {
            let b = Self::transform(b, len);
            Self::multiply(&mut a, &b);
        }
        Self::inverse_transform(a, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algebra::AddMulOperation, rand, tools::Xorshift};

    const A: i64 = 100_000;

    #[test]
    fn test_bitwisexor_convolve() {
        let mut rng = Xorshift::default();

        for k in 0..12 {
            let n = 1 << k;
            rand!(rng, f: [-A..A; n], g: [-A..A; n]);
            let mut h = vec![0i64; n];
            for i in 0..n {
                for j in 0..n {
                    h[i ^ j] += f[i] * g[j];
                }
            }
            let i = BitwisexorConvolve::<AddMulOperation<i64>, true>::convolve(f, g);
            assert_eq!(h, i);
        }
    }
}
