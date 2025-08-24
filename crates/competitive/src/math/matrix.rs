use super::{Field, Invertible, Ring, SemiRing, SerdeByteStr};
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Add, Index, IndexMut, Mul, Sub},
};

pub struct Matrix<R>
where
    R: SemiRing,
{
    pub shape: (usize, usize),
    pub data: Vec<Vec<R::T>>,
    _marker: PhantomData<fn() -> R>,
}

impl<R> Debug for Matrix<R>
where
    R: SemiRing,
    R::T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Matrix")
            .field("shape", &self.shape)
            .field("data", &self.data)
            .field("_marker", &self._marker)
            .finish()
    }
}

impl<R> Clone for Matrix<R>
where
    R: SemiRing,
{
    fn clone(&self) -> Self {
        Self {
            shape: self.shape,
            data: self.data.clone(),
            _marker: self._marker,
        }
    }
}

impl<R> PartialEq for Matrix<R>
where
    R: SemiRing,
    R::T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape && self.data == other.data
    }
}

impl<R> Eq for Matrix<R>
where
    R: SemiRing,
    R::T: Eq,
{
}

impl<R> Matrix<R>
where
    R: SemiRing,
{
    pub fn new(shape: (usize, usize), z: R::T) -> Self {
        Self {
            shape,
            data: vec![vec![z; shape.1]; shape.0],
            _marker: PhantomData,
        }
    }
    pub fn from_vec(data: Vec<Vec<R::T>>) -> Self {
        Self {
            shape: (data.len(), data.first().map(Vec::len).unwrap_or_default()),
            data,
            _marker: PhantomData,
        }
    }
    pub fn from_fn(shape: (usize, usize), mut f: impl FnMut(usize, usize) -> R::T) -> Self {
        let data = (0..shape.0)
            .map(|i| (0..shape.1).map(|j| f(i, j)).collect())
            .collect();
        Self {
            shape,
            data,
            _marker: PhantomData,
        }
    }
    pub fn zeros(shape: (usize, usize)) -> Self {
        Self {
            shape,
            data: vec![vec![R::zero(); shape.1]; shape.0],
            _marker: PhantomData,
        }
    }
    pub fn eye(shape: (usize, usize)) -> Self {
        let mut data = vec![vec![R::zero(); shape.1]; shape.0];
        for (i, d) in data.iter_mut().enumerate() {
            d[i] = R::one();
        }
        Self {
            shape,
            data,
            _marker: PhantomData,
        }
    }
    // A^T B
    pub fn dot(&self, other: &Self) -> Self {
        assert_eq!(self.shape.0, other.shape.0);
        let mut res = Matrix::zeros((self.shape.1, other.shape.1));
        for k in 0..self.shape.0 {
            for i in 0..self.shape.1 {
                for j in 0..other.shape.1 {
                    R::add_assign(&mut res[i][j], &R::mul(&self[k][i], &other[k][j]));
                }
            }
        }
        res
    }
    pub fn map<S, F>(&self, mut f: F) -> Matrix<S>
    where
        S: SemiRing,
        F: FnMut(&R::T) -> S::T,
    {
        Matrix::<S>::from_fn(self.shape, |i, j| f(&self[i][j]))
    }
}
impl<R> Index<usize> for Matrix<R>
where
    R: SemiRing,
{
    type Output = Vec<R::T>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<R> IndexMut<usize> for Matrix<R>
where
    R: SemiRing,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
impl<R> Index<(usize, usize)> for Matrix<R>
where
    R: SemiRing,
{
    type Output = R::T;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0][index.1]
    }
}
impl<R> IndexMut<(usize, usize)> for Matrix<R>
where
    R: SemiRing,
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0][index.1]
    }
}
impl<R> Add for &Matrix<R>
where
    R: SemiRing,
{
    type Output = Matrix<R>;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.shape, rhs.shape);
        let mut res = self.clone();
        for i in 0..self.shape.0 {
            for j in 0..self.shape.1 {
                R::add_assign(&mut res[i][j], &rhs[i][j]);
            }
        }
        res
    }
}
impl<R> Sub for &Matrix<R>
where
    R: Ring,
    R::Additive: Invertible,
{
    type Output = Matrix<R>;
    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(self.shape, rhs.shape);
        let mut res = self.clone();
        for i in 0..self.shape.0 {
            for j in 0..self.shape.1 {
                R::sub_assign(&mut res[i][j], &rhs[i][j]);
            }
        }
        res
    }
}
impl<R> Mul for &Matrix<R>
where
    R: SemiRing,
{
    type Output = Matrix<R>;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.shape.1, rhs.shape.0);
        let mut res = Matrix::zeros((self.shape.0, rhs.shape.1));
        for i in 0..self.shape.0 {
            for k in 0..self.shape.1 {
                for j in 0..rhs.shape.1 {
                    R::add_assign(&mut res[i][j], &R::mul(&self[i][k], &rhs[k][j]));
                }
            }
        }
        res
    }
}
impl<R> Matrix<R>
where
    R: SemiRing,
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
impl<R> Matrix<R>
where
    R: Field,
    R::Additive: Invertible,
    R::Multiplicative: Invertible,
    R::T: PartialEq,
{
    pub fn row_reduction(&mut self, normalize: bool) {
        let (n, m) = self.shape;
        let mut c = 0;
        for r in 0..n {
            loop {
                if c >= m {
                    return;
                }
                if let Some(pivot) = (r..n).find(|&p| self[p][c] != R::zero()) {
                    self.data.swap(r, pivot);
                    break;
                };
                c += 1;
            }
            let d = R::Multiplicative::inverse(&self[r][c]);
            if normalize {
                for j in c..m {
                    R::mul_assign(&mut self[r][j], &d);
                }
            }
            for i in (0..n).filter(|&i| i != r) {
                let mut e = self[i][c].clone();
                if !normalize {
                    R::mul_assign(&mut e, &d);
                }
                for j in c..m {
                    let e = R::mul(&e, &self[r][j]);
                    R::sub_assign(&mut self[i][j], &e);
                }
            }
            c += 1;
        }
    }
    pub fn rank(&mut self) -> usize {
        let n = self.shape.0;
        self.row_reduction(false);
        (0..n)
            .filter(|&i| !self.data[i].iter().all(|x| x == &R::zero()))
            .count()
    }
    pub fn determinant(&mut self) -> R::T {
        assert_eq!(self.shape.0, self.shape.1);
        self.row_reduction(false);
        let mut d = R::one();
        for i in 0..self.shape.0 {
            R::mul_assign(&mut d, &self[i][i]);
        }
        d
    }
    pub fn solve_system_of_linear_equations(&self, b: &[R::T]) -> Option<Vec<R::T>> {
        assert_eq!(self.shape.0, b.len());
        let (n, m) = self.shape;
        let mut c = Matrix::<R>::zeros((n, m + 1));
        for i in 0..n {
            c[i][..m].clone_from_slice(&self[i]);
            c[i][m] = b[i].clone();
        }
        c.row_reduction(true);
        let mut x = vec![R::zero(); m];
        for i in 0..n {
            let mut j = 0usize;
            while j <= m && c[i][j] == R::zero() {
                j += 1;
            }
            if j == m {
                return None;
            }
            if j < m {
                x[j] = c[i][m].clone();
            }
        }
        Some(x)
    }
    pub fn inverse(&self) -> Option<Matrix<R>> {
        assert_eq!(self.shape.0, self.shape.1);
        let n = self.shape.0;
        let mut c = Matrix::<R>::zeros((n, n * 2));
        for i in 0..n {
            c[i][..n].clone_from_slice(&self[i]);
            c[i][n + i] = R::one();
        }
        c.row_reduction(true);
        if (0..n).any(|i| c[i][i] == R::zero()) {
            None
        } else {
            Some(Self::from_vec(
                c.data.into_iter().map(|r| r[n..].to_vec()).collect(),
            ))
        }
    }
}

impl<R> SerdeByteStr for Matrix<R>
where
    R: SemiRing,
    R::T: SerdeByteStr,
{
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.data.serialize(buf);
    }

    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        Self::from_vec(Vec::deserialize(iter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AddMulOperation,
        num::mint_basic::DynMIntU32,
        rand_value,
        tools::{RandomSpec, Xorshift},
    };
    struct D;
    impl RandomSpec<DynMIntU32> for D {
        fn rand(&self, rng: &mut Xorshift) -> DynMIntU32 {
            DynMIntU32::new_unchecked(rng.random(..DynMIntU32::get_mod()))
        }
    }

    #[test]
    fn test_row_reduction() {
        const Q: usize = 1000;
        let mut rng = Xorshift::new();
        let ps = [2, 3, 1_000_000_007];
        for _ in 0..Q {
            let m = ps[rng.random(..ps.len())];
            DynMIntU32::set_mod(m);
            let n = rng.random(2..=30);
            let mat = Matrix::<AddMulOperation<_>>::from_vec(rand_value!(rng, [[D; n]; n]));
            let rank = mat.clone().rank();
            let inv = mat.inverse();
            assert_eq!(rank == n, inv.is_some());
            if let Some(inv) = inv {
                assert_eq!(&mat * &inv, Matrix::<AddMulOperation<_>>::eye((n, n)));
            }
        }
    }
}
