#[cargo_snippet::snippet("Matrix")]
use std::ops::{Add, Index, IndexMut, Mul, Sub};
#[cargo_snippet::snippet("Matrix")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix<T> {
    pub shape: (usize, usize),
    pub data: Vec<Vec<T>>,
}
#[cargo_snippet::snippet("Matrix")]
impl<T: Clone> Matrix<T> {
    pub fn new(shape: (usize, usize), z: T) -> Self {
        Matrix {
            shape: shape,
            data: vec![vec![z; shape.1]; shape.0],
        }
    }
}
#[cargo_snippet::snippet("Matrix")]
impl<T> Matrix<T> {
    pub fn from_vec(data: Vec<Vec<T>>) -> Self {
        Matrix {
            shape: (data.len(), data.get(0).map(|v| v.len()).unwrap_or(0)),
            data: data,
        }
    }
}
#[cargo_snippet::snippet("Matrix")]
impl<T: Clone + Default> Matrix<T> {
    pub fn zeros(shape: (usize, usize)) -> Self {
        Matrix {
            shape: shape,
            data: vec![vec![Default::default(); shape.1]; shape.0],
        }
    }
    pub fn eye(shape: (usize, usize), one: T) -> Self {
        let mut data = vec![vec![Default::default(); shape.1]; shape.0];
        for i in 0..shape.0 {
            data[i][i] = one.clone();
        }
        Matrix {
            shape: shape,
            data: data,
        }
    }
}
#[cargo_snippet::snippet("Matrix")]
impl<T> Index<usize> for Matrix<T> {
    type Output = Vec<T>;
    fn index(&self, i: usize) -> &Self::Output {
        &self.data[i]
    }
}
#[cargo_snippet::snippet("Matrix")]
impl<T> IndexMut<usize> for Matrix<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.data[i]
    }
}
#[cargo_snippet::snippet("Matrix")]
impl<'a, T: Copy + Default + Add<Output = T>> Add for &'a Matrix<T> {
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
#[cargo_snippet::snippet("Matrix")]
impl<'a, T: Copy + Default + Sub<Output = T>> Sub for &'a Matrix<T> {
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
#[cargo_snippet::snippet("Matrix")]
impl<'a, T: Copy + Default + Add<Output = T> + Sub<Output = T> + Mul<Output = T>> Mul
    for &'a Matrix<T>
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
#[cargo_snippet::snippet("Matrix")]
impl<T: Copy + Default + Add<Output = T> + Sub<Output = T> + Mul<Output = T>> Matrix<T> {
    pub fn pow(&self, n: usize, one: T) -> Self {
        assert_eq!(self.shape.0, self.shape.1);
        let mut x = self.clone();
        let mut n = n;
        let mut res = Matrix::eye(self.shape, one);
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
impl Matrix<f64> {
    pub fn gauss_jordan(&self, b: &Vec<f64>) -> Option<Vec<f64>> {
        assert_eq!(self.shape.0, self.shape.1);
        assert_eq!(self.shape.0, b.len());
        let n = self.shape.0;
        let mut c = Matrix::zeros((self.shape.0, self.shape.1 + 1));
        for i in 0..n {
            for j in 0..n {
                c[i][j] = self[i][j];
            }
            c[i][n] = b[i];
        }
        for i in 0..n {
            let pivot = (i..n)
                .max_by(|&j, &k| c[j][i].partial_cmp(&c[k][i]).unwrap())
                .unwrap();
            c.data.swap(i, pivot);
            if c[i][i].abs() < 1e-8 {
                return None;
            }
            for j in i + 1..n + 1 {
                c[i][j] /= c[i][i];
            }
            for j in 0..n {
                if i != j {
                    for k in i + 1..n + 1 {
                        c[j][k] -= c[j][i] * c[i][k];
                    }
                }
            }
        }
        Some((0..n).map(|i| c[i][n]).collect::<Vec<_>>())
    }
}

#[test]
fn test_gauss_jordan() {
    let a = Matrix::from_vec(vec![
        vec![1., -2., 3.],
        vec![4., -5., 6.],
        vec![7., -8., 10.],
    ]);
    let b = vec![6., 12., 21.];
    let x = a.gauss_jordan(&b).unwrap();
    let expect = vec![1., 2., 3.];
    for i in 0..3 {
        assert!((x[i] - expect[i]).abs() < 1e-10);
    }
}