use super::{
    AddMulOperation, DotProduct, MInt, MIntConvert, MIntDotProduct, Matrix, MemorizedFactorial,
    One, Xorshift, Zero,
};

pub trait MIntMatrix<M>
where
    M: MIntDotProduct,
{
    /// det(self + other * x)
    fn determinant_linear(self, other: Self) -> Option<Vec<MInt<M>>>
    where
        M: MIntConvert<usize> + MIntConvert<u64>;

    fn pow_frobenius(self, k: usize) -> Self
    where
        M: MIntConvert<u64>;
}

impl<M> MIntMatrix<M> for Matrix<AddMulOperation<MInt<M>>>
where
    M: MIntDotProduct,
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

    fn pow_frobenius(self, k: usize) -> Self
    where
        M: MIntConvert<u64>,
    {
        assert_eq!(self.shape.0, self.shape.1);
        let a = self.transpose();
        let mut rng = Xorshift::new();
        let f = loop {
            if let Some(f) = frobenius_decomposition(&a, &mut rng) {
                break f;
            }
        };
        let fk = f.pow(k);
        let n = f.t.shape.0;
        if f.blocks
            .iter()
            .map(|p| (p.0.len() - 1).pow(2))
            .sum::<usize>()
            * 4
            <= n * n
        {
            let mut ft = Matrix::zeros((n, n));
            let mut first = 0;
            for p in &f.blocks {
                let d = p.0.len() - 1;
                for i in first..first + d {
                    for j in first..first + d {
                        MInt::add_scaled_assign(&mut ft[i], &f.t[j], &fk[i][j]);
                    }
                }
                first += d;
            }
            &f.t_inv * &ft
        } else {
            &(&f.t_inv * &fk) * &f.t
        }
    }
}

impl<M> Matrix<AddMulOperation<MInt<M>>>
where
    M: MIntDotProduct,
{
    fn determinant_linear_non_singular(mut self, mut other: Self) -> Option<Vec<MInt<M>>>
    where
        M: MIntDotProduct,
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

struct EchelonRow<M>
where
    M: MIntDotProduct,
{
    pivot: usize,
    inv: MInt<M>,
    row: Vec<MInt<M>>,
}

struct Polynomial<M>(Vec<MInt<M>>)
where
    M: MIntDotProduct;

struct FrobeniusDecomposition<M>
where
    M: MIntDotProduct,
{
    t: Matrix<AddMulOperation<MInt<M>>>,
    t_inv: Matrix<AddMulOperation<MInt<M>>>,
    blocks: Vec<Polynomial<M>>,
}

impl<M> EchelonRow<M>
where
    M: MIntDotProduct,
{
    fn reduce(&self, row: &mut [MInt<M>]) {
        let a = -row[self.pivot] * self.inv;
        if a.is_zero() {
            return;
        }
        let end = self.row.len();
        MInt::add_scaled_assign(&mut row[self.pivot..end], &self.row[self.pivot..], &a);
    }
}

fn generate_frobenius_block<M>(
    a: &Matrix<AddMulOperation<MInt<M>>>,
    mut v: Vec<MInt<M>>,
    rows: &mut Vec<EchelonRow<M>>,
    t: &mut Vec<Vec<MInt<M>>>,
) -> Polynomial<M>
where
    M: MIntDotProduct,
{
    let n = a.shape.0;
    loop {
        let mut row = vec![MInt::zero(); n + rows.len() + 1];
        let (x, c) = row.split_at_mut(n);
        x.copy_from_slice(&v);
        c[rows.len()] = MInt::one();
        for r in rows.iter() {
            r.reduce(&mut row);
        }
        if let Some(pivot) = row[..n].iter().position(|x| !x.is_zero()) {
            t.push(v);
            let u = t.last().unwrap();
            v = a.data.iter().map(|row| MInt::dot_product(u, row)).collect();
            rows.push(EchelonRow {
                pivot,
                inv: row[pivot].inv(),
                row,
            });
        } else {
            let mut p = row.split_off(n);
            while p.last().is_some_and(|x| x.is_zero()) {
                p.pop();
            }
            return Polynomial(p);
        }
    }
}

impl<M> Polynomial<M>
where
    M: MIntDotProduct,
{
    fn exact_div(mut self, rhs: &Self) -> Option<Self> {
        let mut q = vec![MInt::zero(); self.0.len() - rhs.0.len() + 1];
        let inv = rhs.0.last().unwrap().inv();
        for i in (0..q.len()).rev() {
            q[i] = self.0[i + rhs.0.len() - 1] * inv;
            MInt::add_scaled_assign(&mut self.0[i..i + rhs.0.len()], &rhs.0, &-q[i]);
        }
        self.0.iter().all(|x| x.is_zero()).then_some(Self(q))
    }

    fn square_mod(&self, p: &Self) -> Self {
        let d = p.0.len() - 1;
        let mut c = vec![MInt::zero(); 2 * d - 1];
        for (i, &x) in self.0.iter().enumerate() {
            MInt::add_scaled_assign(&mut c[i..2 * i], &self.0[..i], &(x + x));
            c[2 * i] += x * x;
        }
        for i in (d..c.len()).rev() {
            let x = c[i];
            MInt::add_scaled_assign(&mut c[i - d..=i], &p.0, &-x);
        }
        c.truncate(d);
        Self(c)
    }

    fn x_pow_mod(&self, k: usize) -> Self {
        let d = self.0.len() - 1;
        if d == 1 {
            return Self(vec![(-self.0[0]).pow(k)]);
        }
        let mut r = Self(vec![MInt::zero(); d]);
        r.0[0] = MInt::one();
        for bit in (0..usize::BITS - k.leading_zeros()).rev() {
            r = r.square_mod(self);
            if k >> bit & 1 != 0 {
                let x = r.0[d - 1];
                for i in (1..d).rev() {
                    r.0[i] = r.0[i - 1] - x * self.0[i];
                }
                r.0[0] = -x * self.0[0];
            }
        }
        r
    }
}

fn frobenius_decomposition<M>(
    a: &Matrix<AddMulOperation<MInt<M>>>,
    rng: &mut Xorshift,
) -> Option<FrobeniusDecomposition<M>>
where
    M: MIntDotProduct + MIntConvert<u64>,
{
    let n = a.shape.0;
    let mut rows = Vec::with_capacity(n);
    let mut t = Vec::with_capacity(n);
    let mut blocks: Vec<Polynomial<M>> = Vec::new();
    while rows.len() < n {
        let s = rows.len();
        let v = (0..n).map(|_| MInt::from(rng.rand64())).collect();
        let c = generate_frobenius_block(a, v, &mut rows, &mut t);
        if rows.len() == s {
            continue;
        }
        let p = Polynomial(c.0[s..].to_vec());
        if c.0[..s].iter().any(|x| !x.is_zero()) {
            let q = c.exact_div(&p)?;
            let d = rows.len() - s;
            let mut coefficients = q.0[..s].to_vec();
            let mut shifts = Vec::with_capacity(d);
            for _ in 0..d {
                shifts.push(coefficients.clone());
                let mut first = 0;
                for block in &blocks {
                    let len = block.0.len() - 1;
                    let c = &mut coefficients[first..first + len];
                    let last = c[len - 1];
                    for j in (1..len).rev() {
                        c[j] = c[j - 1] - last * block.0[j];
                    }
                    c[0] = -last * block.0[0];
                    first += len;
                }
            }
            let shifts: Matrix<AddMulOperation<MInt<M>>> = Matrix::from_vec(shifts);
            if d < 32 {
                let (previous, current) = t.split_at_mut(s);
                for (shift, row) in shifts.data.iter().zip(current) {
                    for (factor, source) in shift.iter().zip(previous.iter()) {
                        if !factor.is_zero() {
                            MInt::add_scaled_assign(row, source, factor);
                        }
                    }
                }
            } else {
                let previous = Matrix::from_vec(t[..s].to_vec());
                let correction = &shifts * &previous;
                for (row, correction) in t[s..].iter_mut().zip(&correction.data) {
                    for (x, &y) in row.iter_mut().zip(correction) {
                        *x += y;
                    }
                }
            }
            for row in &mut rows[s..] {
                // Keep the reduced vector fixed: T_new += S*T_old gives C_old -= C_new*S.
                let (previous, current) = row.row[n..].split_at_mut(s);
                for (&x, shift) in current.iter().zip(&shifts.data) {
                    MInt::add_scaled_assign(previous, shift, &-x);
                }
            }
        }
        blocks.push(p);
    }

    let mut t_inv = vec![vec![MInt::zero(); n]; n];
    for i in (0..n).rev() {
        let row = &rows[i];
        let mut c = row.row[n..].to_vec();
        c.resize(n, MInt::zero());
        for x in &mut c {
            *x *= row.inv;
        }
        for next in &rows[i + 1..] {
            let factor = -row.row[next.pivot] * row.inv;
            if !factor.is_zero() {
                MInt::add_scaled_assign(&mut c, &t_inv[next.pivot], &factor);
            }
        }
        t_inv[row.pivot] = c;
    }
    Some(FrobeniusDecomposition {
        t: Matrix::from_vec(t),
        t_inv: Matrix::from_vec(t_inv),
        blocks,
    })
}

impl<M> FrobeniusDecomposition<M>
where
    M: MIntDotProduct,
{
    fn pow(&self, k: usize) -> Matrix<AddMulOperation<MInt<M>>> {
        let n = self.t.shape.0;
        let mut a = vec![vec![MInt::zero(); n]; n];
        let mut s = 0;
        for p in &self.blocks {
            let d = p.0.len() - 1;
            let mut c = p.x_pow_mod(k).0;
            for row in &mut a[s..s + d] {
                row[s..s + d].copy_from_slice(&c);
                let x = c[d - 1];
                for i in (1..d).rev() {
                    c[i] = c[i - 1] - x * p.0[i];
                }
                c[0] = -x * p.0[0];
            }
            s += d;
        }
        Matrix::from_vec(a)
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

    #[test]
    fn test_pow_frobenius() {
        let mut rng = Xorshift::default();
        for iteration in 0..100 {
            let n = if iteration < 16 {
                rng.random(32..100)
            } else {
                rng.random(0..30)
            };
            let k = rng.random(0..1_000_000_000);
            rand!(rng, data: [[0..998244353; n]; n]);
            let matrix = Matrix::<AddMulOperation<_>>::from_vec(data)
                .map::<AddMulOperation<MInt998244353>, _>(|&x| MInt998244353::new(x));
            assert_eq!(matrix.clone().pow(k), matrix.pow_frobenius(k));

            let scalar: MInt998244353 = rng.random(..);
            let matrix: Matrix<AddMulOperation<MInt998244353>> =
                Matrix::new_with((n, n), |i, j| if i == j { scalar } else { MInt::zero() });
            assert_eq!(matrix.clone().pow(k), matrix.pow_frobenius(k));

            let mut matrix: Matrix<AddMulOperation<MInt998244353>> =
                Matrix::new_with((n, n), |i, j| {
                    if i == j {
                        scalar
                    } else if i + 1 == j && rng.gen_bool(0.8) {
                        MInt::one()
                    } else {
                        MInt::zero()
                    }
                });
            if n >= 2 {
                for _ in 0..4 * n {
                    let i = rng.random(..n);
                    let j = (i + rng.random(1..n)) % n;
                    let factor: MInt998244353 = rng.random(..);
                    for k in 0..n {
                        let x = factor * matrix[j][k];
                        matrix[i][k] += x;
                    }
                    for row in &mut matrix.data {
                        let x = factor * row[i];
                        row[j] -= x;
                    }
                }
            }
            assert_eq!(matrix.clone().pow(k), matrix.pow_frobenius(k));
        }
    }
}
