use crate::num::{One, Zero};

#[codesnip::entry("Matrix")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix<T> {
    pub shape: (usize, usize),
    pub data: Vec<Vec<T>>,
}
#[codesnip::entry("Matrix")]
mod matrix_impls {
    use super::*;
    use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};
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
    impl<T: Clone + Zero> Matrix<T> {
        pub fn zeros(shape: (usize, usize)) -> Self {
            Self {
                shape,
                data: vec![vec![Zero::zero(); shape.1]; shape.0],
            }
        }
    }
    impl<T: Clone + Zero + One> Matrix<T> {
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
    impl<'a, T: Copy + Zero + Add<Output = T>> Add for &'a Matrix<T> {
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
    impl<'a, T: Copy + Zero + Sub<Output = T>> Sub for &'a Matrix<T> {
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
    impl<'a, T: Copy + Zero + Add<Output = T> + Mul<Output = T>> Mul for &'a Matrix<T> {
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
    impl<T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>> Matrix<T> {
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
    impl<T: Copy + Zero + One + Sub<Output = T> + Mul<Output = T> + Div<Output = T>> Matrix<T> {
        pub fn row_reduction(&mut self) {
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
                for j in c..m {
                    self[r][j] = self[r][j] * d;
                }
                for i in (0..n).filter(|&i| i != r) {
                    let d = self[i][c];
                    for j in c..m {
                        self[i][j] = self[i][j] - d * self[r][j];
                    }
                }
                c += 1;
            }
        }
        pub fn rank(&mut self) -> usize {
            let n = self.shape.0;
            self.row_reduction();
            (0..n).filter(|&i| !self[i][i].is_zero()).count()
        }
        pub fn solve_system_of_linear_equations(&self, b: &[T]) -> Option<Vec<T>> {
            assert_eq!(self.shape.0, self.shape.1);
            assert_eq!(self.shape.0, b.len());
            let n = self.shape.0;
            let mut c = Matrix::<T>::zeros((n, n + 1));
            for i in 0..n {
                c[i][..n].clone_from_slice(&self[i]);
                c[i][n] = b[i];
            }
            c.row_reduction();
            if (0..n).any(|i| c[i][i].is_zero()) {
                None
            } else {
                Some((0..n).map(|i| c[i][n]).collect::<Vec<_>>())
            }
        }
        pub fn inverse(&self) -> Option<Matrix<T>> {
            assert_eq!(self.shape.0, self.shape.1);
            let n = self.shape.0;
            let mut c = Matrix::<T>::zeros((n, n * 2));
            for i in 0..n {
                c[i][..n].clone_from_slice(&self[i]);
                c[i][n + i] = T::one();
            }
            c.row_reduction();
            if (0..n).any(|i| c[i][i].is_zero()) {
                None
            } else {
                Some(Self::from_vec(
                    c.data.into_iter().map(|r| r[n..].to_vec()).collect(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::{MInt, Modulus};
    use crate::tools::Xorshift;
    struct DM {}
    static mut MOD: u32 = 2;
    impl Modulus for DM {
        #[inline]
        fn get_modulus() -> u32 {
            unsafe { MOD }
        }
    }

    #[test]
    fn test_row_reduction() {
        const Q: usize = 1000;
        type M = MInt<DM>;
        let mut rand = Xorshift::time();
        let ps = vec![2, 3, 1_000_000_007];
        for _ in 0..Q {
            unsafe { MOD = ps[rand.rand(ps.len() as u64) as usize] as u32 };
            let n = rand.rand(30) as usize + 2;
            let mat = Matrix::from_vec(
                (0..n)
                    .map(|_| {
                        (0..n)
                            .map(|_| M::from(rand.rand(M::get_mod() as u64)))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            );
            let rank = mat.clone().rank();
            let inv = mat.inverse();
            assert_eq!(rank == n, inv.is_some());
            if let Some(inv) = inv {
                assert_eq!(&mat * &inv, Matrix::eye((n, n)));
            }
        }
    }
}
