use super::{One, Zero};
use std::{
    mem::swap,
    ops::{Add, Div, Mul, Sub},
};

pub fn berlekamp_massey<T>(a: &[T]) -> Vec<T>
where
    T: Zero
        + One
        + Clone
        + PartialEq
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>,
{
    let n = a.len();
    let mut b = Vec::with_capacity(n + 1);
    let mut c = Vec::with_capacity(n + 1);
    let mut tmp = Vec::with_capacity(n + 1);
    b.push(T::one());
    c.push(T::one());
    let mut y = T::one();
    for k in 1..=n {
        let clen = c.len();
        let mut x = T::zero();
        for (c, a) in c.iter().zip(&a[k - clen..]) {
            x = x + c.clone() * a.clone();
        }
        b.push(T::zero());
        let blen = b.len();
        if x.is_zero() {
            continue;
        }
        let freq = x.clone() / y.clone();
        if clen < blen {
            swap(&mut c, &mut tmp);
            c.clear();
            c.resize_with(blen - clen, T::zero);
            c.extend(tmp.iter().cloned());
            for (c, b) in c.iter_mut().rev().zip(b.iter().rev()) {
                *c = c.clone() - freq.clone() * b.clone();
            }
            swap(&mut b, &mut tmp);
            y = x;
        } else {
            for (c, b) in c.iter_mut().rev().zip(b.iter().rev()) {
                *c = c.clone() - freq.clone() * b.clone();
            }
        }
    }
    c.reverse();
    c
}
