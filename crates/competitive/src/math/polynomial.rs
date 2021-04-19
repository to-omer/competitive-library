#![allow(clippy::suspicious_arithmetic_impl)]

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
    impl<T: Zero + One> One for Polynomial<T> {
        fn one() -> Self {
            Self::from_vec(vec![Zero::zero(), One::one()])
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
    impl<T: Copy + Sub<Output = T>> Sub<&Polynomial<T>> for &Polynomial<T> {
        type Output = Polynomial<T>;
        fn sub(self, rhs: &Polynomial<T>) -> Self::Output {
            let (x, y) = if self.length() < rhs.length() {
                (rhs, self)
            } else {
                (self, rhs)
            };
            let mut x = x.clone();
            for j in 0..y.length() {
                x[j] = x[j] - y[j];
            }
            x
        }
    }
    impl<T: Copy + Zero + Add<Output = T> + Mul<Output = T>> Mul<&Polynomial<T>> for &Polynomial<T> {
        type Output = Polynomial<T>;
        fn mul(self, rhs: &Polynomial<T>) -> Self::Output {
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
                x = &x * &x;
                n >>= 1;
            }
            res
        }
    }
}
