use super::{One, Zero};
use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix<T> {
    pub shape: (usize, usize),
    pub data: Vec<Vec<T>>,
}

impl<T: Clone> Matrix<T> {
    pub fn new(shape: (usize, usize), z: T) -> Self {
        Self {
            shape,
            data: vec![vec![z; shape.1]; shape.0],
        }
    }
}
impl<T> Matrix<T> {
    pub fn from_vec(data: Vec<Vec<T>>) -> Self {
        Self {
            shape: (data.len(), data.first().map(Vec::len).unwrap_or_default()),
            data,
        }
    }
}
impl<T> Matrix<T>
where
    T: Clone + Zero,
{
    pub fn zeros(shape: (usize, usize)) -> Self {
        Self {
            shape,
            data: vec![vec![Zero::zero(); shape.1]; shape.0],
        }
    }
}
impl<T> Matrix<T>
where
    T: Clone + Zero + One,
{
    pub fn eye(shape: (usize, usize)) -> Self {
        let mut data = vec![vec![Zero::zero(); shape.1]; shape.0];
        for (i, d) in data.iter_mut().enumerate() {
            d[i] = One::one();
        }
        Self { shape, data }
    }
}
impl<T> Index<usize> for Matrix<T> {
    type Output = Vec<T>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<T> IndexMut<usize> for Matrix<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0][index.1]
    }
}
impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0][index.1]
    }
}
impl<'a, T> Add for &'a Matrix<T>
where
    T: Copy + Zero + Add<Output = T>,
{
    type Output = Matrix<T>;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.shape, rhs.shape);
        let mut res = Matrix::zeros(self.shape);
        for i in 0..self.shape.0 {
            for j in 0..self.shape.1 {
                res[i][j] = self[i][j] + rhs[i][j];
            }
        }
        res
    }
}
impl<'a, T> Sub for &'a Matrix<T>
where
    T: Copy + Zero + Sub<Output = T>,
{
    type Output = Matrix<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(self.shape, rhs.shape);
        let mut res = Matrix::zeros(self.shape);
        for i in 0..self.shape.0 {
            for j in 0..self.shape.1 {
                res[i][j] = self[i][j] - rhs[i][j];
            }
        }
        res
    }
}
impl<'a, T> Mul for &'a Matrix<T>
where
    T: Copy + Zero + Add<Output = T> + Mul<Output = T>,
{
    type Output = Matrix<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.shape.1, rhs.shape.0);
        let mut res = Matrix::zeros((self.shape.0, rhs.shape.1));
        for i in 0..self.shape.0 {
            for j in 0..rhs.shape.1 {
                for k in 0..self.shape.1 {
                    res[i][j] = res[i][j] + self[i][k] * rhs[k][j];
                }
            }
        }
        res
    }
}
impl<T> Matrix<T>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>,
{
    pub fn pow(&self, mut n: usize) -> Self {
        assert_eq!(self.shape.0, self.shape.1);
        let mut x = self.clone();
        let mut res = Matrix::eye(self.shape);
        while n > 0 {
            if n & 1 == 1 {
                res = &res * &x;
            }
            x = &x * &x;
            n >>= 1;
        }
        res
    }
}
impl<T> Matrix<T>
where
    T: Copy + PartialEq + Zero + One + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    pub fn row_reduction(&mut self, normalize: bool) {
        let (n, m) = self.shape;
        let mut c = 0;
        for r in 0..n {
            loop {
                if c >= m {
                    return;
                }
                if let Some(pivot) = (r..n).find(|&p| !self[p][c].is_zero()) {
                    self.data.swap(r, pivot);
                    break;
                };
                c += 1;
            }
            let d = T::one() / self[r][c];
            if normalize {
                for j in c..m {
                    self[r][j] = self[r][j] * d;
                }
            }
            for i in (0..n).filter(|&i| i != r) {
                let mut e = self[i][c];
                if !normalize {
                    e = e * d;
                }
                for j in c..m {
                    self[i][j] = self[i][j] - e * self[r][j];
                }
            }
            c += 1;
        }
    }
    pub fn rank(&mut self) -> usize {
        let n = self.shape.0;
        self.row_reduction(false);
        (0..n)
            .filter(|&i| !self.data[i].iter().all(|x| x.is_zero()))
            .count()
    }
    pub fn determinant(&mut self) -> T {
        assert_eq!(self.shape.0, self.shape.1);
        self.row_reduction(false);
        let mut d = T::one();
        for i in 0..self.shape.0 {
            d = d * self[i][i];
        }
        d
    }
    pub fn solve_system_of_linear_equations(&self, b: &[T]) -> Option<Vec<T>> {
        assert_eq!(self.shape.0, b.len());
        let (n, m) = self.shape;
        let mut c = Matrix::<T>::zeros((n, m + 1));
        for i in 0..n {
            c[i][..m].clone_from_slice(&self[i]);
            c[i][m] = b[i];
        }
        c.row_reduction(true);
        let mut x = vec![T::zero(); m];
        for i in 0..n {
            let mut j = 0usize;
            while j <= m && c[i][j].is_zero() {
                j += 1;
            }
            if j == m {
                return None;
            }
            if j < m {
                x[j] = c[i][m];
            }
        }
        Some(x)
    }
    pub fn inverse(&self) -> Option<Matrix<T>> {
        assert_eq!(self.shape.0, self.shape.1);
        let n = self.shape.0;
        let mut c = Matrix::<T>::zeros((n, n * 2));
        for i in 0..n {
            c[i][..n].clone_from_slice(&self[i]);
            c[i][n + i] = T::one();
        }
        c.row_reduction(true);
        if (0..n).any(|i| c[i][i].is_zero()) {
            None
        } else {
            Some(Self::from_vec(
                c.data.into_iter().map(|r| r[n..].to_vec()).collect(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        num::mint_basic::DynMIntU32,
        rand_value,
        tools::{RandomSpec, Xorshift},
    };
    struct D;
    impl RandomSpec<DynMIntU32> for D {
        fn rand(&self, rng: &mut Xorshift) -> DynMIntU32 {
            DynMIntU32::new_unchecked(rng.gen(..DynMIntU32::get_mod()))
        }
    }

    #[test]
    fn test_row_reduction() {
        const Q: usize = 1000;
        let mut rng = Xorshift::new();
        let ps = vec![2, 3, 1_000_000_007];
        for _ in 0..Q {
            let m = ps[rng.gen(..ps.len())];
            DynMIntU32::set_mod(m);
            let n = rng.gen(2..=30);
            let mat = Matrix::from_vec(rand_value!(rng, [[D; n]; n]));
            let rank = mat.clone().rank();
            let inv = mat.inverse();
            assert_eq!(rank == n, inv.is_some());
            if let Some(inv) = inv {
                assert_eq!(&mat * &inv, Matrix::eye((n, n)));
            }
        }
    }
}
