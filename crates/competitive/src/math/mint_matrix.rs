use super::{
    AddMulOperation, MInt, MIntBase, MIntConvert, Matrix, MemorizedFactorial, One, Xorshift, Zero,
};

pub trait MIntMatrix<M>
where
    M: MIntBase,
{
    /// det(self + other * x)
    fn determinant_linear(self, other: Self) -> Option<Vec<MInt<M>>>
    where
        M: MIntConvert<usize> + MIntConvert<u64>;
}

impl<M> MIntMatrix<M> for Matrix<AddMulOperation<MInt<M>>>
where
    M: MIntBase,
{
    fn determinant_linear(mut self, other: Self) -> Option<Vec<MInt<M>>>
    where
        M: MIntConvert<usize> + MIntConvert<u64>,
    {
        let mut rng = Xorshift::new();
        let a = MInt::from(rng.rand64());
        let n = self.data.len();
        for i in 0..n {
            for j in 0..n {
                self[i][j] += other[i][j] * a;
            }
        }
        let mut f = other.determinant_linear_non_singular(self)?;
        f.reverse();
        Some(taylor_shift::<M>(f, -a))
    }
}

impl<M> Matrix<AddMulOperation<MInt<M>>>
where
    M: MIntBase,
{
    fn determinant_linear_non_singular(mut self, mut other: Self) -> Option<Vec<MInt<M>>>
    where
        M: MIntBase,
    {
        let n = self.data.len();
        let mut f = MInt::one();
        for d in 0..n {
            let i = other.data.iter().position(|other| !other[d].is_zero())?;
            if i != d {
                self.data.swap(i, d);
                other.data.swap(i, d);
                f = -f;
            }
            f *= other[d][d];
            let r = other[d][d].inv();
            for j in 0..n {
                self[d][j] *= r;
                other[d][j] *= r;
            }
            assert!(other[d][d].is_one());
            for i in d + 1..n {
                let a = other[i][d];
                for k in 0..n {
                    self[i][k] = self[i][k] - a * self[d][k];
                    other[i][k] = other[i][k] - a * other[d][k];
                }
            }
            for j in d + 1..n {
                let a = other[d][j];
                for k in 0..n {
                    self[k][j] = self[k][j] - a * self[k][d];
                    other[k][j] = other[k][j] - a * other[k][d];
                }
            }
        }
        for s in self.data.iter_mut() {
            for s in s.iter_mut() {
                *s = -*s;
            }
        }
        let mut p = self.characteristic_polynomial();
        for p in p.iter_mut() {
            *p *= f;
        }
        Some(p)
    }
}

fn taylor_shift<M>(f: Vec<MInt<M>>, a: MInt<M>) -> Vec<MInt<M>>
where
    M: MIntConvert<usize>,
{
    let n = f.len();
    if n == 0 {
        return f;
    }
    let mf = MemorizedFactorial::new(n);
    let mut res = vec![MInt::<M>::zero(); n];
    let mut apow = vec![MInt::<M>::one(); n];
    for i in 1..n {
        apow[i] = apow[i - 1] * a;
    }
    for j in 0..n {
        if f[j].is_zero() {
            continue;
        }
        for k in 0..=j {
            res[k] += f[j] * apow[j - k] * mf.combination(j, k);
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::lagrange_interpolation_polynomial, num::montgomery::MInt998244353, rand};

    #[test]
    fn test_determinant_linear() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30, m0: [[0..998244353; n]; n], m1: [[0..998244353; n]; n]);
            let m0 = Matrix::<AddMulOperation<_>>::from_vec(m0)
                .map::<AddMulOperation<MInt998244353>, _>(|&x| MInt998244353::new(x));
            let m1 = Matrix::<AddMulOperation<_>>::from_vec(m1)
                .map::<AddMulOperation<MInt998244353>, _>(|&x| MInt998244353::new(x));
            let f = m0.clone().determinant_linear(m1.clone()).unwrap();

            let d: Vec<_> = (0..=n)
                .map(|k| {
                    let mut mat = Matrix::<AddMulOperation<_>>::new_with((n, n), |i, j| {
                        m0[i][j] + m1[i][j] * MInt998244353::from(k)
                    });
                    mat.determinant()
                })
                .collect();
            let (x, y): (Vec<_>, Vec<_>) = (0..=n).map(|k| (MInt998244353::from(k), d[k])).unzip();
            let g = lagrange_interpolation_polynomial(&x, &y);
            assert_eq!(f, g);
        }
    }
}
