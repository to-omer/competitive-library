use super::{ConvolveSteps, Invertible, Ring};
use std::marker::PhantomData;

pub struct SubsetConvolve<M> {
    _marker: PhantomData<fn() -> M>,
}

impl<R> SubsetConvolve<R>
where
    R: Ring<T: PartialEq, Additive: Invertible>,
{
    fn ranked(t: Vec<R::T>, len: usize) -> (Vec<R::T>, usize) {
        let width = len.trailing_zeros() as usize + 1;
        let mut ranked = vec![R::zero(); len * width];
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
        assert_eq!(a.len(), b.len());
        let len = a.len();
        let same = a == b;
        let (mut x, width) = Self::ranked(a, len);
        let (mut y, _) = if same {
            (x.clone(), width)
        } else {
            Self::ranked(b, len)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algebra::AddMulOperation, rand, tools::Xorshift};

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
}
