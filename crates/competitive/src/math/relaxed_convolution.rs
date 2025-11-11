use super::{ConvolveSteps, Zero};
use std::{
    fmt::Debug,
    iter::zip,
    marker::PhantomData,
    ops::{AddAssign, Index},
    slice::SliceIndex,
};

pub struct RelaxedConvolution<T, C>
where
    C: ConvolveSteps<T = Vec<T>>,
{
    a: Vec<T>,
    b: Vec<T>,
    c: Vec<T>,
    _marker: PhantomData<fn() -> C>,
}

impl<T: Debug, C> Debug for RelaxedConvolution<T, C>
where
    C: ConvolveSteps<T = Vec<T>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelaxedConvolution")
            .field("a", &self.a)
            .field("b", &self.b)
            .field("c", &self.c)
            .finish()
    }
}

impl<T, C> Default for RelaxedConvolution<T, C>
where
    C: ConvolveSteps<T = Vec<T>>,
{
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
            c: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<T, C> RelaxedConvolution<T, C>
where
    T: Clone + Zero + AddAssign<T>,
    C: ConvolveSteps<T = Vec<T>>,
{
    pub fn push(&mut self, x: T, y: T) {
        let q = self.a.len();
        self.a.push(x);
        self.b.push(y);
        self.c.push(T::zero());
        if q != 0 {
            self.c.push(T::zero());
        }
        let k = (q + 2).trailing_zeros();
        let mut s = 0;
        for k in 0..k + 1 - (1 << k == q + 2) as u32 {
            let size = 1 << k;
            self.calc_block(s, q + 1 - size, size);
            if q + 1 - size != s {
                self.calc_block(q + 1 - size, s, size);
            }
            s += size;
        }
    }

    fn calc_block(&mut self, la: usize, lb: usize, size: usize) {
        let a = self.a[la..la + size].to_vec();
        let b = self.b[lb..lb + size].to_vec();
        let c = C::convolve(a, b);
        for (c, d) in zip(c, &mut self.c[la + lb..]) {
            *d += c;
        }
    }
}

impl<T, C, I> Index<I> for RelaxedConvolution<T, C>
where
    C: ConvolveSteps<T = Vec<T>>,
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        self.c.index(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::Convolve998244353, num::montgomery::MInt998244353, tools::Xorshift};

    #[test]
    fn test_relaxed_convolution() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.random(1..100);
            let a: Vec<MInt998244353> = (0..n).map(|_| rng.random(..)).take(n).collect();
            let b: Vec<MInt998244353> = (0..n).map(|_| rng.random(..)).take(n).collect();
            let mut conv = RelaxedConvolution::<MInt998244353, Convolve998244353>::default();
            for (&x, &y) in zip(&a, &b) {
                conv.push(x, y);
            }
            let c = Convolve998244353::convolve(a, b);
            for i in 0..n {
                assert_eq!(conv[i], c[i]);
            }
        }
    }
}
