use super::{with_prime_list, ConvolveSteps, Group, Invertible, Monoid, Ring};
use std::marker::PhantomData;

pub struct GcdConvolve<M> {
    _marker: PhantomData<fn() -> M>,
}

impl<M> GcdConvolve<M>
where
    M: Monoid,
{
    /// $$g(m) = \sum_{n \mid m}f(n)$$
    pub fn zeta_transform(f: &mut [M::T]) {
        let n = f.len().saturating_sub(1) as u64;
        with_prime_list(n, |pl| {
            for &p in pl.primes_lte(n).iter() {
                for (i, j) in (0..f.len()).step_by(p as _).enumerate().rev() {
                    f[i] = M::operate(&f[i], &f[j]);
                }
            }
        })
    }
}

impl<G> GcdConvolve<G>
where
    G: Group,
{
    /// $$f(m) = \sum_{n \mid m}h(n)$$
    pub fn mobius_transform(f: &mut [G::T]) {
        let n = f.len().saturating_sub(1) as u64;
        with_prime_list(n, |pl| {
            for &p in pl.primes_lte(n).iter() {
                for (i, j) in (0..f.len()).step_by(p as _).enumerate() {
                    f[i] = G::rinv_operate(&f[i], &f[j]);
                }
            }
        })
    }
}

impl<R> ConvolveSteps for GcdConvolve<R>
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
        GcdConvolve::<R::Additive>::zeta_transform(&mut t);
        t
    }

    fn inverse_transform(mut f: Self::F, _len: usize) -> Self::T {
        GcdConvolve::<R::Additive>::mobius_transform(&mut f);
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
        math::gcd,
        rand,
        tools::Xorshift,
    };

    const A: i64 = 100_000;

    #[test]
    fn test_gcd_convolve() {
        let mut rng = Xorshift::default();

        for m in 1..=300 {
            rand!(rng, mut f: [-A..A; m]);
            f[0] = 0;
            let mut g = vec![0i64; m];
            let h = f.clone();
            for (s, f) in f.iter().enumerate().skip(1) {
                for (t, g) in g.iter_mut().enumerate().skip(1) {
                    if s % t == 0 {
                        *g += f;
                    }
                }
            }
            GcdConvolve::<AdditiveOperation<i64>>::zeta_transform(&mut f);
            assert_eq!(&f[1..], &g[1..]);
            GcdConvolve::<AdditiveOperation<i64>>::mobius_transform(&mut f);
            assert_eq!(&f[1..], &h[1..]);

            rand!(rng, mut f: [-A..A; m], mut g: [-A..A; m]);
            f[0] = 0;
            g[0] = 0;
            let mut h = vec![0i64; m];
            for (i, f) in f.iter().enumerate().skip(1) {
                for (j, g) in g.iter().enumerate().skip(1) {
                    let k = gcd(i as _, j as _) as usize;
                    if k < m {
                        h[k] += f * g;
                    }
                }
            }
            let i = GcdConvolve::<AddMulOperation<i64>>::convolve(f, g);
            assert_eq!(&h[1..], &i[1..]);
        }
    }
}
