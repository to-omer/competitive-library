use super::{Approx, ApproxOrd, Complex, Zero};
use std::{
    cmp::Ordering,
    ops::{Add, Mul, Sub},
};

pub trait Ccwable:
    ApproxOrd + Copy + Zero + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
}

impl Ccwable for i8 {}
impl Ccwable for i16 {}
impl Ccwable for i32 {}
impl Ccwable for i64 {}
impl Ccwable for i128 {}
impl Ccwable for isize {}
impl Ccwable for f32 {}
impl Ccwable for f64 {}

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
    pub fn new<T>(a: Complex<T>, b: Complex<T>, c: Complex<T>) -> Self
    where
        T: Ccwable,
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
    pub fn new_open<T>(a: Complex<T>, b: Complex<T>, c: Complex<T>) -> Self
    where
        T: Ccwable,
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
