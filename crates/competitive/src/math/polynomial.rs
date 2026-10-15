use crate::num::{One, Zero};

#[codesnip::entry("Polynomial", include("zero_one"))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Polynomial<T> {
    pub data: Vec<T>,
}
#[codesnip::entry("Polynomial")]
mod polynomial_impls {
    use super::*;
    use std::ops::{Add, Div, Index, IndexMut, Mul, Rem, Sub};
    impl<T> Polynomial<T> {
        pub fn from_vec(data: Vec<T>) -> Self {
            Self { data }
        }
        pub fn length(&self) -> usize {
            self.data.len()
        }
    }
    impl<T> Zero for Polynomial<T> {
        fn zero() -> Self {
            Self::from_vec(Vec::new())
        }
    }
    impl<T> One for Polynomial<T>
    where
        T: One,
    {
        fn one() -> Self {
            Self::from_vec(vec![One::one()])
        }
    }
    impl<T: Clone + Zero + Add<Output = T> + Mul<Output = T>> Polynomial<T> {
        pub fn assign(&self, x: T) -> T {
            let mut res = Zero::zero();
            for c in self.data.iter().rev().cloned() {
                res = res * x.clone() + c;
            }
            res
        }
    }
    impl<T> Index<usize> for Polynomial<T> {
        type Output = T;
        fn index(&self, index: usize) -> &Self::Output {
            &self.data[index]
        }
    }
    impl<T> IndexMut<usize> for Polynomial<T> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.data[index]
        }
    }
    impl<T: Copy + Add<Output = T>> Add<&Polynomial<T>> for &Polynomial<T> {
        type Output = Polynomial<T>;
        fn add(self, rhs: &Polynomial<T>) -> Self::Output {
            let (x, y) = if self.length() < rhs.length() {
                (rhs, self)
            } else {
                (self, rhs)
            };
            let mut x = x.clone();
            for j in 0..y.length() {
                x[j] = x[j] + y[j];
            }
            x
        }
    }
    impl<T> Sub<&Polynomial<T>> for &Polynomial<T>
    where
        T: Copy + Zero + Sub<Output = T>,
    {
        type Output = Polynomial<T>;
        fn sub(self, rhs: &Polynomial<T>) -> Self::Output {
            let mut x = self.clone();
            x.data.resize(self.length().max(rhs.length()), Zero::zero());
            for (a, &b) in x.data.iter_mut().zip(&rhs.data) {
                *a = *a - b;
            }
            x
        }
    }
    impl<T: Copy + Zero + Add<Output = T> + Mul<Output = T>> Mul<&Polynomial<T>> for &Polynomial<T> {
        type Output = Polynomial<T>;
        fn mul(self, rhs: &Polynomial<T>) -> Self::Output {
            if self.data.is_empty() || rhs.data.is_empty() {
                return Polynomial::zero();
            }
            let mut res =
                Polynomial::from_vec(vec![Zero::zero(); self.length() + rhs.length() - 1]);
            for i in 0..self.length() {
                for j in 0..rhs.length() {
                    res[i + j] = res[i + j] + self[i] * rhs[j];
                }
            }
            res
        }
    }
    impl<T: Copy + Zero + Sub<Output = T> + Mul<Output = T> + Div<Output = T>> Div<&Polynomial<T>>
        for &Polynomial<T>
    {
        type Output = Polynomial<T>;
        fn div(self, rhs: &Polynomial<T>) -> Self::Output {
            let mut x = self.clone();
            let mut res = Polynomial::from_vec(vec![]);
            for i in (rhs.length() - 1..x.length()).rev() {
                let t = x[i] / rhs[rhs.length() - 1];
                res.data.push(t);
                for j in 0..rhs.length() {
                    x[i - j] = x[i - j] - t * rhs[rhs.length() - 1 - j];
                }
            }
            res.data.reverse();
            res
        }
    }
    impl<T: Copy + Zero + Sub<Output = T> + Mul<Output = T> + Div<Output = T>> Rem<&Polynomial<T>>
        for &Polynomial<T>
    {
        type Output = Polynomial<T>;
        fn rem(self, rhs: &Polynomial<T>) -> Self::Output {
            let mut x = self.clone();
            for i in (rhs.length() - 1..x.length()).rev() {
                let t = x[i] / rhs[rhs.length() - 1];
                for j in 0..rhs.length() {
                    x[i - j] = x[i - j] - t * rhs[rhs.length() - 1 - j];
                }
            }
            x.data.truncate(rhs.length() - 1);
            x
        }
    }
    impl<T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>> Polynomial<T> {
        pub fn pow(&self, mut n: usize) -> Self {
            let mut x = self.clone();
            let mut res = Self::one();
            while n > 0 {
                if n & 1 == 1 {
                    res = &res * &x;
                }
                n >>= 1;
                if n != 0 {
                    x = &x * &x;
                }
            }
            res
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        num::mint_basic::MInt998244353,
        tools::{
            Xorshift,
            testutil::{integer_cases, sequence_cases},
        },
    };

    #[test]
    fn test_sub() {
        let mut rng = Xorshift::default();
        let cases: Vec<_> = sequence_cases(&mut rng, -1_000_000_000i64..=1_000_000_000, 0..=2048)
            .map(Polynomial::from_vec)
            .collect();
        for lhs in &cases {
            for rhs in &cases {
                let actual = lhs - rhs;
                assert_eq!(actual.length(), lhs.length().max(rhs.length()));
                for (i, &coefficient) in actual.data.iter().enumerate() {
                    let a = lhs.data.get(i).copied().unwrap_or(0);
                    let b = rhs.data.get(i).copied().unwrap_or(0);
                    assert_eq!(coefficient, a - b);
                }
            }
        }
    }

    #[test]
    fn test_mul_zero() {
        let mut rng = Xorshift::default();
        for values in sequence_cases(&mut rng, -4i64..=4, 0..=32) {
            let polynomial = Polynomial::from_vec(values);
            let zero = Polynomial::zero();
            assert!((&polynomial * &zero).data.is_empty());
            assert!((&zero * &polynomial).data.is_empty());
        }
    }

    #[test]
    fn test_pow() {
        for x in integer_cases(i64::MIN..=i64::MAX) {
            for exponent in 0..=i64::BITS {
                if let Some(expected) = x.checked_pow(exponent) {
                    let polynomial = Polynomial::from_vec(vec![x]);
                    assert_eq!(polynomial.pow(exponent as usize).data, vec![expected]);
                }
            }
        }
        let mut input_rng = Xorshift::default();
        for values in sequence_cases(&mut input_rng, -4i64..=4, 0..=32) {
            let coefficients: Vec<_> = values.into_iter().map(MInt998244353::from).collect();
            let polynomial = Polynomial::from_vec(coefficients.clone());
            let mut expected = vec![MInt998244353::one()];
            for exponent in 0..=16 {
                if exponent != 0 {
                    if coefficients.is_empty() {
                        expected.clear();
                    } else {
                        let mut product =
                            vec![MInt998244353::zero(); expected.len() + coefficients.len() - 1];
                        for (i, &a) in expected.iter().enumerate() {
                            for (j, &b) in coefficients.iter().enumerate() {
                                product[i + j] += a * b;
                            }
                        }
                        expected = product;
                    }
                }
                assert_eq!(polynomial.pow(exponent).data, expected);
            }
        }
    }
}
