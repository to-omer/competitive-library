use super::{ConvolveSteps, Group, Invertible, Monoid, Ring};
use std::marker::PhantomData;

pub struct BitwiseandConvolve<M> {
    _marker: PhantomData<fn() -> M>,
}

impl<M> BitwiseandConvolve<M>
where
    M: Monoid,
{
    /// $$g(m) = \sum_{n \mid m}f(n)$$
    pub fn zeta_transform(f: &mut [M::T]) {
        let n = f.len();
        let mut i = 1;
        while i < n {
            for j in 0..n {
                if j & i == 0 {
                    f[j] = M::operate(&f[j], &f[j | i]);
                }
            }
            i <<= 1;
        }
    }
}

impl<G> BitwiseandConvolve<G>
where
    G: Group,
{
    /// $$f(m) = \sum_{n \mid m}h(n)$$
    pub fn mobius_transform(f: &mut [G::T]) {
        let n = f.len();
        let mut i = 1;
        while i < n {
            for j in 0..n {
                if j & i == 0 {
                    f[j] = G::rinv_operate(&f[j], &f[j | i]);
                }
            }
            i <<= 1;
        }
    }
}

impl<R> ConvolveSteps for BitwiseandConvolve<R>
where
    R: Ring,
    R::Additive: Invertible,
{
    type T = Vec<R::T>;
    type F = Vec<R::T>;

    fn length(t: &Self::T) -> usize {
        t.len()
    }

    fn transform(mut t: Self::T, _len: usize) -> Self::F {
        BitwiseandConvolve::<R::Additive>::zeta_transform(&mut t);
        t
    }

    fn inverse_transform(mut f: Self::F, _len: usize) -> Self::T {
        BitwiseandConvolve::<R::Additive>::mobius_transform(&mut f);
        f
    }

    fn multiply(f: &mut Self::F, g: &Self::F) {
        for (f, g) in f.iter_mut().zip(g) {
            *f = R::mul(f, g);
        }
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
    fn test_bitwiseand_convolve() {
        let mut rng = Xorshift::new();

        for k in 0..12 {
            let n = 1 << k;
            rand!(rng, mut f: [-A..A; n]);
            let mut g = vec![0i64; n];
            let h = f.clone();
            for (s, f) in f.iter().enumerate() {
                for (t, g) in g.iter_mut().enumerate() {
                    if s | t == s {
                        *g += f;
                    }
                }
            }
            BitwiseandConvolve::<AdditiveOperation<i64>>::zeta_transform(&mut f);
            assert_eq!(f, g);
            BitwiseandConvolve::<AdditiveOperation<i64>>::mobius_transform(&mut f);
            assert_eq!(f, h);

            rand!(rng, f: [-A..A; n], g: [-A..A; n]);
            let mut h = vec![0i64; n];
            for i in 0..n {
                for j in 0..n {
                    h[i & j] += f[i] * g[j];
                }
            }
            let i = BitwiseandConvolve::<AddMulOperation<i64>>::convolve(f, g);
            assert_eq!(h, i);
        }
    }
}
