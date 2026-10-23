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
        let k = f.len().trailing_zeros() as usize;
        assert!(f.len() == 1 << k);
        for i in (0..k).rev() {
            for chunk in f.chunks_exact_mut(2 << i) {
                let (left, right) = chunk.split_at_mut(1 << i);
                for (y, x) in left.iter_mut().zip(right.iter_mut()) {
                    *x = G::rinv_operate(x, y);
                }
            }
        }
    }
}

impl<R> ConvolveSteps for BitwiseorConvolve<R>
where
    R: Ring<T: PartialEq, Additive: Invertible>,
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

pub struct OnlineSubsetZetaTransform<M>
where
    M: Monoid,
{
    data: Vec<M::T>,
}

impl<M> OnlineSubsetZetaTransform<M>
where
    M: Monoid,
{
    pub fn new(size: usize) -> Self {
        Self {
            data: Vec::with_capacity(size),
        }
    }

    pub fn push(&mut self, x: M::T) {
        let i = self.data.len();
        self.data.push(x);
        let mut k = 0;
        while i >> k & 1 == 1 {
            let size = 1 << k;
            let chunk = &mut self.data[i - (size * 2 - 1)..];
            let (x, y) = chunk.split_at_mut(size);
            for (x, y) in x.iter().zip(y.iter_mut()) {
                *y = M::operate(y, x);
            }
            k += 1;
        }
    }

    pub fn get(&self, index: usize) -> M::T {
        let mut cur = self.data.len();
        let mut s = M::unit();
        while cur != 0 {
            let d = cur.trailing_zeros() as usize;
            let base = cur ^ (1 << d);
            if (base & !index) >> d == 0 {
                let mask = (1 << d) - 1;
                s = M::operate(&s, &self.data[base ^ ((index ^ base) & mask)]);
            }
            cur = base;
        }
        s
    }
}

pub struct OnlineSubsetMobiusTransform<M>
where
    M: Group,
{
    data: Vec<M::T>,
}

impl<M> OnlineSubsetMobiusTransform<M>
where
    M: Group,
{
    pub fn new(size: usize) -> Self {
        Self {
            data: Vec::with_capacity(size),
        }
    }

    pub fn push(&mut self, x: M::T) {
        let i = self.data.len();
        self.data.push(x);
        let mut k = 0;
        while i >> k & 1 == 1 {
            let size = 1 << k;
            let chunk = &mut self.data[i - (size * 2 - 1)..];
            let (x, y) = chunk.split_at_mut(size);
            for (x, y) in x.iter().zip(y.iter_mut()) {
                *y = M::rinv_operate(y, x);
            }
            k += 1;
        }
    }

    pub fn get(&self, index: usize) -> M::T {
        let mut cur = self.data.len();
        let mut s = M::unit();
        while cur != 0 {
            let d = cur.trailing_zeros() as usize;
            let base = cur ^ (1 << d);
            if (base & !index) >> d == 0 {
                let mask = (1 << d) - 1;
                let j = base ^ ((index ^ base) & mask);
                if ((index ^ base) >> d).count_ones() & 1 == 1 {
                    s = M::rinv_operate(&s, &self.data[j]);
                } else {
                    s = M::operate(&s, &self.data[j]);
                }
            }
            cur = base;
        }
        s
    }
}

#[cfg(kani)]
mod verification {
    use super::*;
    use crate::algebra::{AdditiveOperation, BitXorOperation, FixedPermutationOperation};
    use std::num::Wrapping;

    macro_rules! proof_transform {
        ($name:ident, $n:literal) => {
            #[kani::proof]
            #[kani::unwind(18)]
            fn $name() {
                let values: [u64; $n] = kani::any();
                let query: u8 = kani::any();
                kani::assume(query < $n);
                let mut actual = values;
                BitwiseorConvolve::<BitXorOperation<u64>>::zeta_transform(&mut actual);
                let mut expected = 0u64;
                for (mask, &value) in values.iter().enumerate() {
                    if mask & query as usize == mask {
                        expected ^= value;
                    }
                }
                kani::assert(
                    actual[query as usize] == expected,
                    concat!(
                        "For every ",
                        $n,
                        "-element u64 array, subset zeta transformation XORs exactly the submasks of each index",
                    ),
                );
                BitwiseorConvolve::<BitXorOperation<u64>>::mobius_transform(&mut actual);
                kani::assert(
                    actual[query as usize] == values[query as usize],
                    "Subset Mobius inversion over XOR restores every original u64 value",
                );
                kani::cover!(query == 0);
                kani::cover!(query == $n - 1 && values[0] == u64::MIN && values[$n - 1] == u64::MAX);

                // XOR is self-inverse; a weighted basis vector distinguishes addition from subtraction.
                let mut basis = [Wrapping(0u8); $n];
                basis[query as usize] = Wrapping(kani::any());
                let expected = basis;
                BitwiseorConvolve::<AdditiveOperation<Wrapping<u8>>>::zeta_transform(&mut basis);
                BitwiseorConvolve::<AdditiveOperation<Wrapping<u8>>>::mobius_transform(&mut basis);
                kani::assert(
                    basis == expected,
                    concat!(
                        "For every position and u8 value in a ",
                        $n,
                        "-element array with at most one nonzero entry, subset zeta and Mobius transforms over addition modulo 256 restore the entire array",
                    ),
                );
            }
        };
    }

    proof_transform!(proof_transform_eight, 8);
    proof_transform!(proof_transform_sixteen, 16);

    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_noncommutative_transform() {
        let values: [[u8; 3]; 4] = kani::any();
        for p in &values {
            kani::assume(p[0] < 3 && p[1] < 3 && p[2] < 3);
            kani::assume(p[0] != p[1] && p[0] != p[2] && p[1] != p[2]);
        }
        let values = values.map(|p| p.map(usize::from));
        let (query, len): (u8, u8) = kani::any();
        kani::assume(query < 4 && len <= 4);
        let query = query as usize;
        let len = len as usize;
        let mut actual = values;
        BitwiseorConvolve::<FixedPermutationOperation<3>>::zeta_transform(&mut actual);
        let mut expected = [0, 1, 2];
        for mask in (0..values.len()).rev() {
            if mask & query == mask {
                let p = values[mask];
                expected = [expected[p[0]], expected[p[1]], expected[p[2]]];
            }
        }
        for label in 0..3 {
            kani::assert(
                actual[query][label] == expected[label],
                "For any sequence of four permutations of three labels, subset zeta combines the matching masks in descending order",
            );
        }
        BitwiseorConvolve::<FixedPermutationOperation<3>>::mobius_transform(&mut actual);
        for index in 0..values.len() {
            for label in 0..3 {
                kani::assert(
                    actual[index][label] == values[index][label],
                    "Subset Mobius inversion restores all four permutations without assuming commutativity",
                );
            }
        }
        let mut online =
            OnlineSubsetZetaTransform::<FixedPermutationOperation<3>>::new(values.len());
        for &value in &values[..len] {
            online.push(value);
        }
        let mut expected = [0, 1, 2];
        for mask in (0..len).rev() {
            if mask & query == mask {
                let p = values[mask];
                expected = [expected[p[0]], expected[p[1]], expected[p[2]]];
            }
        }
        let actual = online.get(query);
        for label in 0..3 {
            kani::assert(
                actual[label] == expected[label],
                "For every prefix of four arbitrary permutations, online subset zeta preserves the same noncommutative order as the static transform",
            );
        }
        kani::cover!(len == 3 && query == 3 && values[0][values[1][0]] != values[1][values[0][0]]);
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
        let mut rng = Xorshift::default();

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

    #[test]
    fn test_online_subset_zeta_transform() {
        let mut rng = Xorshift::default();
        for k in 0..12 {
            let n = 1 << k;
            rand!(rng, f: [-A..A; n]);
            let mut ost = OnlineSubsetZetaTransform::<AdditiveOperation<i64>>::new(0);
            for i in 0..n {
                ost.push(f[i]);
                let mut g: Vec<_> = f[..=i]
                    .iter()
                    .cloned()
                    .chain(std::iter::repeat_n(0, n - i - 1))
                    .collect();
                BitwiseorConvolve::<AdditiveOperation<i64>>::zeta_transform(&mut g);
                for (j, &g) in g.iter().enumerate() {
                    assert_eq!(ost.get(j), g);
                }
            }
        }
    }

    #[test]
    fn test_online_subset_mobius_transform() {
        let mut rng = Xorshift::default();
        for k in 0..12 {
            let n = 1 << k;
            rand!(rng, f: [-A..A; n]);
            let mut ost = OnlineSubsetMobiusTransform::<AdditiveOperation<i64>>::new(0);
            for i in 0..n {
                ost.push(f[i]);
                let mut g: Vec<_> = f[..=i]
                    .iter()
                    .cloned()
                    .chain(std::iter::repeat_n(0, n - i - 1))
                    .collect();
                BitwiseorConvolve::<AdditiveOperation<i64>>::mobius_transform(&mut g);
                for (j, &g) in g.iter().enumerate() {
                    assert_eq!(ost.get(j), g);
                }
            }
        }
    }
}
