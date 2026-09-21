use super::{Field, Invertible, Matrix, SemiRing};
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

pub trait BlackBoxMatrix<R>
where
    R: SemiRing,
{
    fn apply(&self, v: &[R::T]) -> Vec<R::T>;

    fn shape(&self) -> (usize, usize);
}

impl<R> BlackBoxMatrix<R> for Matrix<R>
where
    R: SemiRing,
{
    fn apply(&self, v: &[R::T]) -> Vec<R::T> {
        assert_eq!(self.shape.1, v.len());
        self.data.iter().map(|row| R::dot_product(row, v)).collect()
    }

    fn shape(&self) -> (usize, usize) {
        self.shape
    }
}

pub struct SparseMatrix<R>
where
    R: SemiRing,
{
    shape: (usize, usize),
    nonzero: Vec<(usize, usize, R::T)>,
}

impl<R> Debug for SparseMatrix<R>
where
    R: SemiRing<T: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SparseMatrix")
            .field("shape", &self.shape)
            .field("nonzero", &self.nonzero)
            .finish()
    }
}

impl<R> Clone for SparseMatrix<R>
where
    R: SemiRing,
{
    fn clone(&self) -> Self {
        Self {
            shape: self.shape,
            nonzero: self.nonzero.clone(),
        }
    }
}

impl<R> SparseMatrix<R>
where
    R: SemiRing,
{
    pub fn new(shape: (usize, usize)) -> Self {
        Self {
            shape,
            nonzero: vec![],
        }
    }
    pub fn new_with<F>(shape: (usize, usize), f: F) -> Self
    where
        R: SemiRing<T: PartialEq>,
        F: Fn(usize, usize) -> R::T,
    {
        let mut nonzero = vec![];
        for i in 0..shape.0 {
            for j in 0..shape.1 {
                let v = f(i, j);
                if !R::is_zero(&v) {
                    nonzero.push((i, j, v));
                }
            }
        }
        Self { shape, nonzero }
    }
    pub fn from_nonzero(shape: (usize, usize), nonzero: Vec<(usize, usize, R::T)>) -> Self {
        Self { shape, nonzero }
    }
}

impl<R> SparseMatrix<R>
where
    R: Field<T: PartialEq, Additive: Invertible, Multiplicative: Invertible>,
{
    pub fn determinant(&self) -> R::T {
        assert_eq!(self.shape.0, self.shape.1);
        let n = self.shape.0;
        let mut columns = vec![Vec::<(usize, R::T)>::new(); n];
        for &(i, j, ref value) in &self.nonzero {
            columns[j].push((i, value.clone()));
        }
        let mut degrees = vec![0; n];
        for column in &mut columns {
            column.sort_unstable_by_key(|&(i, _)| i);
            let mut merged: Vec<(usize, R::T)> = Vec::with_capacity(column.len());
            for (i, value) in column.drain(..) {
                if let Some((_, x)) = merged.last_mut().filter(|(last, _)| *last == i) {
                    R::add_assign(x, &value);
                } else {
                    merged.push((i, value));
                }
            }
            merged.retain(|(i, value)| {
                if R::is_zero(value) {
                    false
                } else {
                    degrees[*i] += 1;
                    true
                }
            });
            *column = merged;
        }
        let mut order: Vec<_> = (0..n).collect();
        order.sort_unstable_by_key(|&j| columns[j].len());
        let mut lower: Vec<Vec<(usize, R::T)>> = Vec::with_capacity(n);
        let mut pivots: Vec<Option<usize>> = vec![None; n];
        let mut x = vec![R::zero(); n];
        let mut seen = vec![0; n];
        let mut stack = Vec::new();
        let mut support = Vec::new();
        let mut determinant = R::one();
        for (k, &j) in order.iter().enumerate() {
            support.clear();
            for &(i, _) in &columns[j] {
                if seen[i] == k + 1 {
                    continue;
                }
                seen[i] = k + 1;
                x[i] = R::zero();
                stack.push((i, 0));
                while let Some((i, next)) = stack.last_mut() {
                    if let Some(pivot) = pivots[*i]
                        && *next < lower[pivot].len()
                    {
                        let row = lower[pivot][*next].0;
                        *next += 1;
                        if seen[row] != k + 1 {
                            seen[row] = k + 1;
                            x[row] = R::zero();
                            stack.push((row, 0));
                        }
                        continue;
                    }
                    support.push(*i);
                    stack.pop();
                }
            }
            for &(i, ref value) in &columns[j] {
                x[i] = value.clone();
            }
            let mut pivot = None;
            for &i in support.iter().rev() {
                if let Some(p) = pivots[i] {
                    let factor = R::neg(&x[i]);
                    for &(row, ref value) in &lower[p] {
                        R::add_assign(&mut x[row], &R::mul(&factor, value));
                    }
                } else if !R::is_zero(&x[i]) && pivot.is_none_or(|p| degrees[i] < degrees[p]) {
                    pivot = Some(i);
                }
            }
            let Some(pivot) = pivot else { return R::zero() };
            R::mul_assign(&mut determinant, &x[pivot]);
            let inv = R::inv(&x[pivot]);
            pivots[pivot] = Some(k);
            lower.push(
                support
                    .iter()
                    .filter(|&&i| pivots[i].is_none() && !R::is_zero(&x[i]))
                    .map(|&i| (i, R::mul(&x[i], &inv)))
                    .collect(),
            );
        }
        for mut permutation in [order, pivots.into_iter().map(Option::unwrap).collect()] {
            for i in 0..n {
                while permutation[i] != i {
                    let j = permutation[i];
                    permutation.swap(i, j);
                    determinant = R::neg(&determinant);
                }
            }
        }
        determinant
    }
}

impl<R> From<Matrix<R>> for SparseMatrix<R>
where
    R: SemiRing<T: PartialEq>,
{
    fn from(mat: Matrix<R>) -> Self {
        let mut nonzero = vec![];
        for i in 0..mat.shape.0 {
            for j in 0..mat.shape.1 {
                let v = mat[(i, j)].clone();
                if !R::is_zero(&v) {
                    nonzero.push((i, j, v));
                }
            }
        }
        Self {
            shape: mat.shape,
            nonzero,
        }
    }
}

impl<R> From<SparseMatrix<R>> for Matrix<R>
where
    R: SemiRing,
{
    fn from(smat: SparseMatrix<R>) -> Self {
        let mut mat = Matrix::zeros(smat.shape);
        for &(i, j, ref v) in &smat.nonzero {
            R::add_assign(&mut mat[(i, j)], v);
        }
        mat
    }
}

impl<R> BlackBoxMatrix<R> for SparseMatrix<R>
where
    R: SemiRing,
{
    fn apply(&self, v: &[R::T]) -> Vec<R::T> {
        assert_eq!(self.shape.1, v.len());
        let mut res = vec![R::zero(); self.shape.0];
        for &(i, j, ref val) in &self.nonzero {
            R::add_assign(&mut res[i], &R::mul(val, &v[j]));
        }
        res
    }

    fn shape(&self) -> (usize, usize) {
        self.shape
    }
}

pub struct BlackBoxMatrixImpl<R, F> {
    shape: (usize, usize),
    apply_fn: F,
    _marker: PhantomData<fn() -> R>,
}

impl<R, F> Debug for BlackBoxMatrixImpl<R, F>
where
    F: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlackBoxMatrixImpl")
            .field("shape", &self.shape)
            .field("apply_fn", &self.apply_fn)
            .finish()
    }
}

impl<R, F> Clone for BlackBoxMatrixImpl<R, F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            shape: self.shape,
            apply_fn: self.apply_fn.clone(),
            _marker: PhantomData,
        }
    }
}

impl<R, F> BlackBoxMatrixImpl<R, F> {
    pub fn new(shape: (usize, usize), apply_fn: F) -> Self {
        Self {
            shape,
            apply_fn,
            _marker: PhantomData,
        }
    }
}

impl<R, F> BlackBoxMatrix<R> for BlackBoxMatrixImpl<R, F>
where
    R: SemiRing,
    F: Fn(&[R::T]) -> Vec<R::T>,
{
    fn apply(&self, v: &[R::T]) -> Vec<R::T> {
        assert_eq!(self.shape.1, v.len());
        (self.apply_fn)(v)
    }

    fn shape(&self) -> (usize, usize) {
        self.shape
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AddMulOperation,
        math::{BlackBoxMIntMatrix, Convolve998244353},
        num::{Zero, montgomery::MInt998244353},
        rand,
        tools::Xorshift,
    };

    type R = AddMulOperation<MInt998244353>;

    fn random_matrix(rng: &mut Xorshift, shape: (usize, usize)) -> Matrix<R> {
        if rng.gen_bool(0.5) {
            Matrix::<R>::new_with(shape, |_, _| rng.random(..))
        } else if rng.gen_bool(0.5) {
            let r = rng.randf();
            Matrix::<R>::new_with(shape, |_, _| {
                if rng.gen_bool(r) {
                    rng.random(..)
                } else {
                    MInt998244353::zero()
                }
            })
        } else {
            let mut mat = Matrix::<R>::new_with(shape, |_, _| rng.random(..));
            let i0 = rng.random(0..shape.0);
            let i1 = rng.random(0..shape.0);
            let x: MInt998244353 = rng.random(..);
            for j in 0..shape.1 {
                mat[(i0, j)] = mat[(i1, j)] * x;
            }
            mat
        }
    }

    #[test]
    fn test_apply() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30, m: 1..30);
            let mat = random_matrix(&mut rng, (n, m));
            let smat = SparseMatrix::from(mat.clone());
            let v: Vec<_> = (0..m).map(|_| rng.random(..)).collect();
            let av = mat.apply(&v);
            let asv = smat.apply(&v);
            assert_eq!(av, asv);
        }
    }

    #[test]
    fn test_minimal_polynomial() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30);
            let a = random_matrix(&mut rng, (n, n));
            let p = a.minimal_polynomial();
            assert!(p.len() <= n + 1);
            let mut res = Matrix::<R>::zeros((n, n));
            let mut pow = Matrix::<R>::eye((n, n));
            for p in p {
                for i in 0..n {
                    for j in 0..n {
                        res[(i, j)] += p * pow[(i, j)];
                    }
                }
                pow = &pow * &a;
            }
            assert_eq!(res, Matrix::<R>::zeros((n, n)));
        }
    }

    #[test]
    fn test_apply_pow() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30, k: 0..1_000_000_000);
            let a = random_matrix(&mut rng, (n, n));
            let b: Vec<_> = (0..n).map(|_| rng.random(..)).collect();
            let expected = a.clone().pow(k).apply(&b);
            let result = a.apply_pow::<Convolve998244353>(b, k);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_sparse_determinant() {
        let mut rng = Xorshift::new_with_seed(94623);
        for _ in 0..500 {
            let n = rng.random(0..40);
            let count = rng.random(0..n * n * 2 + 1);
            let mut entries = Vec::new();
            for _ in 0..count {
                let i = rng.random(0..n);
                let j = rng.random(0..n);
                let value: MInt998244353 = rng.random(..);
                entries.push((i, j, value));
                if rng.gen_bool(0.25) {
                    entries.push((i, j, -value));
                }
            }
            let sparse = SparseMatrix::<R>::from_nonzero((n, n), entries);
            let expected = Matrix::from(sparse.clone()).determinant();
            assert_eq!(sparse.determinant(), expected);
        }
    }

    #[test]
    fn test_black_box_determinant() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30);
            let mut a = random_matrix(&mut rng, (n, n));
            let result = a.black_box_determinant();
            let expected = a.determinant();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_black_box_linear_equation() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30);
            let a = random_matrix(&mut rng, (n, n));
            let b: Vec<_> = (0..n).map(|_| rng.random(..)).collect();
            let expected = a
                .solve_system_of_linear_equations(&b)
                .map(|sol| sol.particular);
            let result = a.black_box_linear_equation(b);
            assert_eq!(result, expected);
        }
    }
}
