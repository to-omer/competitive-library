use super::{
    AddMulOperation, DotProduct, MInt, MIntBase, MIntConvert, Matrix, MemorizedFactorial, One,
    Xorshift, Zero,
};

pub trait MIntMatrix<M>
where
    M: MIntBase,
{
    /// det(self + other * x)
    fn determinant_linear(self, other: Self) -> Option<Vec<MInt<M>>>
    where
        M: MIntConvert<usize> + MIntConvert<u64>;

    /// Computes `self^k` using Frobenius normal form.
    fn pow_frobenius(self, k: usize) -> Self
    where
        M: MIntConvert<u64>;
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
        &(&f.t_inv * &fk) * &f.t
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

struct EchelonRow<M>
where
    M: MIntBase,
{
    pivot: usize,
    inv: MInt<M>,
    row: Vec<MInt<M>>,
}

struct Polynomial<M>(Vec<MInt<M>>)
where
    M: MIntBase;

struct FrobeniusDecomposition<M>
where
    M: MIntBase,
{
    t: Matrix<AddMulOperation<MInt<M>>>,
    t_inv: Matrix<AddMulOperation<MInt<M>>>,
    blocks: Vec<Polynomial<M>>,
}

impl<M> EchelonRow<M>
where
    M: MIntBase,
{
    fn reduce(&self, row: &mut [MInt<M>]) {
        let a = -row[self.pivot] * self.inv;
        if a.is_zero() {
            return;
        }
        for (x, &y) in row[self.pivot..].iter_mut().zip(&self.row[self.pivot..]) {
            *x += a * y;
        }
    }
}

fn generate_frobenius_block<M>(
    a: &Matrix<AddMulOperation<MInt<M>>>,
    mut v: Vec<MInt<M>>,
    rows: &mut Vec<EchelonRow<M>>,
    t: &mut Vec<Vec<MInt<M>>>,
) -> Polynomial<M>
where
    M: MIntBase,
{
    let n = a.shape.0;
    loop {
        let mut row = vec![MInt::zero(); 2 * n + 1];
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
    M: MIntBase,
{
    fn exact_div(mut self, rhs: &Self) -> Option<Self> {
        let mut q = vec![MInt::zero(); self.0.len() - rhs.0.len() + 1];
        let inv = rhs.0.last().unwrap().inv();
        for i in (0..q.len()).rev() {
            q[i] = self.0[i + rhs.0.len() - 1] * inv;
            for (x, &y) in self.0[i..].iter_mut().zip(&rhs.0) {
                *x -= q[i] * y;
            }
        }
        self.0.iter().all(|x| x.is_zero()).then_some(Self(q))
    }

    fn mul_mod(&self, rhs: &Self, p: &Self) -> Self {
        let d = p.0.len() - 1;
        let mut c = vec![MInt::zero(); 2 * d - 1];
        for (i, &x) in self.0.iter().enumerate() {
            for (z, &y) in c[i..].iter_mut().zip(&rhs.0) {
                *z += x * y;
            }
        }
        for i in (d..c.len()).rev() {
            let x = c[i];
            for (y, &m) in c[i - d..].iter_mut().zip(&p.0) {
                *y -= x * m;
            }
        }
        c.truncate(d);
        Self(c)
    }

    fn x_pow_mod(&self, mut k: usize) -> Self {
        let d = self.0.len() - 1;
        let mut r = Self(vec![MInt::zero(); d]);
        r.0[0] = MInt::one();
        let mut x = Self(vec![MInt::zero(); d]);
        if d == 1 {
            x.0[0] = -self.0[0];
        } else {
            x.0[1] = MInt::one();
        }
        while k > 0 {
            if k & 1 != 0 {
                r = r.mul_mod(&x, self);
            }
            k >>= 1;
            if k > 0 {
                x = x.mul_mod(&x, self);
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
    M: MIntBase + MIntConvert<u64>,
{
    let n = a.shape.0;
    let mut rows = Vec::with_capacity(n);
    let mut t = Vec::with_capacity(n);
    let mut blocks = Vec::new();
    while rows.len() < n {
        let s = rows.len();
        let v = (0..n).map(|_| MInt::from(rng.rand64())).collect();
        let mut c = generate_frobenius_block(a, v, &mut rows, &mut t);
        if rows.len() == s {
            continue;
        }
        if c.0[..s].iter().any(|x| !x.is_zero()) {
            let p = Polynomial(c.0[s..].to_vec());
            let q = c.exact_div(&p)?;
            let mut v = t[s].clone();
            for (&x, u) in q.0.iter().zip(&t[..s]) {
                for (v, &u) in v.iter_mut().zip(u) {
                    *v += x * u;
                }
            }
            rows.truncate(s);
            t.truncate(s);
            c = generate_frobenius_block(a, v, &mut rows, &mut t);
        }
        blocks.push(Polynomial(c.0[s..].to_vec()));
    }

    for i in 0..n {
        let (left, right) = rows.split_at_mut(i + 1);
        for row in right {
            row.reduce(&mut left[i].row);
        }
    }
    let mut t_inv = vec![vec![MInt::zero(); n]; n];
    for row in rows {
        let (_, c) = row.row.split_at(n);
        for (x, &y) in t_inv[row.pivot].iter_mut().zip(c) {
            *x = row.inv * y;
        }
    }
    Some(FrobeniusDecomposition {
        t: Matrix::from_vec(t),
        t_inv: Matrix::from_vec(t_inv),
        blocks,
    })
}

impl<M> FrobeniusDecomposition<M>
where
    M: MIntBase,
{
    fn pow(&self, k: usize) -> Matrix<AddMulOperation<MInt<M>>> {
        let n = self.t.shape.0;
        let mut a = vec![vec![MInt::zero(); n]; n];
        let mut s = 0;
        for p in &self.blocks {
            let d = p.0.len() - 1;
            let mut c = p.x_pow_mod(k).0;
            c.resize(d, MInt::zero());
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
        for _ in 0..100 {
            rand!(rng, n: 1..30, k: 0..30, data: [[0..998244353; n]; n]);
            let matrix = Matrix::<AddMulOperation<_>>::from_vec(data)
                .map::<AddMulOperation<MInt998244353>, _>(|&x| MInt998244353::new(x));
            assert_eq!(matrix.clone().pow(k), matrix.pow_frobenius(k));

            let scalar: MInt998244353 = rng.random(..);
            let matrix: Matrix<AddMulOperation<MInt998244353>> =
                Matrix::new_with((n, n), |i, j| if i == j { scalar } else { MInt::zero() });
            assert_eq!(matrix.clone().pow(k), matrix.pow_frobenius(k));
        }
    }
}
