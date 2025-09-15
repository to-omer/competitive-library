use super::{
    AddMulOperation, ConvolveSteps, FormalPowerSeries, MInt, MIntBase, MIntConvert, Matrix, One,
    SemiRing, Xorshift, Zero, berlekamp_massey,
};
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
        let mut res = vec![R::zero(); self.shape.0];
        for i in 0..self.shape.0 {
            for j in 0..self.shape.1 {
                R::add_assign(&mut res[i], &R::mul(&self[(i, j)], &v[j]));
            }
        }
        res
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
    R: SemiRing,
    R::T: Debug,
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
        R::T: PartialEq,
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

impl<R> From<Matrix<R>> for SparseMatrix<R>
where
    R: SemiRing,
    R::T: PartialEq,
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

pub trait BlackBoxMIntMatrix<M>: BlackBoxMatrix<AddMulOperation<MInt<M>>>
where
    M: MIntBase,
{
    fn minimal_polynomial(&self) -> Vec<MInt<M>>
    where
        M: MIntConvert<u64>,
    {
        assert_eq!(self.shape().0, self.shape().1);
        let n = self.shape().0;
        let mut rng = Xorshift::new();
        let b: Vec<MInt<M>> = (0..n).map(|_| MInt::from(rng.rand64())).collect();
        let u: Vec<MInt<M>> = (0..n).map(|_| MInt::from(rng.rand64())).collect();
        let a: Vec<MInt<M>> = (0..2 * n)
            .scan(b, |b, _| {
                let a = b.iter().zip(&u).fold(MInt::zero(), |s, (x, y)| s + x * y);
                *b = self.apply(b);
                Some(a)
            })
            .collect();
        let mut p = berlekamp_massey(&a);
        p.reverse();
        p
    }

    fn apply_pow<C>(&self, mut b: Vec<MInt<M>>, k: usize) -> Vec<MInt<M>>
    where
        M: MIntConvert<usize> + MIntConvert<u64>,
        C: ConvolveSteps<T = Vec<MInt<M>>>,
    {
        assert_eq!(self.shape().0, self.shape().1);
        assert_eq!(self.shape().1, b.len());
        let n = self.shape().0;
        let p = self.minimal_polynomial();
        let f = FormalPowerSeries::<MInt<M>, C>::from_vec(p).pow_mod(k);
        let mut res = vec![MInt::zero(); n];
        for f in f {
            for j in 0..n {
                res[j] += f * b[j];
            }
            b = self.apply(&b);
        }
        res
    }

    fn black_box_determinant(&self) -> MInt<M>
    where
        M: MIntConvert<u64>,
    {
        assert_eq!(self.shape().0, self.shape().1);
        let n = self.shape().0;
        let mut rng = Xorshift::new();
        let d: Vec<MInt<M>> = (0..n).map(|_| MInt::from(rng.rand64())).collect();
        let det_d = d.iter().fold(MInt::one(), |s, x| s * x);
        let ad = BlackBoxMatrixImpl::<AddMulOperation<MInt<M>>, _>::new(
            self.shape(),
            |v: &[MInt<M>]| {
                let mut w = self.apply(v);
                for (w, d) in w.iter_mut().zip(&d) {
                    *w *= d;
                }
                w
            },
        );
        let p = ad.minimal_polynomial();
        let det_ad = if n % 2 == 0 { p[0] } else { -p[0] };
        det_ad / det_d
    }

    fn black_box_linear_equation(&self, mut b: Vec<MInt<M>>) -> Option<Vec<MInt<M>>>
    where
        M: MIntConvert<u64>,
    {
        assert_eq!(self.shape().0, self.shape().1);
        assert_eq!(self.shape().1, b.len());
        let n = self.shape().0;
        let p = self.minimal_polynomial();
        if p.is_empty() || p[0].is_zero() {
            return None;
        }
        let p0_inv = p[0].inv();
        let mut x = vec![MInt::zero(); n];
        for p in p.into_iter().skip(1) {
            let p = -p * p0_inv;
            for i in 0..n {
                x[i] += p * b[i];
            }
            b = self.apply(&b);
        }
        Some(x)
    }
}

impl<M, B> BlackBoxMIntMatrix<M> for B
where
    M: MIntBase,
    B: BlackBoxMatrix<AddMulOperation<MInt<M>>>,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::Convolve998244353, num::montgomery::MInt998244353, rand, tools::RandomSpec};

    struct D;
    impl RandomSpec<MInt998244353> for D {
        fn rand(&self, rng: &mut Xorshift) -> MInt998244353 {
            MInt998244353::new_unchecked(rng.random(..MInt998244353::get_mod()))
        }
    }

    fn random_matrix(
        rng: &mut Xorshift,
        shape: (usize, usize),
    ) -> Matrix<AddMulOperation<MInt998244353>> {
        if rng.gen_bool(0.5) {
            Matrix::<AddMulOperation<_>>::new_with(shape, |_, _| rng.random(D))
        } else if rng.gen_bool(0.5) {
            let r = rng.randf();
            Matrix::<AddMulOperation<_>>::new_with(shape, |_, _| {
                if rng.gen_bool(r) {
                    rng.random(D)
                } else {
                    MInt998244353::zero()
                }
            })
        } else {
            let mut mat = Matrix::<AddMulOperation<_>>::new_with(shape, |_, _| rng.random(D));
            let i0 = rng.random(0..shape.0);
            let i1 = rng.random(0..shape.0);
            let x = rng.random(D);
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
            let v: Vec<_> = (0..m).map(|_| rng.random(D)).collect();
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
            let mut res = Matrix::<AddMulOperation<MInt998244353>>::zeros((n, n));
            let mut pow = Matrix::<AddMulOperation<MInt998244353>>::eye((n, n));
            for p in p {
                for i in 0..n {
                    for j in 0..n {
                        res[(i, j)] += p * pow[(i, j)];
                    }
                }
                pow = &pow * &a;
            }
            assert_eq!(res, Matrix::<AddMulOperation<MInt998244353>>::zeros((n, n)));
        }
    }

    #[test]
    fn test_apply_pow() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30, k: 0..1_000_000_000);
            let a = random_matrix(&mut rng, (n, n));
            let b: Vec<_> = (0..n).map(|_| rng.random(D)).collect();
            let expected = a.clone().pow(k).apply(&b);
            let result = a.apply_pow::<Convolve998244353>(b, k);
            assert_eq!(result, expected);
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
            let b: Vec<_> = (0..n).map(|_| rng.random(D)).collect();
            let expected = a
                .solve_system_of_linear_equations(&b)
                .map(|sol| sol.particular);
            let result = a.black_box_linear_equation(b);
            assert_eq!(result, expected);
        }
    }
}
