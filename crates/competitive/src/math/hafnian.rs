use super::{Invertible, Matrix, Ring, SubsetConvolve};

impl<R> Matrix<R>
where
    R: Ring<T: PartialEq, Additive: Invertible>,
{
    /// Sums the weights of perfect matchings of a symmetric square matrix over a commutative ring.
    pub fn hafnian(&self) -> R::T {
        assert_eq!(self.shape.0, self.shape.1);
        let n = self.shape.0;
        let matrix = &self.data;
        if n == 0 {
            return R::one();
        }
        if !n.is_multiple_of(2) {
            return R::zero();
        }
        let pairs = n / 2;
        let mut cycles = vec![R::zero(); 1 << pairs];
        for anchor in 0..pairs {
            let width = 2 * anchor;
            let base = 1 << anchor;
            cycles[base] = matrix[width][width + 1].clone();
            let mut paths = vec![R::zero(); base * width];
            for mask in 1..base {
                let mut remaining = mask;
                while remaining != 0 {
                    let pair = remaining.trailing_zeros() as usize;
                    remaining &= remaining - 1;
                    let previous = mask ^ (1 << pair);
                    for end in 2 * pair..2 * pair + 2 {
                        // Enter the final fixed pair at the vertex opposite the endpoint.
                        let value = if previous == 0 {
                            matrix[width][end ^ 1].clone()
                        } else {
                            let row = &paths[previous * width..(previous + 1) * width];
                            R::dot_product(row, &matrix[end ^ 1][..width])
                        };
                        paths[mask * width + end] = value;
                    }
                }
                let row = &paths[mask * width..(mask + 1) * width];
                cycles[base + mask] = R::dot_product(row, &matrix[width + 1][..width]);
            }
        }
        let (other_cycles, last_cycle) = cycles.split_at_mut(1 << (pairs - 1));
        // Reversing subset order pairs each last cycle with its complement.
        last_cycle.reverse();
        R::dot_product(&SubsetConvolve::<R>::exp(other_cycles), last_cycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algebra::AddMulOperation, rand, tools::Xorshift};

    #[test]
    fn perfect_matchings() {
        let mut rng = Xorshift::default();
        for n in 0..=14 {
            for mode in 0..5 {
                let mut matrix = vec![vec![0i64; n]; n];
                for i in 0..n {
                    let (previous, current) = matrix.split_at_mut(i);
                    for (row, entry) in previous.iter_mut().zip(&mut current[0]) {
                        rand!(rng, weight: -4i64..=4);
                        *entry = match mode {
                            0 => 0,
                            1 => 1,
                            2 => -1,
                            3 => i64::from(weight == 0),
                            _ => weight,
                        };
                        row[i] = *entry;
                    }
                }
                let mut dp = vec![0; 1 << n];
                dp[0] = 1;
                for mask in 1usize..1 << n {
                    let first = mask.trailing_zeros() as usize;
                    let rest = mask & (mask - 1);
                    let mut partners = rest;
                    while partners != 0 {
                        let j = partners.trailing_zeros() as usize;
                        dp[mask] += matrix[first][j] * dp[rest ^ (1 << j)];
                        partners &= partners - 1;
                    }
                }
                assert_eq!(
                    Matrix::<AddMulOperation<_>>::from_vec(matrix).hafnian(),
                    dp[(1 << n) - 1]
                );
            }
        }
    }
}
