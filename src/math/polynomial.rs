use cargo_snippet::snippet;

#[snippet("Polynomial")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Polynomial(pub Vec<i64>, pub i64);
#[snippet("Polynomial")]
impl Polynomial {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
#[snippet("Polynomial")]
pub fn poly_add(x: &Polynomial, y: &Polynomial, p: i64) -> Polynomial {
    let (x, y) = if x.len() < y.len() { (y, x) } else { (x, y) };
    let mut x = x.clone();
    for j in 0..y.len() {
        x[j] = (x[j] + y[j]) % p;
    }
    x
}
#[snippet("Polynomial")]
pub fn poly_sub(x: &Polynomial, y: &Polynomial, p: i64) -> Polynomial {
    let (x, y) = if x.len() < y.len() { (y, x) } else { (x, y) };
    let mut x = x.clone();
    for j in 0..y.len() {
        x[j] = ((x[j] - y[j]) % p + p) % p;
    }
    x
}
#[snippet("Polynomial")]
pub fn poly_mul(x: &Polynomial, y: &Polynomial, p: i64) -> Polynomial {
    let mut res = Polynomial(vec![0; x.len() + y.len() - 1], p);
    for i in 0..x.len() {
        for j in 0..y.len() {
            res[i + j] = (res[i + j] + x[i] * y[j] % p) % p;
        }
    }
    res
}
#[snippet("Polynomial")]
pub fn poly_div(x: &Polynomial, y: &Polynomial, p: i64) -> Polynomial {
    let mut x = x.clone();
    let mut res = Polynomial(vec![], p);
    for i in (y.len() - 1..x.len()).rev() {
        let t = x[i] / y[y.len() - 1];
        res.0.push(t);
        for j in 0..y.len() {
            x[i - j] = ((x[i - j] - t * y[y.len() - 1 - j] % p) % p + p) % p;
        }
    }
    res.0.reverse();
    res
}
#[snippet("Polynomial")]
pub fn poly_mod(x: &Polynomial, y: &Polynomial, p: i64) -> Polynomial {
    let mut x = x.clone();
    for i in (y.len() - 1..x.len()).rev() {
        let t = x[i] / y[y.len() - 1];
        for j in 0..y.len() {
            x[i - j] = ((x[i - j] - t * y[y.len() - 1 - j] % p) % p + p) % p;
        }
    }
    x.0.truncate(y.len() - 1);
    x
}
#[snippet("Polynomial")]
pub fn poly_assign(x: &Polynomial, a: i64, p: i64) -> i64 {
    let mut res = 0;
    for &c in x.0.iter().rev() {
        res = (res * a % p + c) % p;
    }
    res
}
#[snippet("Polynomial")]
impl std::ops::Index<usize> for Polynomial {
    type Output = i64;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
#[snippet("Polynomial")]
impl std::ops::IndexMut<usize> for Polynomial {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
#[snippet("Polynomial")]
impl std::ops::Add<&Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn add(self, rhs: &Polynomial) -> Self::Output {
        poly_add(self, rhs, self.1)
    }
}
#[snippet("Polynomial")]
impl std::ops::Sub<&Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn sub(self, rhs: &Polynomial) -> Self::Output {
        poly_sub(self, rhs, self.1)
    }
}
#[snippet("Polynomial")]
impl std::ops::Mul<&Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn mul(self, rhs: &Polynomial) -> Self::Output {
        poly_mul(self, rhs, self.1)
    }
}
#[snippet("Polynomial")]
impl std::ops::Div<&Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn div(self, rhs: &Polynomial) -> Self::Output {
        poly_div(self, rhs, self.1)
    }
}
#[snippet("Polynomial")]
impl std::ops::Rem<&Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn rem(self, rhs: &Polynomial) -> Self::Output {
        poly_mod(self, rhs, self.1)
    }
}

pub mod poly_mod_poly {
    use super::super::modi64::*;
    pub type Poly = Vec<Modi64>;
    pub fn poly_mulmod(x: &Poly, y: &Poly, z: &Poly) -> Poly {
        let mut res = vec![Modi64(0); x.len() + y.len() - 1];
        for i in 0..x.len() {
            for j in 0..y.len() {
                res[i + j] += x[i] * y[j];
            }
        }
        for i in (z.len() - 1..x.len() + y.len() - 1).rev() {
            let t = res[i] / z[z.len() - 1];
            for j in 0..z.len() {
                res[i - j] -= t * z[z.len() - 1 - j];
            }
        }
        res.truncate(z.len() - 1);
        res
    }
    pub fn poly_pow(x: Poly, y: usize, z: Poly) -> Poly {
        let mut x = x;
        let mut y = y;
        let mut res = vec![Modi64(1)];
        while y > 0 {
            if y & 1 == 1 {
                res = poly_mulmod(&res, &x, &z);
            }
            x = poly_mulmod(&x, &x, &z);
            y >>= 1;
        }
        res
    }
}
