use super::{Field, Invertible, Ring, SemiRing};
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
    fn eliminate<const DETERMINANT: bool>(&mut self) -> (usize, R::T) {
        let (n, m) = self.shape;
        let mut rank = 0;
        let mut determinant = R::one();
        let mut negative = false;
        for first in (0..m).step_by(64) {
            if rank == n {
                break;
            }
            let end = (first + 64).min(m);
            let start = rank;
            let mut pivots = Vec::new();
            let mut panel = vec![vec![R::zero(); end - first]; end - first];
            for col in first..end {
                if panel[col - first][..col - first]
                    .iter()
                    .any(|x| !R::is_zero(x))
                {
                    for row in &mut self.data[rank..] {
                        let value =
                            R::dot_product(&row[first..col], &panel[col - first][..col - first]);
                        R::sub_assign(&mut row[col], &value);
                    }
                }
                let Some(pivot) = (rank..n).find(|&i| !R::is_zero(&self[i][col])) else {
                    continue;
                };
                if pivot != rank {
                    self.data.swap(rank, pivot);
                    negative = !negative;
                }
                if DETERMINANT {
                    R::mul_assign(&mut determinant, &self[rank][col]);
                }
                let inv = R::inv(&self[rank][col]);
                let row = &mut self.data[rank];
                for c in col + 1..end {
                    let value = R::dot_product(&row[first..col], &panel[c - first][..col - first]);
                    R::sub_assign(&mut row[c], &value);
                    panel[c - first][col - first] = row[c].clone();
                }
                for row in &mut self.data[rank + 1..] {
                    R::mul_assign(&mut row[col], &inv);
                }
                pivots.push(col);
                rank += 1;
                if rank == n {
                    break;
                }
            }
            for i in start..if rank - start < 32 { n } else { rank } {
                let (upper, lower) = self.data.split_at_mut(i);
                let row = &mut lower[0];
                for (j, &col) in pivots[..(i - start).min(pivots.len())].iter().enumerate() {
                    if R::is_zero(&row[col]) {
                        continue;
                    }
                    let factor = R::neg(&row[col]);
                    R::add_scaled_assign(&mut row[end..], &upper[start + j][end..], &factor);
                }
            }
            if rank < n
                && end < m
                && rank - start >= 32
                && self.data[rank..]
                    .iter()
                    .any(|row| pivots.iter().any(|&col| !R::is_zero(&row[col])))
            {
                let lower = Self::new_with((n - rank, rank - start), |i, j| {
                    R::neg(&self[rank + i][pivots[j]])
                });
                let upper = Self::new_with((rank - start, m - end), |i, j| {
                    self[start + i][end + j].clone()
                });
                let update = &lower * &upper;
                for (row, update) in self.data[rank..].iter_mut().zip(update.data) {
                    for (x, y) in row[end..].iter_mut().zip(update) {
                        R::add_assign(x, &y);
                    }
                }
            }
            for (i, &col) in pivots.iter().enumerate() {
                for row in &mut self.data[start + i + 1..] {
                    row[col] = R::zero();
                }
            }
            if DETERMINANT && rank < end {
                return (rank, R::zero());
            }
        }
        if DETERMINANT && negative {
            determinant = R::neg(&determinant);
        }
        (rank, determinant)
    }

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
                for value in &mut self[r][c..m] {
                    R::mul_assign(value, &d);
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
        self.eliminate::<false>().0
    }

    pub fn determinant(&mut self) -> R::T {
        assert_eq!(self.shape.0, self.shape.1);
        self.eliminate::<true>().1
    }

    pub fn solve_system_of_linear_equations(
        &self,
        b: &[R::T],
    ) -> Option<SystemOfLinearEquationsSolution<R>> {
        assert_eq!(self.shape.0, b.len());
        let m = self.shape.1;
        let mut a = Self::new_with((self.shape.0, m + 1), |i, j| {
            if j == m {
                b[i].clone()
            } else {
                self[i][j].clone()
            }
        });
        let rank = a.eliminate::<false>().0;
        let mut pivots = Vec::with_capacity(rank);
        let mut b = Vec::with_capacity(rank);
        for row in &a.data[..rank] {
            let c = row.iter().position(|x| !R::is_zero(x)).unwrap();
            if c == m {
                return None;
            }
            pivots.push(c);
            b.push(row[m].clone());
        }

        let mut free = Vec::with_capacity(m - rank);
        let mut pivot = 0;
        for c in 0..m {
            if pivot < rank && pivots[pivot] == c {
                pivot += 1;
            } else {
                free.push(c);
            }
        }
        let mut coefficients: Vec<Vec<_>> = (0..rank)
            .map(|i| free.iter().map(|&c| a[i][c].clone()).collect())
            .collect();
        for k in (0..rank).rev() {
            let c = pivots[k];
            let inv = R::inv(&a[k][c]);
            R::mul_assign(&mut b[k], &inv);
            let pivot_b = b[k].clone();
            let (upper, lower) = coefficients.split_at_mut(k);
            let pivot_coefficients = &mut lower[0];
            for x in pivot_coefficients.iter_mut() {
                R::mul_assign(x, &inv);
            }
            for ((row, value), coefficients) in a.data[..k].iter_mut().zip(&mut b[..k]).zip(upper) {
                if R::is_zero(&row[c]) {
                    continue;
                }
                let factor = row[c].clone();
                row[c] = R::zero();
                R::sub_assign(value, &R::mul(&factor, &pivot_b));
                R::add_scaled_assign(coefficients, pivot_coefficients, &R::neg(&factor));
            }
        }

        let mut particular = vec![R::zero(); m];
        for i in 0..rank {
            particular[pivots[i]] = b[i].clone();
        }
        let mut basis = Vec::with_capacity(free.len());
        for (j, &c) in free.iter().enumerate() {
            let mut vector = vec![R::zero(); m];
            vector[c] = R::one();
            for i in 0..rank {
                vector[pivots[i]] = R::neg(&coefficients[i][j]);
            }
            basis.push(vector);
        }
        Some(SystemOfLinearEquationsSolution { particular, basis })
    }

    pub fn inverse(&self) -> Option<Matrix<R>> {
        assert_eq!(self.shape.0, self.shape.1);
        let n = self.shape.0;
        if n >= 64 {
            let m = n / 2;
            let a = Self::new_with((m, m), |i, j| self[i][j].clone());
            if let Some(mut ai) = a.inverse() {
                let b = Self::new_with((m, n - m), |i, j| self[i][j + m].clone());
                let c = Self::new_with((n - m, m), |i, j| self[i + m][j].clone());
                let mut d = Self::new_with((n - m, n - m), |i, j| self[i + m][j + m].clone());
                let u = &ai * &b;
                let v = &c * &ai;
                d -= &v * &b;
                let di = d.inverse()?;
                let r = &u * &di;
                let t = &di * &v;
                ai += &r * &v;
                let mut inverse = Self::zeros((n, n));
                for i in 0..m {
                    inverse[i][..m].clone_from_slice(&ai[i]);
                    for (x, y) in inverse[i][m..].iter_mut().zip(&r[i]) {
                        *x = R::neg(y);
                    }
                }
                for i in m..n {
                    for (x, y) in inverse[i][..m].iter_mut().zip(&t[i - m]) {
                        *x = R::neg(y);
                    }
                    inverse[i][m..].clone_from_slice(&di[i - m]);
                }
                return Some(inverse);
            }
        }
        let mut a = self.clone();
        let mut inverse = Self::eye((n, n));
        let mut ranges: Vec<_> = (0..n).map(|i| (i, i + 1)).collect();
        for r in 0..n {
            let pivot = (r..n).find(|&i| !R::is_zero(&a[i][r]))?;
            a.data.swap(r, pivot);
            inverse.data.swap(r, pivot);
            ranges.swap(r, pivot);

            let d = R::inv(&a[r][r]);
            for x in &mut a[r][r..] {
                R::mul_assign(x, &d);
            }
            let (left, right) = ranges[r];
            for x in &mut inverse[r][left..right] {
                R::mul_assign(x, &d);
            }

            let (a_upper, a_lower) = a.data.split_at_mut(r + 1);
            let pivot_a = &a_upper[r];
            let (inverse_upper, inverse_lower) = inverse.data.split_at_mut(r + 1);
            let pivot_inverse = &inverse_upper[r];
            let (ranges_upper, ranges_lower) = ranges.split_at_mut(r + 1);
            let (left, right) = ranges_upper[r];
            for ((a, inverse), range) in a_lower.iter_mut().zip(inverse_lower).zip(ranges_lower) {
                if R::is_zero(&a[r]) {
                    continue;
                }
                let e = a[r].clone();
                a[r] = R::zero();
                R::add_scaled_assign(&mut a[(r + 1)..], &pivot_a[(r + 1)..], &R::neg(&e));
                R::add_scaled_assign(
                    &mut inverse[left..right],
                    &pivot_inverse[left..right],
                    &R::neg(&e),
                );
                range.0 = range.0.min(left);
                range.1 = range.1.max(right);
            }
        }
        for r in (0..n).rev() {
            let (left, right) = ranges[r];
            let (inverse_upper, inverse_lower) = inverse.data.split_at_mut(r);
            let pivot_inverse = &inverse_lower[0];
            let (ranges_upper, _) = ranges.split_at_mut(r);
            for ((a, inverse), range) in a.data[..r].iter_mut().zip(inverse_upper).zip(ranges_upper)
            {
                if R::is_zero(&a[r]) {
                    continue;
                }
                let e = a[r].clone();
                a[r] = R::zero();
                R::add_scaled_assign(
                    &mut inverse[left..right],
                    &pivot_inverse[left..right],
                    &R::neg(&e),
                );
                range.0 = range.0.min(left);
                range.1 = range.1.max(right);
            }
        }
        Some(inverse)
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
                    R::add_scaled_assign(&mut a[j..], &src[j..], &R::neg(&mul));
                    v.push(mul);
                }
                self[j + 1] = src;
                for a in self.data.iter_mut() {
                    let v = R::dot_product(&a[(j + 2)..], &v);
                    R::add_assign(&mut a[j + 1], &v);
                }
            }
        }
        // dp[k][j - k] stores [x^k] det(xI - A[..j, ..j]).
        let mut dp: Vec<Vec<R::T>> = (0..=n).map(|i| Vec::with_capacity(n + 1 - i)).collect();
        dp[0].push(R::one());
        for i in 0..n {
            let mut c = vec![R::zero(); i + 1];
            c[i] = R::neg(&self[i][i]);
            let mut mul = R::one();
            for j in (0..i).rev() {
                mul = R::mul(&mul, &self[j + 1][j]);
                c[j] = R::neg(&R::mul(&mul, &self[j][i]));
            }
            for k in (0..=i).rev() {
                let mut value = R::dot_product(&dp[k], &c[k..]);
                if k > 0 {
                    R::add_assign(&mut value, dp[k - 1].last().unwrap());
                }
                dp[k].push(value);
            }
            dp[i + 1].push(R::one());
        }
        dp.into_iter().map(|mut c| c.pop().unwrap()).collect()
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
        if let Some(data) = R::try_matrix_product(&self.data, &rhs.data) {
            return Matrix::from_vec(data);
        }
        let rhs = rhs.transpose();
        Matrix::new_with((self.shape.0, rhs.shape.0), |i, j| {
            R::dot_product(&self[i], &rhs[j])
        })
    }
}

fn strassen_rec<R: Ring>(
    a: &[R::T],
    b: &[R::T],
    c: &mut [R::T],
    shape: (usize, usize, usize),
    stride_a: usize,
    stride_b: usize,
) {
    let (n, m, p) = shape;
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

    if n.min(m).min(p) <= 128 {
        let transposed: Vec<_> = (0..p)
            .flat_map(|j| (0..m).map(move |i| b[i * stride_b + j].clone()))
            .collect();
        for (a, c) in a.chunks(stride_a).zip(c.chunks_exact_mut(p)) {
            for (b, c) in transposed.chunks_exact(m).zip(c) {
                *c = R::dot_product(&a[..m], b);
            }
        }
        return;
    }
    let (h, k, w) = (n / 2, m / 2, p / 2);
    let a11 = 0;
    let a12 = k;
    let a21 = h * stride_a;
    let a22 = a21 + k;
    let b11 = 0;
    let b12 = w;
    let b21 = k * stride_b;
    let b22 = b21 + w;

    let block = h * w;
    let mut buf = vec![R::zero(); h * k + k * w + block * 7];
    let (s1, rest) = buf.split_at_mut(h * k);
    let (s2, m_buf) = rest.split_at_mut(k * w);
    let (m1, rest) = m_buf.split_at_mut(block);
    let (m2, rest) = rest.split_at_mut(block);
    let (m3, rest) = rest.split_at_mut(block);
    let (m4, rest) = rest.split_at_mut(block);
    let (m5, rest) = rest.split_at_mut(block);
    let (m6, m7) = rest.split_at_mut(block);

    // (A11 + A22)(B11 + B22)
    add_block::<R>(&a[a11..], &a[a22..], s1, k, stride_a, stride_a);
    add_block::<R>(&b[b11..], &b[b22..], s2, w, stride_b, stride_b);
    strassen_rec::<R>(s1, s2, m1, (h, k, w), k, w);

    // (A21 + A22) B11
    add_block::<R>(&a[a21..], &a[a22..], s1, k, stride_a, stride_a);
    strassen_rec::<R>(s1, &b[b11..], m2, (h, k, w), k, stride_b);

    // A11 (B12 - B22)
    sub_block::<R>(&b[b12..], &b[b22..], s2, w, stride_b, stride_b);
    strassen_rec::<R>(&a[a11..], s2, m3, (h, k, w), stride_a, w);

    // A22 (B21 - B11)
    sub_block::<R>(&b[b21..], &b[b11..], s2, w, stride_b, stride_b);
    strassen_rec::<R>(&a[a22..], s2, m4, (h, k, w), stride_a, w);

    // (A11 + A12) B22
    add_block::<R>(&a[a11..], &a[a12..], s1, k, stride_a, stride_a);
    strassen_rec::<R>(s1, &b[b22..], m5, (h, k, w), k, stride_b);

    // (A21 - A11)(B11 + B12)
    sub_block::<R>(&a[a21..], &a[a11..], s1, k, stride_a, stride_a);
    add_block::<R>(&b[b11..], &b[b12..], s2, w, stride_b, stride_b);
    strassen_rec::<R>(s1, s2, m6, (h, k, w), k, w);

    // (A12 - A22)(B21 + B22)
    sub_block::<R>(&a[a12..], &a[a22..], s1, k, stride_a, stride_a);
    add_block::<R>(&b[b21..], &b[b22..], s2, w, stride_b, stride_b);
    strassen_rec::<R>(s1, s2, m7, (h, k, w), k, w);

    let c11 = 0;
    let c12 = w;
    let c21 = h * p;
    let c22 = c21 + w;
    for ((((m1, m4), m5), m7), c) in m1
        .iter()
        .zip(m4.iter())
        .zip(m5.iter())
        .zip(m7.iter())
        .zip(c[c11..].chunks_mut(p).flat_map(|c| c.iter_mut().take(w)))
    {
        *c = R::add(m1, m4);
        R::sub_assign(c, m5);
        R::add_assign(c, m7);
    }
    for ((m3, m5), c) in m3
        .iter()
        .zip(m5.iter())
        .zip(c[c12..].chunks_mut(p).flat_map(|c| c.iter_mut().take(w)))
    {
        *c = R::add(m3, m5);
    }
    for ((m2, m4), c) in m2
        .iter()
        .zip(m4.iter())
        .zip(c[c21..].chunks_mut(p).flat_map(|c| c.iter_mut().take(w)))
    {
        *c = R::add(m2, m4);
    }
    for ((((m1, m2), m3), m6), c) in m1
        .iter()
        .zip(m2.iter())
        .zip(m3.iter())
        .zip(m6.iter())
        .zip(c[c22..].chunks_mut(p).flat_map(|c| c.iter_mut().take(w)))
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
        if let Some(data) = R::try_matrix_product(&self.data, &rhs.data) {
            return Matrix::from_vec(data);
        }
        let (n, m) = self.shape;
        let p = rhs.shape.1;
        if n == 0 || m == 0 || p == 0 {
            return Matrix::zeros((n, p));
        }
        let split = n.min(m).min(p).div_ceil(128).next_power_of_two();
        if split <= 2 {
            return self * rhs;
        }
        let rows = n.div_ceil(split) * split;
        let inner = m.div_ceil(split) * split;
        let cols = p.div_ceil(split) * split;
        let mut a = vec![R::zero(); rows * inner];
        for (a, data) in a.chunks_exact_mut(inner).zip(&self.data) {
            a[..m].clone_from_slice(data);
        }
        let mut b = vec![R::zero(); inner * cols];
        for (b, data) in b.chunks_exact_mut(cols).zip(&rhs.data) {
            b[..p].clone_from_slice(data);
        }
        let mut c = vec![R::zero(); rows * cols];
        strassen_rec::<R>(&a, &b, &mut c, (rows, inner, cols), inner, cols);
        let mut res = Matrix::zeros((n, p));
        for (data, c) in res.data.iter_mut().zip(c.chunks_exact(cols)) {
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
        } else if rng.gen_bool(0.5) {
            let mut mat = Matrix::new_with(shape, |_, _| rng.random(..));
            let i0 = rng.random(0..shape.0);
            let i1 = rng.random(0..shape.0);
            let x: DynMIntU32 = rng.random(..);
            for j in 0..shape.1 {
                mat[(i0, j)] = mat[(i1, j)] * x;
            }
            mat
        } else {
            let mut rows: Vec<_> = (0..shape.0).collect();
            let mut cols: Vec<_> = (0..shape.1).collect();
            rng.shuffle(&mut rows);
            rng.shuffle(&mut cols);
            let mut mat = Matrix::zeros(shape);
            for (&i, &j) in rows.iter().zip(&cols) {
                mat[i][j] = DynMIntU32::one();
            }
            mat
        }
    }

    #[test]
    fn test_eye() {
        for (n, m) in (0..=32).flat_map(|n| (0..=32).map(move |m| (n, m))) {
            let result = Matrix::<R>::eye((n, m));
            let expected = Matrix::<R>::new_with((n, m), |i, j| DynMIntU32::from((i == j) as u32));
            assert_eq!(result, expected);
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
        for _ in 0..12 {
            rand!(rng, n: 257..520, m: 257..520, l: 257..520);
            let a = Matrix::<R>::new_with((n, m), |_, _| rng.random(..));
            let b = Matrix::<R>::new_with((m, l), |_, _| rng.random(..));
            let bt = b.transpose();
            let expected = Matrix::new_with((n, l), |i, j| R::dot_product(&a[i], &bt[j]));
            assert_eq!(&a * &b, expected);
            assert_eq!(a.mul_strassen(&b), expected);
        }
    }

    #[test]
    fn test_row_reduction() {
        const Q: usize = 1000;
        let mut rng = Xorshift::default();
        let ps = [2, 3, 1_000_000_007];
        for iteration in 0..Q {
            let m = ps[rng.random(..ps.len())];
            DynMIntU32::set_mod(m);
            let n = if iteration < 12 {
                rng.random(128..260)
            } else {
                rng.random(2..=30)
            };
            let mat = Matrix::<R>::new_with((n, n), |_, _| rng.random(..));
            let rank = mat.clone().rank();
            let inv = mat.inverse();
            assert_eq!(rank == n, inv.is_some());
            if let Some(inv) = inv {
                assert_eq!(&mat * &inv, Matrix::eye((n, n)));
            }
        }
        for _ in 0..100 {
            let m = ps[rng.random(..ps.len())];
            DynMIntU32::set_mod(m);
            let shape = (rng.random(1..=30), rng.random(1..=30));
            let mat = random_matrix(&mut rng, shape);
            let mut reduced = mat.clone();
            reduced.row_reduction(false);
            let expected = reduced
                .data
                .iter()
                .filter(|row| row.iter().any(|x| !R::is_zero(x)))
                .count();
            assert_eq!(mat.clone().rank(), expected);
        }
    }

    #[test]
    fn test_determinant() {
        let mut rng = Xorshift::new_with_seed(358224);
        let ps = [2, 3, 1_000_000_007];
        for iteration in 0..300 {
            DynMIntU32::set_mod(ps[rng.random(..ps.len())]);
            let n = if iteration < 24 {
                rng.random(128..260)
            } else {
                rng.random(0..32)
            };
            let mut mat = Matrix::<R>::new_with((n, n), |i, j| {
                if i <= j {
                    rng.random(..)
                } else {
                    DynMIntU32::zero()
                }
            });
            if n != 0 && rng.gen_bool(0.5) {
                let col = rng.random(..n);
                for row in &mut mat.data {
                    row[col] = DynMIntU32::zero();
                }
            }
            let mut expected: DynMIntU32 = (0..n).map(|i| mat[i][i]).product();
            for _ in 0..3 * n {
                let i = rng.random(..n);
                let j = rng.random(..n);
                if i == j {
                    continue;
                }
                if rng.gen_bool(0.5) {
                    mat.data.swap(i, j);
                    expected = -expected;
                } else {
                    let factor: DynMIntU32 = rng.random(..);
                    for k in 0..n {
                        let x = mat[j][k] * factor;
                        mat[i][k] += x;
                    }
                }
            }
            let mut reduced = mat.clone();
            reduced.row_reduction(false);
            let rank = reduced
                .data
                .iter()
                .filter(|row| row.iter().any(|x| !R::is_zero(x)))
                .count();
            assert_eq!(mat.determinant(), expected);
            assert_eq!(mat.rank(), rank);
        }
    }

    #[test]
    fn test_system_of_linear_equations() {
        let mut rng = Xorshift::new_with_seed(746182);
        let ps = [2, 3, 1_000_000_007];
        for iteration in 0..300 {
            DynMIntU32::set_mod(ps[rng.random(..ps.len())]);
            let (n, m): (usize, usize) = if iteration < 24 {
                (rng.random(96..200), rng.random(96..200))
            } else {
                (rng.random(0..32), rng.random(0..32))
            };
            let r = rng.random(0..=n.min(m));
            let mut a = &Matrix::<R>::new_with((n, r), |_, _| rng.random(..))
                * &Matrix::new_with((r, m), |_, _| rng.random(..));
            for j in 0..m {
                if rng.gen_bool(0.1) {
                    for row in &mut a.data {
                        row[j] = DynMIntU32::zero();
                    }
                }
            }
            let b: Vec<DynMIntU32> = if rng.gen_bool(0.5) {
                let x: Vec<DynMIntU32> = rand_value!(rng, [..; m]);
                a.data.iter().map(|row| R::dot_product(row, &x)).collect()
            } else {
                rand_value!(rng, [..; n])
            };
            let mut reduced = a.clone();
            reduced.add_col_with(|i, _| b[i]);
            reduced.row_reduction(true);
            let rank = reduced
                .data
                .iter()
                .filter(|row| row[..m].iter().any(|x| !x.is_zero()))
                .count();
            let solvable = reduced
                .data
                .iter()
                .all(|row| row[m].is_zero() || row[..m].iter().any(|x| !x.is_zero()));
            let solution = a.solve_system_of_linear_equations(&b);
            assert_eq!(solution.is_some(), solvable);
            if let Some(sol) = solution {
                assert_eq!(sol.basis.len(), m - rank);
                for (row, expected) in a.data.iter().zip(&b) {
                    assert_eq!(R::dot_product(row, &sol.particular), *expected);
                    for vector in &sol.basis {
                        assert!(R::dot_product(row, vector).is_zero());
                    }
                }
                let mut basis = Matrix::<R>::from_vec(sol.basis);
                basis.row_reduction(true);
                assert_eq!(
                    basis
                        .data
                        .iter()
                        .filter(|row| row.iter().any(|x| !x.is_zero()))
                        .count(),
                    m - rank
                );
            }
        }
        DynMIntU32::set_mod(1_000_000_007);
    }
}
