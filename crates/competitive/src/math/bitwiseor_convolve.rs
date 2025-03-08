use super::{ConvolveSteps, Group, Invertible, Monoid, Ring, bitwise_transform};
use std::marker::PhantomData;

pub struct BitwiseorConvolve<M> {
    _marker: PhantomData<fn() -> M>,
}

impl<M> BitwiseorConvolve<M>
where
    M: Monoid,
{
    /// $$g(m) = \sum_{n \mid m}f(n)$$
    pub fn zeta_transform(f: &mut [M::T]) {
        bitwise_transform(f, |y, x| *x = M::operate(x, y));
    }
}

impl<G> BitwiseorConvolve<G>
where
    G: Group,
{
    /// $$f(m) = \sum_{n \mid m}h(n)$$
    pub fn mobius_transform(f: &mut [G::T]) {
        bitwise_transform(f, |y, x| *x = G::rinv_operate(x, y));
    }
}

impl<R> ConvolveSteps for BitwiseorConvolve<R>
where
    R: Ring,
    R::T: PartialEq,
    R::Additive: Invertible,
{
    type T = Vec<R::T>;
    type F = Vec<R::T>;

    fn length(t: &Self::T) -> usize {
        t.len()
    }

    fn transform(mut t: Self::T, _len: usize) -> Self::F {
        BitwiseorConvolve::<R::Additive>::zeta_transform(&mut t);
        t
    }

    fn inverse_transform(mut f: Self::F, _len: usize) -> Self::T {
        BitwiseorConvolve::<R::Additive>::mobius_transform(&mut f);
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
    use crate::{
        algebra::{AddMulOperation, AdditiveOperation},
        rand,
        tools::Xorshift,
    };

    const A: i64 = 100_000;

    #[test]
    fn test_bitwiseor_convolve() {
        let mut rng = Xorshift::new();

        for k in 0..12 {
            let n = 1 << k;
            rand!(rng, mut f: [-A..A; n]);
            let mut g = vec![0i64; n];
            let h = f.clone();
            for (s, f) in f.iter().enumerate() {
                for (t, g) in g.iter_mut().enumerate() {
                    if s | t == t {
                        *g += f;
                    }
                }
            }
            BitwiseorConvolve::<AdditiveOperation<i64>>::zeta_transform(&mut f);
            assert_eq!(f, g);
            BitwiseorConvolve::<AdditiveOperation<i64>>::mobius_transform(&mut f);
            assert_eq!(f, h);

            rand!(rng, f: [-A..A; n], g: [-A..A; n]);
            let mut h = vec![0i64; n];
            for i in 0..n {
                for j in 0..n {
                    h[i | j] += f[i] * g[j];
                }
            }
            let i = BitwiseorConvolve::<AddMulOperation<i64>>::convolve(f, g);
            assert_eq!(h, i);
        }
    }
}
