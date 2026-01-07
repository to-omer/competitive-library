use super::{Field, Invertible, Ring, SemiRing, SerdeByteStr};
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign},
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
    R: SemiRing<T: Debug>,
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
    R: SemiRing<T: PartialEq>,
{
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape && self.data == other.data
    }
}

impl<R> Eq for Matrix<R> where R: SemiRing<T: Eq> {}

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
        let shape = (data.len(), data.first().map(Vec::len).unwrap_or_default());
        assert!(data.iter().all(|r| r.len() == shape.1));
        Self {
            shape,
            data,
            _marker: PhantomData,
        }
    }

    pub fn new_with(shape: (usize, usize), mut f: impl FnMut(usize, usize) -> R::T) -> Self {
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
        for (i, d) in data.iter_mut().enumerate().take(shape.1) {
            d[i] = R::one();
        }
        Self {
            shape,
            data,
            _marker: PhantomData,
        }
    }

    pub fn transpose(&self) -> Self {
        Self::new_with((self.shape.1, self.shape.0), |i, j| self[j][i].clone())
    }

    pub fn map<S, F>(&self, mut f: F) -> Matrix<S>
    where
        S: SemiRing,
        F: FnMut(&R::T) -> S::T,
    {
        Matrix::<S>::new_with(self.shape, |i, j| f(&self[i][j]))
    }

    pub fn add_row_with(&mut self, mut f: impl FnMut(usize, usize) -> R::T) {
        self.data
            .push((0..self.shape.1).map(|j| f(self.shape.0, j)).collect());
        self.shape.0 += 1;
    }

    pub fn add_col_with(&mut self, mut f: impl FnMut(usize, usize) -> R::T) {
        for i in 0..self.shape.0 {
            self.data[i].push(f(i, self.shape.1));
        }
        self.shape.1 += 1;
    }

    pub fn pairwise_assign<F>(&mut self, other: &Self, mut f: F)
    where
        F: FnMut(&mut R::T, &R::T),
    {
        assert_eq!(self.shape, other.shape);
        for i in 0..self.shape.0 {
            for j in 0..self.shape.1 {
                f(&mut self[i][j], &other[i][j]);
            }
        }
    }
}

#[derive(Debug)]
pub struct SystemOfLinearEquationsSolution<R>
where
    R: Field<Additive: Invertible, Multiplicative: Invertible>,
{
    pub particular: Vec<R::T>,
    pub basis: Vec<Vec<R::T>>,
}

impl<R> Matrix<R>
where
    R: Field<T: PartialEq, Additive: Invertible, Multiplicative: Invertible>,
{
    /// f: (row, pivot_row, col)
    pub fn row_reduction_with<F>(&mut self, normalize: bool, mut f: F)
    where
        F: FnMut(usize, usize, usize),
    {
        let (n, m) = self.shape;
        let mut c = 0;
        for r in 0..n {
            loop {
                if c >= m {
                    return;
                }
                if let Some(pivot) = (r..n).find(|&p| !R::is_zero(&self[p][c])) {
                    f(r, pivot, c);
                    self.data.swap(r, pivot);
                    break;
                };
                c += 1;
            }
            let d = R::inv(&self[r][c]);
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

    pub fn row_reduction(&mut self, normalize: bool) {
        self.row_reduction_with(normalize, |_, _, _| {});
    }

    pub fn rank(&mut self) -> usize {
        let n = self.shape.0;
        self.row_reduction(false);
        (0..n)
            .filter(|&i| !self.data[i].iter().all(|x| R::is_zero(x)))
            .count()
    }

    pub fn determinant(&mut self) -> R::T {
        assert_eq!(self.shape.0, self.shape.1);
        let mut neg = false;
        self.row_reduction_with(false, |r, p, _| neg ^= r != p);
        let mut d = R::one();
        if neg {
            d = R::neg(&d);
        }
        for i in 0..self.shape.0 {
            R::mul_assign(&mut d, &self[i][i]);
        }
        d
    }

    pub fn solve_system_of_linear_equations(
        &self,
        b: &[R::T],
    ) -> Option<SystemOfLinearEquationsSolution<R>> {
        assert_eq!(self.shape.0, b.len());
        let (n, m) = self.shape;
        let mut c = Matrix::<R>::zeros((n, m + 1));
        for i in 0..n {
            c[i][..m].clone_from_slice(&self[i]);
            c[i][m] = b[i].clone();
        }
        let mut reduced = vec![!0; m + 1];
        c.row_reduction_with(true, |r, _, c| reduced[c] = r);
        if reduced[m] != !0 {
            return None;
        }
        let mut particular = vec![R::zero(); m];
        let mut basis = vec![];
        for j in 0..m {
            if reduced[j] != !0 {
                particular[j] = c[reduced[j]][m].clone();
            } else {
                let mut v = vec![R::zero(); m];
                v[j] = R::one();
                for i in 0..m {
                    if reduced[i] != !0 {
                        R::sub_assign(&mut v[i], &c[reduced[i]][j]);
                    }
                }
                basis.push(v);
            }
        }
        Some(SystemOfLinearEquationsSolution { particular, basis })
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
        if (0..n).any(|i| R::is_zero(&c[i][i])) {
            None
        } else {
            Some(Self::from_vec(
                c.data.into_iter().map(|r| r[n..].to_vec()).collect(),
            ))
        }
    }

    pub fn characteristic_polynomial(&mut self) -> Vec<R::T> {
        let n = self.shape.0;
        if n == 0 {
            return vec![R::one()];
        }
        assert!(self.data.iter().all(|a| a.len() == n));
        for j in 0..(n - 1) {
            if let Some(x) = ((j + 1)..n).find(|&x| !R::is_zero(&self[x][j])) {
                self.data.swap(j + 1, x);
                self.data.iter_mut().for_each(|a| a.swap(j + 1, x));
                let inv = R::inv(&self[j + 1][j]);
                let mut v = vec![];
                let src = std::mem::take(&mut self[j + 1]);
                for a in self.data[(j + 2)..].iter_mut() {
                    let mul = R::mul(&a[j], &inv);
                    for (a, src) in a[j..].iter_mut().zip(src[j..].iter()) {
                        R::sub_assign(a, &R::mul(&mul, src));
                    }
                    v.push(mul);
                }
                self[j + 1] = src;
                for a in self.data.iter_mut() {
                    let v = a[(j + 2)..]
                        .iter()
                        .zip(v.iter())
                        .fold(R::zero(), |s, a| R::add(&s, &R::mul(a.0, a.1)));
                    R::add_assign(&mut a[j + 1], &v);
                }
            }
        }
        let mut dp = vec![vec![R::one()]];
        for i in 0..n {
            let mut next = vec![R::zero(); i + 2];
            for (j, dp) in dp[i].iter().enumerate() {
                R::sub_assign(&mut next[j], &R::mul(dp, &self[i][i]));
                R::add_assign(&mut next[j + 1], dp);
            }
            let mut mul = R::one();
            for j in (0..i).rev() {
                mul = R::mul(&mul, &self[j + 1][j]);
                let c = R::mul(&mul, &self[j][i]);
                for (next, dp) in next.iter_mut().zip(dp[j].iter()) {
                    R::sub_assign(next, &R::mul(&c, dp));
                }
            }
            dp.push(next);
        }
        dp.pop().unwrap()
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

macro_rules! impl_matrix_pairwise_binop {
    ($imp:ident, $method:ident, $imp_assign:ident, $method_assign:ident $(where [$($clauses:tt)*])?) => {
        impl<R> $imp_assign for Matrix<R>
        where
            R: SemiRing,
            $($($clauses)*)?
        {
            fn $method_assign(&mut self, rhs: Self) {
                self.pairwise_assign(&rhs, |a, b| R::$method_assign(a, b));
            }
        }
        impl<R> $imp_assign<&Matrix<R>> for Matrix<R>
        where
            R: SemiRing,
            $($($clauses)*)?
        {
            fn $method_assign(&mut self, rhs: &Self) {
                self.pairwise_assign(rhs, |a, b| R::$method_assign(a, b));
            }
        }
        impl<R> $imp for Matrix<R>
        where
            R: SemiRing,
            $($($clauses)*)?
        {
            type Output = Matrix<R>;
            fn $method(mut self, rhs: Self) -> Self::Output {
                self.$method_assign(rhs);
                self
            }
        }
        impl<R> $imp<&Matrix<R>> for Matrix<R>
        where
            R: SemiRing,
            $($($clauses)*)?
        {
            type Output = Matrix<R>;
            fn $method(mut self, rhs: &Self) -> Self::Output {
                self.$method_assign(rhs);
                self
            }
        }
        impl<R> $imp<Matrix<R>> for &Matrix<R>
        where
            R: SemiRing,
            $($($clauses)*)?
        {
            type Output = Matrix<R>;
            fn $method(self, mut rhs: Matrix<R>) -> Self::Output {
                rhs.pairwise_assign(self, |a, b| *a = R::$method(b, a));
                rhs
            }
        }
        impl<R> $imp<&Matrix<R>> for &Matrix<R>
        where
            R: SemiRing,
            $($($clauses)*)?
        {
            type Output = Matrix<R>;
            fn $method(self, rhs: &Matrix<R>) -> Self::Output {
                let mut this = self.clone();
                this.$method_assign(rhs);
                this
            }
        }
    };
}

impl_matrix_pairwise_binop!(Add, add, AddAssign, add_assign);
impl_matrix_pairwise_binop!(Sub, sub, SubAssign, sub_assign where [R: SemiRing<Additive: Invertible>]);

impl<R> Mul for Matrix<R>
where
    R: SemiRing,
{
    type Output = Matrix<R>;
    fn mul(self, rhs: Self) -> Self::Output {
        (&self).mul(&rhs)
    }
}
impl<R> Mul<&Matrix<R>> for Matrix<R>
where
    R: SemiRing,
{
    type Output = Matrix<R>;
    fn mul(self, rhs: &Matrix<R>) -> Self::Output {
        (&self).mul(rhs)
    }
}
impl<R> Mul<Matrix<R>> for &Matrix<R>
where
    R: SemiRing,
{
    type Output = Matrix<R>;
    fn mul(self, rhs: Matrix<R>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<R> Mul<&Matrix<R>> for &Matrix<R>
where
    R: SemiRing,
{
    type Output = Matrix<R>;
    fn mul(self, rhs: &Matrix<R>) -> Self::Output {
        assert_eq!(self.shape.1, rhs.shape.0);
        let mut res = Matrix::zeros((self.shape.0, rhs.shape.1));
        for (a, c) in self.data.iter().zip(res.data.iter_mut()) {
            for (a, b) in a.iter().zip(rhs.data.iter()) {
                for (b, c) in b.iter().zip(c.iter_mut()) {
                    R::add_assign(c, &R::mul(a, b));
                }
            }
        }
        res
    }
}

fn strassen_rec<R: Ring>(
    a: &[R::T],
    b: &[R::T],
    c: &mut [R::T],
    n: usize,
    stride_a: usize,
    stride_b: usize,
) {
    fn add_block<R: Ring>(
        a: &[R::T],
        b: &[R::T],
        out: &mut [R::T],
        n: usize,
        stride_a: usize,
        stride_b: usize,
    ) {
        for ((a, b), c) in a
            .chunks(stride_a)
            .zip(b.chunks(stride_b))
            .zip(out.chunks_exact_mut(n))
        {
            for ((a, b), c) in a.iter().zip(b.iter()).zip(c.iter_mut()) {
                *c = R::add(a, b);
            }
        }
    }

    fn sub_block<R: Ring>(
        a: &[R::T],
        b: &[R::T],
        out: &mut [R::T],
        n: usize,
        stride_a: usize,
        stride_b: usize,
    ) {
        for ((a, b), c) in a
            .chunks(stride_a)
            .zip(b.chunks(stride_b))
            .zip(out.chunks_exact_mut(n))
        {
            for ((a, b), c) in a.iter().zip(b.iter()).zip(c.iter_mut()) {
                *c = R::sub(a, b);
            }
        }
    }

    if n <= 64 {
        for (a, c) in a.chunks(stride_a).zip(c.chunks_exact_mut(n)) {
            for (a, b) in a.iter().zip(b.chunks(stride_b)).take(n) {
                for (b, c) in b.iter().zip(c.iter_mut()) {
                    R::add_assign(c, &R::mul(a, b));
                }
            }
        }
        return;
    }
    let h = n / 2;
    let a11 = 0;
    let a12 = h;
    let a21 = h * stride_a;
    let a22 = a21 + h;
    let b11 = 0;
    let b12 = h;
    let b21 = h * stride_b;
    let b22 = b21 + h;

    let block = h * h;
    let mut buf = vec![R::zero(); block * 9];
    let (s_buf, m_buf) = buf.split_at_mut(block * 2);
    let (s1, s2) = s_buf.split_at_mut(block);
    let (m1, rest) = m_buf.split_at_mut(block);
    let (m2, rest) = rest.split_at_mut(block);
    let (m3, rest) = rest.split_at_mut(block);
    let (m4, rest) = rest.split_at_mut(block);
    let (m5, rest) = rest.split_at_mut(block);
    let (m6, m7) = rest.split_at_mut(block);

    // (A11 + A22)(B11 + B22)
    add_block::<R>(&a[a11..], &a[a22..], s1, h, stride_a, stride_a);
    add_block::<R>(&b[b11..], &b[b22..], s2, h, stride_b, stride_b);
    strassen_rec::<R>(s1, s2, m1, h, h, h);

    // (A21 + A22) B11
    add_block::<R>(&a[a21..], &a[a22..], s1, h, stride_a, stride_a);
    strassen_rec::<R>(s1, &b[b11..], m2, h, h, stride_b);

    // A11 (B12 - B22)
    sub_block::<R>(&b[b12..], &b[b22..], s1, h, stride_b, stride_b);
    strassen_rec::<R>(&a[a11..], s1, m3, h, stride_a, h);

    // A22 (B21 - B11)
    sub_block::<R>(&b[b21..], &b[b11..], s1, h, stride_b, stride_b);
    strassen_rec::<R>(&a[a22..], s1, m4, h, stride_a, h);

    // (A11 + A12) B22
    add_block::<R>(&a[a11..], &a[a12..], s1, h, stride_a, stride_a);
    strassen_rec::<R>(s1, &b[b22..], m5, h, h, stride_b);

    // (A21 - A11)(B11 + B12)
    sub_block::<R>(&a[a21..], &a[a11..], s1, h, stride_a, stride_a);
    add_block::<R>(&b[b11..], &b[b12..], s2, h, stride_b, stride_b);
    strassen_rec::<R>(s1, s2, m6, h, h, h);

    // (A12 - A22)(B21 + B22)
    sub_block::<R>(&a[a12..], &a[a22..], s1, h, stride_a, stride_a);
    add_block::<R>(&b[b21..], &b[b22..], s2, h, stride_b, stride_b);
    strassen_rec::<R>(s1, s2, m7, h, h, h);

    let c11 = 0;
    let c12 = h;
    let c21 = h * n;
    let c22 = c21 + h;
    for ((((m1, m4), m5), m7), c) in m1
        .iter()
        .zip(m4.iter())
        .zip(m5.iter())
        .zip(m7.iter())
        .zip(c[c11..].chunks_mut(n).flat_map(|c| c.iter_mut().take(h)))
    {
        *c = R::add(m1, m4);
        R::sub_assign(c, m5);
        R::add_assign(c, m7);
    }
    for ((m3, m5), c) in m3
        .iter()
        .zip(m5.iter())
        .zip(c[c12..].chunks_mut(n).flat_map(|c| c.iter_mut().take(h)))
    {
        *c = R::add(m3, m5);
    }
    for ((m2, m4), c) in m2
        .iter()
        .zip(m4.iter())
        .zip(c[c21..].chunks_mut(n).flat_map(|c| c.iter_mut().take(h)))
    {
        *c = R::add(m2, m4);
    }
    for ((((m1, m2), m3), m6), c) in m1
        .iter()
        .zip(m2.iter())
        .zip(m3.iter())
        .zip(m6.iter())
        .zip(c[c22..].chunks_mut(n).flat_map(|c| c.iter_mut().take(h)))
    {
        *c = R::sub(m1, m2);
        R::add_assign(c, m3);
        R::add_assign(c, m6);
    }
}

impl<R> Matrix<R>
where
    R: Ring,
{
    pub fn mul_strassen(&self, rhs: &Matrix<R>) -> Matrix<R> {
        assert_eq!(self.shape.1, rhs.shape.0);
        let (n, m) = self.shape;
        let p = rhs.shape.1;
        if n == 0 || m == 0 || p == 0 {
            return Matrix::zeros((n, p));
        }
        let max_dim = n.max(m).max(p);
        if max_dim <= 64 {
            return self * rhs;
        }
        let size = max_dim.next_power_of_two();
        let mut a = vec![R::zero(); size * size];
        for (a, data) in a.chunks_exact_mut(size).zip(&self.data) {
            a[..m].clone_from_slice(data);
        }
        let mut b = vec![R::zero(); size * size];
        for (b, data) in b.chunks_exact_mut(size).zip(&rhs.data) {
            b[..p].clone_from_slice(data);
        }
        let mut c = vec![R::zero(); size * size];
        strassen_rec::<R>(&a, &b, &mut c, size, size, size);
        let mut res = Matrix::zeros((n, p));
        for (data, c) in res.data.iter_mut().zip(c.chunks_exact(size)) {
            data.clone_from_slice(&c[..p]);
        }
        res
    }
}

impl<R> MulAssign<&R::T> for Matrix<R>
where
    R: SemiRing,
{
    fn mul_assign(&mut self, rhs: &R::T) {
        for i in 0..self.shape.0 {
            for j in 0..self.shape.1 {
                R::mul_assign(&mut self[(i, j)], rhs);
            }
        }
    }
}

impl<R> Neg for Matrix<R>
where
    R: SemiRing<Additive: Invertible>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.map(|x| R::neg(x))
    }
}

impl<R> Neg for &Matrix<R>
where
    R: SemiRing<Additive: Invertible>,
{
    type Output = Matrix<R>;

    fn neg(self) -> Self::Output {
        self.map(|x| R::neg(x))
    }
}

impl<R> Matrix<R>
where
    R: SemiRing,
{
    pub fn pow(self, mut n: usize) -> Self {
        assert_eq!(self.shape.0, self.shape.1);
        let mut res = Matrix::eye(self.shape);
        let mut x = self;
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
    R: Ring,
{
    pub fn pow_strassen(self, mut n: usize) -> Self {
        assert_eq!(self.shape.0, self.shape.1);
        let mut res = Matrix::eye(self.shape);
        let mut x = self;
        while n > 0 {
            if n & 1 == 1 {
                res = res.mul_strassen(&x);
            }
            x = x.mul_strassen(&x);
            n >>= 1;
        }
        res
    }
}

impl<R> SerdeByteStr for Matrix<R>
where
    R: SemiRing<T: SerdeByteStr>,
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
        num::{One, Zero, mint_basic::DynMIntU32},
        rand, rand_value,
        tools::Xorshift,
    };

    type R = AddMulOperation<DynMIntU32>;

    fn random_matrix(rng: &mut Xorshift, shape: (usize, usize)) -> Matrix<R> {
        if rng.gen_bool(0.5) {
            Matrix::new_with(shape, |_, _| rng.random(..))
        } else if rng.gen_bool(0.5) {
            let r = rng.randf();
            Matrix::new_with(shape, |_, _| {
                if rng.gen_bool(r) {
                    rng.random(..)
                } else {
                    DynMIntU32::zero()
                }
            })
        } else {
            let mut mat = Matrix::new_with(shape, |_, _| rng.random(..));
            let i0 = rng.random(0..shape.0);
            let i1 = rng.random(0..shape.0);
            let x: DynMIntU32 = rng.random(..);
            for j in 0..shape.1 {
                mat[(i0, j)] = mat[(i1, j)] * x;
            }
            mat
        }
    }

    #[test]
    fn test_eye() {
        for n in 0..10 {
            for m in 0..10 {
                let result = Matrix::<R>::eye((n, m));
                let expected = Matrix::<R>::new_with((n, m), |i, j| {
                    if i == j {
                        DynMIntU32::one()
                    } else {
                        DynMIntU32::zero()
                    }
                });
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_add() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30, m: 1..30);
            let a = Matrix::<R>::new_with((n, m), |_, _| rng.random(..));
            let b = Matrix::<R>::new_with((n, m), |_, _| rng.random(..));
            assert_eq!(&a + &b, a.clone() + b.clone());
            assert_eq!(a.clone() + &b, a.clone() + b.clone());
            assert_eq!(&a + b.clone(), a.clone() + b.clone());
        }
    }

    #[test]
    fn test_sub() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30, m: 1..30);
            let a = Matrix::<R>::new_with((n, m), |_, _| rng.random(..));
            let b = Matrix::<R>::new_with((n, m), |_, _| rng.random(..));
            assert_eq!(&a - &b, a.clone() - b.clone());
            assert_eq!(a.clone() - &b, a.clone() - b.clone());
            assert_eq!(&a - b.clone(), a.clone() - b.clone());
        }
    }

    #[test]
    fn test_mul() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30, m: 1..30, l: 1..30);
            let a = Matrix::<R>::new_with((n, m), |_, _| rng.random(..));
            let b = Matrix::<R>::new_with((m, l), |_, _| rng.random(..));
            assert_eq!(&a * &b, a.clone() * b.clone());
            assert_eq!(a.clone() * &b, a.clone() * b.clone());
            assert_eq!(&a * b.clone(), a.clone() * b.clone());
            assert_eq!(
                &a * &b,
                Matrix::new_with((n, l), |i, j| (0..m).map(|k| a[i][k] * b[k][j]).sum())
            );
            let c = rng.random(..);
            let mut ac = a.clone();
            ac *= &c;
            assert_eq!(ac, Matrix::new_with(a.shape, |i, j| a[i][j] * c));
        }
    }

    #[test]
    fn test_row_reduction() {
        const Q: usize = 1000;
        let mut rng = Xorshift::default();
        let ps = [2, 3, 1_000_000_007];
        for _ in 0..Q {
            let m = ps[rng.random(..ps.len())];
            DynMIntU32::set_mod(m);
            let n = rng.random(2..=30);
            let mat = Matrix::<R>::new_with((n, n), |_, _| rng.random(..));
            let rank = mat.clone().rank();
            let inv = mat.inverse();
            assert_eq!(rank == n, inv.is_some());
            if let Some(inv) = inv {
                assert_eq!(&mat * &inv, Matrix::eye((n, n)));
            }
        }
    }

    #[test]
    fn test_system_of_linear_equations() {
        const Q: usize = 1000;
        let mut rng = Xorshift::default();
        let ps = [2, 3, 1_000_000_007];
        for _ in 0..Q {
            let p = ps[rng.random(..ps.len())];
            DynMIntU32::set_mod(p);
            let n = rng.random(1..=30);
            let m = rng.random(1..=30);
            let a = random_matrix(&mut rng, (n, m));
            let b = random_matrix(&mut rng, (1, n))
                .data
                .into_iter()
                .next()
                .unwrap();
            if let Some(sol) = a.solve_system_of_linear_equations(&b) {
                assert_eq!(
                    &a * Matrix::from_vec(vec![sol.particular.clone()]).transpose(),
                    Matrix::from_vec(vec![b.clone()]).transpose()
                );
                let c: Vec<DynMIntU32> = rand_value!(rng, [..; sol.basis.len()]);
                let mut x = sol.particular.clone();
                for (c, v) in c.iter().zip(sol.basis.iter()) {
                    for (x, v) in x.iter_mut().zip(v.iter()) {
                        *x += *c * *v;
                    }
                }
                assert_eq!(
                    &a * Matrix::from_vec(vec![x]).transpose(),
                    Matrix::from_vec(vec![b]).transpose()
                );
            }
        }
    }
}
