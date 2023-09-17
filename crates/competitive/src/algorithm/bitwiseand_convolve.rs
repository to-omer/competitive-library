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
        crate::avx_helper!(
            @avx2 fn zeta_transform<M>(f: &mut [M::T])
            where
                [M: Monoid]
            {
                let k = f.len().trailing_zeros() as usize;
                assert_eq!(f.len(), 1 << k);
                let n = f.len();
                assert_eq!(n.count_ones(), 1);
                for i in 0..k {
                    if i == 0 {
                        for c in f.chunks_exact_mut(2) {
                            let (x, y) = c.split_at_mut(1);
                            for (x, y) in x.iter_mut().zip(y) {
                                *x = M::operate(x, y);
                            }
                        }
                    } else if i == 1 {
                        for c in f.chunks_exact_mut(4) {
                            let (x, y) = c.split_at_mut(2);
                            for (x, y) in x.iter_mut().zip(y) {
                                *x = M::operate(x, y);
                            }
                        }
                    } else if i == 2 {
                        for c in f.chunks_exact_mut(8) {
                            let (x, y) = c.split_at_mut(4);
                            for (x, y) in x.iter_mut().zip(y) {
                                *x = M::operate(x, y);
                            }
                        }
                    } else {
                        assert!(i >= 3);
                        for c in f.chunks_exact_mut(2 << i) {
                            let (x, y) = c.split_at_mut(1 << i);
                            for (x, y) in x.iter_mut().zip(y) {
                                *x = M::operate(x, y);
                            }
                        }
                    }
                }
            }
        );
        zeta_transform::<M>(f);
    }
}

impl<G> BitwiseandConvolve<G>
where
    G: Group,
{
    /// $$f(m) = \sum_{n \mid m}h(n)$$
    pub fn mobius_transform(f: &mut [G::T]) {
        crate::avx_helper!(
            @avx2 fn mobius_transform<G>(f: &mut [G::T])
            where
                [G: Group]
            {
                let k = f.len().trailing_zeros() as usize;
                assert_eq!(f.len(), 1 << k);
                let n = f.len();
                assert_eq!(n.count_ones(), 1);
                for i in 0..k {
                    if i == 0 {
                        for c in f.chunks_exact_mut(2) {
                            let (x, y) = c.split_at_mut(1);
                            for (x, y) in x.iter_mut().zip(y) {
                                *x = G::rinv_operate(x, y);
                            }
                        }
                    } else if i == 1 {
                        for c in f.chunks_exact_mut(4) {
                            let (x, y) = c.split_at_mut(2);
                            for (x, y) in x.iter_mut().zip(y) {
                                *x = G::rinv_operate(x, y);
                            }
                        }
                    } else if i == 2 {
                        for c in f.chunks_exact_mut(8) {
                            let (x, y) = c.split_at_mut(4);
                            for (x, y) in x.iter_mut().zip(y) {
                                *x = G::rinv_operate(x, y);
                            }
                        }
                    } else {
                        assert!(i >= 3);
                        for c in f.chunks_exact_mut(2 << i) {
                            let (x, y) = c.split_at_mut(1 << i);
                            for (x, y) in x.iter_mut().zip(y) {
                                *x = G::rinv_operate(x, y);
                            }
                        }
                    }
                }
            }
        );
        mobius_transform::<G>(f);
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
