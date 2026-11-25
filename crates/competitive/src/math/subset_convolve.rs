use super::{BitwisexorConvolve, ConvolveSteps, Invertible, Ring, advise_huge_pages};
#[cfg(target_arch = "x86_64")]
use super::{SimdBackend, simd_backend};
use std::{borrow::Cow, marker::PhantomData};

pub struct SubsetConvolve<M> {
    _marker: PhantomData<fn() -> M>,
}

impl<R> SubsetConvolve<R>
where
    R: Ring<T: PartialEq, Additive: Invertible>,
{
    /// Sums products of block weights over the partitions of each subset.
    /// The input has power-of-two length and zero constant term over a commutative ring.
    pub fn exp(weights: &[R::T]) -> Vec<R::T> {
        assert!(weights.len().is_power_of_two() && weights[0] == R::zero());
        let mut result = vec![R::one()];
        while result.len() < weights.len() {
            let half = result.len();
            let upper =
                Self::convolve_impl(result.as_slice().into(), (&weights[half..2 * half]).into());
            result.extend(upper);
        }
        result
    }

    fn ranked(t: Vec<R::T>, len: usize) -> (Vec<R::T>, usize) {
        let width = len.trailing_zeros() as usize + 1;
        let mut ranked = Vec::with_capacity(len * width);
        advise_huge_pages(&mut ranked);
        ranked.resize(len * width, R::zero());
        for (i, value) in t.into_iter().enumerate() {
            ranked[i * width + i.count_ones() as usize] = value;
        }
        (ranked, width)
    }

    fn diagonal(ranked: Vec<R::T>, width: usize) -> Vec<R::T> {
        ranked
            .chunks_exact(width)
            .enumerate()
            .map(|(i, row)| row[i.count_ones() as usize].clone())
            .collect()
    }

    #[inline]
    fn multiply_row(
        x: &[R::T],
        y: &[R::T],
        right: &mut [R::T],
        output: &mut [R::T],
        rank: usize,
    ) -> usize {
        for (right, y) in right[..=rank].iter_mut().zip(y[..=rank].iter().rev()) {
            right.clone_from(y);
        }
        let end = (rank * 2).min(x.len() - 1);
        for (degree, output) in output.iter_mut().enumerate().take(end + 1).skip(rank) {
            let first = degree - rank;
            *output = R::dot_product(&x[first..=rank], &right[..=rank - first]);
        }
        end
    }

    fn convolve_impl(a: Cow<'_, [R::T]>, b: Cow<'_, [R::T]>) -> Vec<R::T> {
        const BATCH: usize = 128;
        assert_eq!(a.len(), b.len());
        let len = a.len();
        if len >= 8 * BATCH
            && let Some(inverse_two) = R::inverse_two()
        {
            assert!(len.is_power_of_two());
            crate::avx_helper!(@versions avx512, avx2; <R>(
                a: &[R::T], b: &[R::T], inverse_two: R::T,
            ) -> Vec<R::T> where R: Ring<T: PartialEq, Additive: Invertible> {
                SubsetConvolve::<R>::ranked_xor::<BATCH>(a, b, inverse_two)
            });
            // SAFETY: the dispatcher checks the features used by each kernel.
            return crate::avx_helper!(@dispatch simd_backend, SimdBackend;
                unsafe { avx512::<R>(&a, &b, inverse_two) },
                unsafe { avx2::<R>(&a, &b, inverse_two) },
                Self::ranked_xor::<BATCH>(&a, &b, inverse_two)
            );
        }
        let same = a == b;
        let (mut x, width) = Self::ranked(a.into_owned(), len);
        let (mut y, _) = if same {
            (x.clone(), width)
        } else {
            Self::ranked(b.into_owned(), len)
        };
        let mut right = vec![R::zero(); width];
        let mut output = vec![R::zero(); width];
        for i in 0..len {
            for bit in (0..(i | len).trailing_zeros() as usize).rev() {
                let half = width << bit;
                let start = i * width;
                let (lower, upper) = x[start..start + half * 2].split_at_mut(half);
                for (upper, lower) in upper.iter_mut().zip(lower) {
                    R::add_assign(upper, lower);
                }
                let (lower, upper) = y[start..start + half * 2].split_at_mut(half);
                for (upper, lower) in upper.iter_mut().zip(lower) {
                    R::add_assign(upper, lower);
                }
            }

            let rank = i.count_ones() as usize;
            let start = i * width;
            let x_row = &x[start..start + width];
            let y_row = &y[start..start + width];
            output.fill(R::zero());
            Self::multiply_row(x_row, y_row, &mut right, &mut output, rank);
            x[start..start + width].clone_from_slice(&output);

            for bit in 0..i.trailing_ones() as usize {
                let end = (i + 1) * width;
                let half = width << bit;
                let (lower, upper) = x[end - half * 2..end].split_at_mut(half);
                for (upper, lower) in upper.iter_mut().zip(lower) {
                    R::sub_assign(upper, lower);
                }
            }
        }
        Self::diagonal(x, width)
    }

    #[inline(always)]
    fn ranked_xor<const W: usize>(a: &[R::T], b: &[R::T], inverse_two: R::T) -> Vec<R::T> {
        let n = a.len().ilog2() as usize;
        let width = n - 1;
        let ranks = std::array::from_fn(|i| i.count_ones() as usize);
        let zero = std::array::from_fn(|_| R::zero());
        let mut x = vec![zero.clone(); a.len() / (8 * W) * width];
        let mut y = x.clone();
        let mut local = vec![zero.clone(); W.ilog2() as usize + 4];
        let mut result = vec![R::zero(); a.len()];

        // |A| + |B| - |A xor B| = 2 |A & B|. The rank sum also fixes the parity
        // of A xor B, so its lowest bit can be omitted from the Walsh transform.
        for frequency in 0..4 {
            local.fill(zero.clone());
            Self::project_ranks::<W, false>(a, &mut x, &mut local, frequency, width, &ranks);
            Self::project_ranks::<W, true>(b, &mut y, &mut local, frequency, width, &ranks);
            for (x, y) in x.chunks_exact_mut(width).zip(y.chunks_exact(width)) {
                for k in (0..width).rev() {
                    x[k] = R::dot_product_batch(&x[..=k], &y[width - 1 - k..]);
                }
            }
            BitwisexorConvolve::<R::Additive>::hadamard_transform_rows(
                x.as_flattened_mut(),
                width * W,
            );
            for (middle, (block, output)) in x
                .chunks_exact(width)
                .zip(result.chunks_exact_mut(8 * W))
                .enumerate()
            {
                let base = middle.count_ones() as usize;
                // Omitting both constant terms leaves product ranks 2 through n.
                for (k, row) in local.iter_mut().enumerate() {
                    if base + k < 2 {
                        row.fill(R::zero());
                    } else {
                        row.clone_from(&block[base + k - 2]);
                        BitwisexorConvolve::<R::Additive>::hadamard_transform(row);
                    }
                }
                for (j, (out, &rank)) in output
                    .as_chunks_mut::<8>()
                    .0
                    .iter_mut()
                    .zip(&ranks)
                    .enumerate()
                {
                    let low_ranks = [0, 1, 1, 2, 1, 2, 2, 3];
                    for ((i, value), low_rank) in out.iter_mut().enumerate().zip(low_ranks) {
                        let term = &local[rank + low_rank][j];
                        if frequency == 0 {
                            value.clone_from(term);
                        } else if [false, true, true, false][frequency & (i >> 1)] {
                            R::sub_assign(value, term);
                        } else {
                            R::add_assign(value, term);
                        }
                    }
                }
            }
        }
        let mut scale = R::one();
        for _ in 1..n {
            R::mul_assign(&mut scale, &inverse_two);
        }
        result[0] = R::mul(&a[0], &b[0]);
        for i in 1..a.len() {
            result[i] = R::dot_product(
                &[result[i].clone(), a[0].clone(), a[i].clone()],
                &[scale.clone(), b[i].clone(), b[0].clone()],
            );
        }
        result
    }

    #[inline(always)]
    fn project_ranks<const W: usize, const REVERSE: bool>(
        input: &[R::T],
        transformed: &mut [[R::T; W]],
        local: &mut [[R::T; W]],
        frequency: usize,
        width: usize,
        ranks: &[usize; W],
    ) {
        let first = |x: &R::T, y: &R::T| {
            if frequency & 1 == 0 {
                R::add(x, y)
            } else {
                R::sub(x, y)
            }
        };
        let second = |x: &R::T, y: &R::T| {
            if frequency & 2 == 0 {
                R::add(x, y)
            } else {
                R::sub(x, y)
            }
        };
        let zero = std::array::from_fn(|_| R::zero());
        for (middle, (src, block)) in input
            .chunks_exact(8 * W)
            .zip(transformed.chunks_exact_mut(width))
            .enumerate()
        {
            for (j, (s, &k)) in src.as_chunks::<8>().0.iter().zip(ranks).enumerate() {
                local[k][j].clone_from(&s[0]);
                local[k + 1][j] = second(&first(&s[1], &s[2]), &s[4]);
                let low = if frequency & 1 == 0 {
                    s[3].clone()
                } else {
                    R::neg(&s[3])
                };
                local[k + 2][j] = second(&low, &first(&s[5], &s[6]));
                local[k + 3][j] = if frequency == 1 || frequency == 2 {
                    R::neg(&s[7])
                } else {
                    s[7].clone()
                };
            }
            let base = middle.count_ones() as usize;
            let end = (base + local.len() - 1).min(width);
            block.fill(zero.clone());
            for rank in base.max(1)..=end {
                let source = &local[rank - base];
                let index = if REVERSE { width - rank } else { rank - 1 };
                block[index].clone_from(source);
                BitwisexorConvolve::<R::Additive>::hadamard_transform(&mut block[index]);
            }
        }
        BitwisexorConvolve::<R::Additive>::hadamard_transform_rows(
            transformed.as_flattened_mut(),
            width * W,
        );
    }
}

impl<R> ConvolveSteps for SubsetConvolve<R>
where
    R: Ring<T: PartialEq, Additive: Invertible>,
{
    type T = Vec<R::T>;
    type F = (Vec<R::T>, usize);

    fn length(t: &Self::T) -> usize {
        t.len()
    }

    fn transform(t: Self::T, len: usize) -> Self::F {
        let (mut f, width) = Self::ranked(t, len);
        let k = width - 1;
        for bit in 0..k {
            let half = 1 << bit;
            for base in (0..len).step_by(half * 2) {
                for lower in base..base + half {
                    let upper = lower + half;
                    let ranks = lower.count_ones() as usize + 1;
                    let (lower_rows, upper_rows) = f.split_at_mut(upper * width);
                    let lower_row = &lower_rows[lower * width..lower * width + ranks];
                    let upper_row = &mut upper_rows[..ranks];
                    for (upper, lower) in upper_row.iter_mut().zip(lower_row) {
                        R::add_assign(upper, lower);
                    }
                }
            }
        }
        (f, width)
    }

    fn inverse_transform((mut f, width): Self::F, len: usize) -> Self::T {
        let k = width - 1;
        for bit in 0..k {
            let half = 1 << bit;
            for base in (0..len).step_by(half * 2) {
                for lower in base..base + half {
                    let upper = lower + half;
                    let rank = lower.count_ones() as usize;
                    let (lower_rows, upper_rows) = f.split_at_mut(upper * width);
                    let lower_row = &lower_rows[lower * width + rank..lower * width + width];
                    let upper_row = &mut upper_rows[rank..width];
                    for (upper, lower) in upper_row.iter_mut().zip(lower_row) {
                        R::sub_assign(upper, lower);
                    }
                }
            }
        }
        Self::diagonal(f, width)
    }

    fn multiply(f: &mut Self::F, g: &Self::F) {
        let (f, width) = f;
        let (g, _) = g;
        let mut right = vec![R::zero(); *width];
        let mut output = vec![R::zero(); *width];
        for (i, f) in f.chunks_exact_mut(*width).enumerate() {
            let rank = i.count_ones() as usize;
            let g = &g[i * *width..(i + 1) * *width];
            let end = Self::multiply_row(f, g, &mut right, &mut output, rank);
            f[rank..=end].clone_from_slice(&output[rank..=end]);
        }
    }

    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        Self::convolve_impl(a.into(), b.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AddMulOperation,
        rand,
        tools::{Xorshift, testutil::sequence_cases},
    };

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
            let mut transformed = SubsetConvolve::<AddMulOperation<i64>>::transform(f.clone(), n);
            let other = SubsetConvolve::<AddMulOperation<i64>>::transform(g.clone(), n);
            SubsetConvolve::<AddMulOperation<i64>>::multiply(&mut transformed, &other);
            let j = SubsetConvolve::<AddMulOperation<i64>>::inverse_transform(transformed, n);
            let i = SubsetConvolve::<AddMulOperation<i64>>::convolve(f, g);
            assert_eq!(h, i);
            assert_eq!(h, j);
        }
    }

    #[test]
    fn set_partitions() {
        use crate::num::{One, Zero, mint_basic::MInt998244353 as M};

        let mut rng = Xorshift::default();
        for k in 0..=12 {
            let len = 1 << k;
            for weights in sequence_cases(&mut rng, -2i64..=2, len..=len).take(16) {
                let mut weights: Vec<_> = weights.into_iter().map(M::from).collect();
                weights[0] = M::zero();
                let mut expected = vec![M::zero(); len];
                expected[0] = M::one();
                for mask in 1..len {
                    let rest = mask & (mask - 1);
                    let mut subset = rest;
                    loop {
                        let term = expected[subset] * weights[mask ^ subset];
                        expected[mask] += term;
                        if subset == 0 {
                            break;
                        }
                        subset = (subset - 1) & rest;
                    }
                }
                assert_eq!(
                    SubsetConvolve::<AddMulOperation<_>>::exp(&weights),
                    expected
                );
            }
        }
    }

    #[test]
    fn modular_subset_products() {
        use crate::num::{MInt, MIntConvert, MIntDotProduct, mint_basic::DynModuloU32, montgomery};

        fn check<M>(rng: &mut Xorshift)
        where
            M: MIntDotProduct<Inner = u32> + MIntConvert<u32>,
        {
            let modulus = M::get_mod();
            for bits in 0..=11 {
                let n = 1 << bits;
                let mut coefficient = || MInt::<M>::new(rng.random(..modulus));
                let a: Vec<_> = (0..n).map(|_| coefficient()).collect();
                let b: Vec<_> = (0..n).map(|_| coefficient()).collect();
                let expected: Vec<_> = (0..n)
                    .map(|mask| {
                        let mut sub = mask;
                        let mut sum = 0u128;
                        loop {
                            sum += u128::from(a[sub].inner()) * u128::from(b[mask ^ sub].inner());
                            if sub == 0 {
                                break;
                            }
                            sub = (sub - 1) & mask;
                        }
                        MInt::<M>::new((sum % u128::from(modulus)) as u32)
                    })
                    .collect();
                assert_eq!(
                    SubsetConvolve::<AddMulOperation<_>>::convolve(a, b),
                    expected
                );
            }
        }
        let mut rng = Xorshift::default();
        for modulus in [1, 2, 9, 1 << 31, u32::MAX] {
            DynModuloU32::set_mod(modulus);
            check::<DynModuloU32>(&mut rng);
        }
        DynModuloU32::set_mod(1_000_000_007);
        check::<montgomery::Modulo998244353>(&mut rng);
    }
}
