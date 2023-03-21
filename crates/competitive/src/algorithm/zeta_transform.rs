//! fast zeta transform and fast mobius transform
//!
//! Convolution theorem
//! - bitwiseor convolution: subset
//! - bitwiseand convolution: superset
//! - lcm convolution: divisor
//! - gcd convolution: multiple

use super::{Group, Monoid};
use std::marker::PhantomData;

pub struct SubsetTransform<M>
where
    M: Monoid,
{
    _marker: PhantomData<fn() -> M>,
}

impl<M> SubsetTransform<M>
where
    M: Monoid,
{
    /// $$g(T) = \sum_{S\subset T}f(S)$$
    pub fn zeta_transform(f: &mut [M::T]) {
        let n = f.len();
        let mut i = 1;
        while i < n {
            for j in 0..n {
                if j & i != 0 {
                    f[j] = M::operate(&f[j], &f[j ^ i]);
                }
            }
            i <<= 1;
        }
    }
}

impl<G> SubsetTransform<G>
where
    G: Group,
{
    /// $$f(T) = \sum_{S\subset T}h(S)$$
    pub fn mobius_transform(f: &mut [G::T]) {
        let n = f.len();
        let mut i = 1;
        while i < n {
            for j in 0..n {
                if j & i != 0 {
                    f[j] = G::rinv_operate(&f[j], &f[j ^ i]);
                }
            }
            i <<= 1;
        }
    }
    /// $$h(U) = \sum_{S\cup T=U}f(S)g(T)$$
    pub fn convolve<M>(mut f: Vec<G::T>, mut g: Vec<G::T>) -> Vec<G::T>
    where
        M: Monoid<T = G::T>,
    {
        Self::zeta_transform(&mut f);
        Self::zeta_transform(&mut g);
        for (a, b) in f.iter_mut().zip(g.iter()) {
            *a = M::operate(a, b);
        }
        Self::mobius_transform(&mut f);
        f
    }
}

pub struct SupersetTransform<M>
where
    M: Monoid,
{
    _marker: PhantomData<fn() -> M>,
}

impl<M> SupersetTransform<M>
where
    M: Monoid,
{
    /// $$g(T) = \sum_{S\supset T}f(S)$$
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

impl<G> SupersetTransform<G>
where
    G: Group,
{
    /// $$f(T) = \sum_{S\supset T}h(S)$$
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
    /// $$h(U) = \sum_{S\cap T=U}f(S)g(T)$$
    pub fn convolve<M>(mut f: Vec<G::T>, mut g: Vec<G::T>) -> Vec<G::T>
    where
        M: Monoid<T = G::T>,
    {
        Self::zeta_transform(&mut f);
        Self::zeta_transform(&mut g);
        for (a, b) in f.iter_mut().zip(g.iter()) {
            *a = M::operate(a, b);
        }
        Self::mobius_transform(&mut f);
        f
    }
}

pub struct DivisorTransform<'p, M>
where
    M: Monoid,
{
    primes: &'p [u64],
    _marker: PhantomData<fn() -> M>,
}

impl<'p, M> DivisorTransform<'p, M>
where
    M: Monoid,
{
    pub fn new_with_primes(primes: &'p [u64]) -> Self {
        Self {
            primes,
            _marker: PhantomData,
        }
    }
    fn primes_iter(&self, n: usize) -> impl 'p + Iterator<Item = usize> {
        self.primes
            .iter()
            .map(|&p| p as usize)
            .take_while(move |&p| p < n)
    }
    /// $$g(m) = \sum_{n \mid m}f(n)$$
    pub fn zeta_transform(&self, f: &mut [M::T]) {
        for p in self.primes_iter(f.len()) {
            for (i, j) in (0..f.len()).step_by(p).enumerate() {
                f[j] = M::operate(&f[j], &f[i]);
            }
        }
    }
}

impl<G> DivisorTransform<'_, G>
where
    G: Group,
{
    /// $$f(m) = \sum_{n \mid m}h(n)$$
    pub fn mobius_transform(&self, f: &mut [G::T]) {
        for p in self.primes_iter(f.len()) {
            for (i, j) in (0..f.len()).step_by(p).enumerate().rev() {
                f[j] = G::rinv_operate(&f[j], &f[i]);
            }
        }
    }
    /// $$h(k) = \sum_{\mathrm{lcm}(n, m)=k}f(n)g(m)$$
    pub fn convolve<M>(&self, mut f: Vec<G::T>, mut g: Vec<G::T>) -> Vec<G::T>
    where
        M: Monoid<T = G::T>,
    {
        self.zeta_transform(&mut f);
        self.zeta_transform(&mut g);
        for (a, b) in f.iter_mut().zip(g.iter()) {
            *a = M::operate(a, b);
        }
        self.mobius_transform(&mut f);
        f
    }
}

pub struct MultipleTransform<'p, M>
where
    M: Monoid,
{
    primes: &'p [u64],
    _marker: PhantomData<fn() -> M>,
}

impl<'p, M> MultipleTransform<'p, M>
where
    M: Monoid,
{
    pub fn new_with_primes(primes: &'p [u64]) -> Self {
        Self {
            primes,
            _marker: PhantomData,
        }
    }
    fn primes_iter(&self, n: usize) -> impl 'p + Iterator<Item = usize> {
        self.primes
            .iter()
            .map(|&p| p as usize)
            .take_while(move |&p| p < n)
    }
    /// $$g(m) = \sum_{m \mid n}f(n)$$
    pub fn zeta_transform(&self, f: &mut [M::T]) {
        for p in self.primes_iter(f.len()) {
            for (i, j) in (0..f.len()).step_by(p).enumerate().rev() {
                f[i] = M::operate(&f[i], &f[j]);
            }
        }
    }
}

impl<G> MultipleTransform<'_, G>
where
    G: Group,
{
    /// $$f(m) = \sum_{m \mid n}h(n)$$
    pub fn mobius_transform(&self, f: &mut [G::T]) {
        for p in self.primes_iter(f.len()) {
            for (i, j) in (0..f.len()).step_by(p).enumerate() {
                f[i] = G::rinv_operate(&f[i], &f[j]);
            }
        }
    }
    /// $$h(k) = \sum_{\gcd(n, m)=k}f(n)g(m)$$
    pub fn convolve<M: Monoid<T = G::T>>(&self, mut f: Vec<G::T>, mut g: Vec<G::T>) -> Vec<G::T> {
        self.zeta_transform(&mut f);
        self.zeta_transform(&mut g);
        for (a, b) in f.iter_mut().zip(g.iter()) {
            *a = M::operate(a, b);
        }
        self.mobius_transform(&mut f);
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, MultiplicativeOperation},
        math::{gcd, lcm, PrimeList},
        rand,
        tools::Xorshift,
    };

    const N: usize = 1 << 12;
    const M: usize = 3000;
    const A: i64 = 100_000;

    #[test]
    fn test_subset_transform() {
        let mut rng = Xorshift::new();
        type Subset = SubsetTransform<AdditiveOperation<i64>>;

        rand!(rng, mut f: [-A..A; N]);
        let mut g = vec![0i64; N];
        let h = f.clone();
        for (s, f) in f.iter().enumerate() {
            for (t, g) in g.iter_mut().enumerate() {
                if s | t == t {
                    *g += f;
                }
            }
        }
        Subset::zeta_transform(&mut f);
        assert_eq!(f, g);
        Subset::mobius_transform(&mut f);
        assert_eq!(f, h);

        rand!(rng, f: [-A..A; N], g: [-A..A; N]);
        let mut h = vec![0i64; N];
        for i in 0..N {
            for j in 0..N {
                h[i | j] += f[i] * g[j];
            }
        }
        let i = Subset::convolve::<MultiplicativeOperation<_>>(f, g);
        assert_eq!(h, i);
    }

    #[test]
    fn test_superset_transform() {
        let mut rng = Xorshift::new();
        type Superset = SupersetTransform<AdditiveOperation<i64>>;

        rand!(rng, mut f: [-A..A; N]);
        let mut g = vec![0i64; N];
        let h = f.clone();
        for (s, f) in f.iter().enumerate() {
            for (t, g) in g.iter_mut().enumerate() {
                if s | t == s {
                    *g += f;
                }
            }
        }
        Superset::zeta_transform(&mut f);
        assert_eq!(f, g);
        Superset::mobius_transform(&mut f);
        assert_eq!(f, h);

        rand!(rng, f: [-A..A; N], g: [-A..A; N]);
        let mut h = vec![0i64; N];
        for i in 0..N {
            for j in 0..N {
                h[i & j] += f[i] * g[j];
            }
        }
        let i = Superset::convolve::<MultiplicativeOperation<_>>(f, g);
        assert_eq!(h, i);
    }

    #[test]
    fn test_divisor_transform() {
        let mut rng = Xorshift::new();
        let primes = PrimeList::new(M as u64);
        let divisor = DivisorTransform::<AdditiveOperation<i64>>::new_with_primes(primes.primes());

        rand!(rng, mut f: [-A..A; M]);
        f[0] = 0;
        let mut g = vec![0i64; M];
        let h = f.clone();
        for (s, f) in f.iter().enumerate().skip(1) {
            for (t, g) in g.iter_mut().enumerate().skip(1) {
                if t % s == 0 {
                    *g += f;
                }
            }
        }
        divisor.zeta_transform(&mut f);
        assert_eq!(&f[1..], &g[1..]);
        divisor.mobius_transform(&mut f);
        assert_eq!(&f[1..], &h[1..]);

        rand!(rng, mut f: [-A..A; M], mut g: [-A..A; M]);
        f[0] = 0;
        g[0] = 0;
        let mut h = vec![0i64; M];
        for (i, f) in f.iter().enumerate().skip(1) {
            for (j, g) in g.iter().enumerate().skip(1) {
                let k = lcm(i as _, j as _) as usize;
                if k < M {
                    h[k] += f * g;
                }
            }
        }
        let i = divisor.convolve::<MultiplicativeOperation<_>>(f, g);
        assert_eq!(&h[1..], &i[1..]);
    }

    #[test]
    fn test_multiple_transform() {
        let mut rng = Xorshift::new();
        let primes = PrimeList::new(M as u64);
        let multiple = MultipleTransform::<AdditiveOperation<_>>::new_with_primes(primes.primes());

        rand!(rng, mut f: [-A..A; M]);
        f[0] = 0;
        let mut g = vec![0i64; M];
        let h = f.clone();
        for (s, f) in f.iter().enumerate().skip(1) {
            for (t, g) in g.iter_mut().enumerate().skip(1) {
                if s % t == 0 {
                    *g += f;
                }
            }
        }
        multiple.zeta_transform(&mut f);
        assert_eq!(&f[1..], &g[1..]);
        multiple.mobius_transform(&mut f);
        assert_eq!(&f[1..], &h[1..]);

        rand!(rng, mut f: [-A..A; M], mut g: [-A..A; M]);
        f[0] = 0;
        g[0] = 0;
        let mut h = vec![0i64; M];
        for i in 1..M {
            for j in 1..M {
                h[(gcd(i as _, j as _) as usize)] += f[i] * g[j];
            }
        }
        let i = multiple.convolve::<MultiplicativeOperation<_>>(f, g);
        assert_eq!(&h[1..], &i[1..]);
    }
}
