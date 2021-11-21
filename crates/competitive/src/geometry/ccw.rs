use super::{Approx, ApproxOrd, Complex, Zero};
use std::{
    cmp::Ordering,
    ops::{Add, Mul, Sub},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ccw {
    /// a--b--c
    OnlineFront = -2,
    /// a--b-vc
    Clockwise = -1,
    /// a--c--b
    OnSegment = 0,
    /// a--b-^c
    CounterClockwise = 1,
    /// c--a--b
    OnlineBack = 2,
}
impl Ccw {
    pub fn ccw<T>(a: Complex<T>, b: Complex<T>, c: Complex<T>) -> Self
    where
        T: ApproxOrd + Copy + Zero + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
    {
        let x = b - a;
        let y = c - a;
        let zero = T::zero();
        match x.cross(y).approx_cmp(&zero) {
            Ordering::Less => Self::Clockwise,
            Ordering::Greater => Self::CounterClockwise,
            Ordering::Equal => {
                if Approx(x.dot(y)) < Approx(zero) {
                    Self::OnlineBack
                } else if Approx((a - b).dot(c - b)) < Approx(zero) {
                    Self::OnlineFront
                } else {
                    Self::OnSegment
                }
            }
        }
    }
    pub fn ccw_open<T>(a: Complex<T>, b: Complex<T>, c: Complex<T>) -> Self
    where
        T: ApproxOrd + Copy + Zero + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
    {
        let x = b - a;
        let y = c - a;
        let zero = T::zero();
        match x.cross(y).approx_cmp(&zero) {
            Ordering::Less => Self::Clockwise,
            Ordering::Greater => Self::CounterClockwise,
            Ordering::Equal => {
                if Approx(x.dot(y)) <= Approx(zero) {
                    Self::OnlineBack
                } else if Approx((a - b).dot(c - b)) <= Approx(zero) {
                    Self::OnlineFront
                } else {
                    Self::OnSegment
                }
            }
        }
    }
}
