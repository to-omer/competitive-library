//! fast zeta transform and fast mobius transform
//!
//! Convolution theorem
//! - bitwiseor convolution: subset
//! - bitwiseand convolution: superset
//! - lcm convolution: divisor
//! - gcd convolution: multiple

use crate::algebra::{Group, Monoid};

#[codesnip::entry("SubsetTransform", include("algebra"))]
pub struct SubsetTransform<M: Monoid> {
    monoid: M,
}
#[codesnip::entry("SubsetTransform")]
impl<M: Monoid> SubsetTransform<M> {
    pub fn new(monoid: M) -> Self {
        Self { monoid }
    }
    /// $$g(T) = \sum_{S\subset T}f(S)$$
    pub fn zeta_transform(&self, f: &mut [M::T]) {
        let n = f.len();
        let mut i = 1;
        while i < n {
            for j in 0..n {
                if j & i != 0 {
                    f[j] = self.monoid.operate(&f[j], &f[j ^ i]);
                }
            }
            i <<= 1;
        }
    }
}
#[codesnip::entry("SubsetTransform")]
impl<G: Group> SubsetTransform<G> {
    /// $$f(T) = \sum_{S\subset T}h(S)$$
    pub fn mobius_transform(&self, f: &mut [G::T]) {
        let n = f.len();
        let mut i = 1;
        while i < n {
            for j in 0..n {
                if j & i != 0 {
                    f[j] = self.monoid.rinv_operate(&f[j], &f[j ^ i]);
                }
            }
            i <<= 1;
        }
    }
    /// $$h(U) = \sum_{S\cup T=U}f(S)g(T)$$
    pub fn convolve<M: Monoid<T = G::T>>(
        &self,
        mut f: Vec<G::T>,
        mut g: Vec<G::T>,
        mul: M,
    ) -> Vec<G::T> {
        self.zeta_transform(&mut f);
        self.zeta_transform(&mut g);
        for (a, b) in f.iter_mut().zip(g.iter()) {
            *a = mul.operate(a, b);
        }
        self.mobius_transform(&mut f);
        f
    }
}

#[codesnip::entry("SupersetTransform", include("algebra"))]
pub struct SupersetTransform<M: Monoid> {
    monoid: M,
}
#[codesnip::entry("SupersetTransform")]
impl<M: Monoid> SupersetTransform<M> {
    pub fn new(monoid: M) -> Self {
        Self { monoid }
    }
    /// $$g(T) = \sum_{S\supset T}f(S)$$
    pub fn zeta_transform(&self, f: &mut [M::T]) {
        let n = f.len();
        let mut i = 1;
        while i < n {
            for j in 0..n {
                if j & i == 0 {
                    f[j] = self.monoid.operate(&f[j], &f[j | i]);
                }
            }
            i <<= 1;
        }
    }
}
#[codesnip::entry("SupersetTransform")]
impl<G: Group> SupersetTransform<G> {
    /// $$f(T) = \sum_{S\supset T}h(S)$$
    pub fn mobius_transform(&self, f: &mut [G::T]) {
        let n = f.len();
        let mut i = 1;
        while i < n {
            for j in 0..n {
                if j & i == 0 {
                    f[j] = self.monoid.rinv_operate(&f[j], &f[j | i]);
                }
            }
            i <<= 1;
        }
    }
    /// $$h(U) = \sum_{S\cap T=U}f(S)g(T)$$
    pub fn convolve<M: Monoid<T = G::T>>(
        &self,
        mut f: Vec<G::T>,
        mut g: Vec<G::T>,
        mul: M,
    ) -> Vec<G::T> {
        self.zeta_transform(&mut f);
        self.zeta_transform(&mut g);
        for (a, b) in f.iter_mut().zip(g.iter()) {
            *a = mul.operate(a, b);
        }
        self.mobius_transform(&mut f);
        f
    }
}

#[codesnip::entry("DivisorTransform", include("algebra"))]
pub struct DivisorTransform<M: Monoid> {
    monoid: M,
    primes: Vec<usize>,
}
#[codesnip::entry("DivisorTransform")]
impl<M: Monoid> DivisorTransform<M> {
    pub fn new(monoid: M, primes: Vec<usize>) -> Self {
        Self { monoid, primes }
    }
    /// $$g(m) = \sum_{n \mid m}f(n)$$
    pub fn zeta_transform(&self, f: &mut [M::T]) {
        for &p in self.primes.iter() {
            for (i, j) in (0..f.len()).step_by(p).enumerate() {
                f[j] = self.monoid.operate(&f[j], &f[i]);
            }
        }
    }
}
#[codesnip::entry("DivisorTransform")]
impl<G: Group> DivisorTransform<G> {
    /// $$f(m) = \sum_{n \mid m}h(n)$$
    pub fn mobius_transform(&self, f: &mut [G::T]) {
        for &p in self.primes.iter() {
            for (i, j) in (0..f.len()).step_by(p).enumerate().rev() {
                f[j] = self.monoid.rinv_operate(&f[j], &f[i]);
            }
        }
    }
    /// $$h(k) = \sum_{\mathrm{lcm}(n, m)=k}f(n)g(m)$$
    pub fn convolve<M: Monoid<T = G::T>>(
        &self,
        mut f: Vec<G::T>,
        mut g: Vec<G::T>,
        mul: M,
    ) -> Vec<G::T> {
        self.zeta_transform(&mut f);
        self.zeta_transform(&mut g);
        for (a, b) in f.iter_mut().zip(g.iter()) {
            *a = mul.operate(a, b);
        }
        self.mobius_transform(&mut f);
        f
    }
}

#[codesnip::entry("MultipleTransform", include("algebra"))]
pub struct MultipleTransform<M: Monoid> {
    monoid: M,
    primes: Vec<usize>,
}
#[codesnip::entry("MultipleTransform")]
impl<M: Monoid> MultipleTransform<M> {
    pub fn new(monoid: M, primes: Vec<usize>) -> Self {
        Self { monoid, primes }
    }
    /// $$g(m) = \sum_{m \mid n}f(n)$$
    pub fn zeta_transform(&self, f: &mut [M::T]) {
        for &p in self.primes.iter() {
            for (i, j) in (0..f.len()).step_by(p).enumerate().rev() {
                f[i] = self.monoid.operate(&f[i], &f[j]);
            }
        }
    }
}
#[codesnip::entry("MultipleTransform")]
impl<G: Group> MultipleTransform<G> {
    /// $$f(m) = \sum_{m \mid n}h(n)$$
    pub fn mobius_transform(&self, f: &mut [G::T]) {
        for &p in self.primes.iter() {
            for (i, j) in (0..f.len()).step_by(p).enumerate() {
                f[i] = self.monoid.rinv_operate(&f[i], &f[j]);
            }
        }
    }
    /// $$h(k) = \sum_{\gcd(n, m)=k}f(n)g(m)$$
    pub fn convolve<M: Monoid<T = G::T>>(
        &self,
        mut f: Vec<G::T>,
        mut g: Vec<G::T>,
        mul: M,
    ) -> Vec<G::T> {
        self.zeta_transform(&mut f);
        self.zeta_transform(&mut g);
        for (a, b) in f.iter_mut().zip(g.iter()) {
            *a = mul.operate(a, b);
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
        math::{gcd, lcm, primes},
        rand,
        tools::Xorshift,
    };

    const N: usize = 1 << 12;
    const M: usize = 3000;
    const A: i64 = 100_000;

    #[test]
    fn test_subset_transform() {
        let mut rng = Xorshift::time();
        let subset = SubsetTransform::new(AdditiveOperation::new());

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
        subset.zeta_transform(&mut f);
        assert_eq!(f, g);
        subset.mobius_transform(&mut f);
        assert_eq!(f, h);

        rand!(rng, f: [-A..A; N], g: [-A..A; N]);
        let mut h = vec![0i64; N];
        for i in 0..N {
            for j in 0..N {
                h[i | j] += f[i] * g[j];
            }
        }
        let i = subset.convolve(f, g, MultiplicativeOperation::new());
        assert_eq!(h, i);
    }

    #[test]
    fn test_superset_transform() {
        let mut rng = Xorshift::time();
        let superset = SupersetTransform::new(AdditiveOperation::new());

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
        superset.zeta_transform(&mut f);
        assert_eq!(f, g);
        superset.mobius_transform(&mut f);
        assert_eq!(f, h);

        rand!(rng, f: [-A..A; N], g: [-A..A; N]);
        let mut h = vec![0i64; N];
        for i in 0..N {
            for j in 0..N {
                h[i & j] += f[i] * g[j];
            }
        }
        let i = superset.convolve(f, g, MultiplicativeOperation::new());
        assert_eq!(h, i);
    }

    #[test]
    fn test_divisor_transform() {
        let mut rng = Xorshift::time();
        let divisor = DivisorTransform::new(AdditiveOperation::new(), primes(M));

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
        let i = divisor.convolve(f, g, MultiplicativeOperation::new());
        assert_eq!(&h[1..], &i[1..]);
    }

    #[test]
    fn test_multiple_transform() {
        let mut rng = Xorshift::time();
        let multiple = MultipleTransform::new(AdditiveOperation::new(), primes(M));

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
        let i = multiple.convolve(f, g, MultiplicativeOperation::new());
        assert_eq!(&h[1..], &i[1..]);
    }
}
