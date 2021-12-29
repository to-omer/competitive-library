use super::*;
use std::{
    iter::repeat_with,
    iter::{once, FromIterator},
    ops::{Index, IndexMut},
};

impl<T, Multiplier> FormalPowerSeries<T, Multiplier> {
    pub fn from_vec(data: Vec<T>) -> Self {
        Self {
            data,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn length(&self) -> usize {
        self.data.len()
    }
    pub fn truncate(&mut self, deg: usize) {
        self.data.truncate(deg)
    }
}

impl<T: Clone, Multiplier> Clone for FormalPowerSeries<T, Multiplier> {
    fn clone(&self) -> Self {
        Self::from_vec(self.data.clone())
    }
}
impl<T: PartialEq, Multiplier> PartialEq for FormalPowerSeries<T, Multiplier> {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}
impl<T: PartialEq, Multiplier> Eq for FormalPowerSeries<T, Multiplier> {}

impl<T, Multiplier> FormalPowerSeries<T, Multiplier>
where
    T: Zero,
{
    pub fn zeros(deg: usize) -> Self {
        repeat_with(T::zero).take(deg).collect()
    }
    pub fn resize(&mut self, deg: usize) {
        self.data.resize_with(deg, Zero::zero)
    }
}

impl<T: Clone, Multiplier> FormalPowerSeries<T, Multiplier> {
    pub fn prefix(&self, deg: usize) -> Self {
        if deg < self.length() {
            Self::from_vec(self.data[..deg].to_vec())
        } else {
            self.clone()
        }
    }
    pub fn even(&self) -> Self {
        self.data.iter().cloned().step_by(2).collect()
    }
    pub fn odd(&self) -> Self {
        self.data.iter().cloned().skip(1).step_by(2).collect()
    }
}

impl<T, Multiplier> Zero for FormalPowerSeries<T, Multiplier>
where
    T: PartialEq,
{
    fn zero() -> Self {
        Self::from_vec(Vec::new())
    }
}
impl<T, Multiplier> One for FormalPowerSeries<T, Multiplier>
where
    T: PartialEq + One,
{
    fn one() -> Self {
        Self::from(T::one())
    }
}

impl<T, Multiplier> FromIterator<T> for FormalPowerSeries<T, Multiplier> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl<T, Multiplier> Index<usize> for FormalPowerSeries<T, Multiplier> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<T, Multiplier> IndexMut<usize> for FormalPowerSeries<T, Multiplier> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T, Multiplier> From<T> for FormalPowerSeries<T, Multiplier> {
    fn from(x: T) -> Self {
        once(x).collect()
    }
}
impl<T, Multiplier> From<Vec<T>> for FormalPowerSeries<T, Multiplier> {
    fn from(data: Vec<T>) -> Self {
        Self::from_vec(data)
    }
}

impl<T, Multiplier> FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    pub fn diff(&self) -> Self {
        self.data
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, x)| x.clone() * T::from(i))
            .collect()
    }
    pub fn integral(&self) -> Self {
        once(T::zero())
            .chain(
                self.data
                    .iter()
                    .enumerate()
                    .map(|(i, x)| x.clone() / T::from(i + 1)),
            )
            .collect()
    }
    pub fn eval(&self, x: T) -> T {
        let mut base = T::one();
        let mut res = T::zero();
        for a in &self.data {
            res += base.clone() * a.clone();
            base *= x.clone();
        }
        res
    }
}

impl<T, Multiplier> FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    pub fn inv(&self, deg: usize) -> Self {
        debug_assert!(!self[0].is_zero());
        let mut f = Self::from(T::one() / self[0].clone());
        let mut i = 1;
        while i < deg {
            f = (&f + &f - &f * &f * self.prefix(i * 2)).prefix(i * 2);
            i *= 2;
        }
        f.prefix(deg)
    }
    pub fn exp(&self, deg: usize) -> Self {
        debug_assert!(self[0].is_zero());
        let mut f = Self::one();
        let mut i = 1;
        while i < deg {
            f = (&f * &(self.prefix(i * 2) + &T::one() - f.log(i * 2))).prefix(i * 2);
            i *= 2;
        }
        f.prefix(deg)
    }
    pub fn log(&self, deg: usize) -> Self {
        (self.diff() * self.inv(deg)).integral().prefix(deg)
    }
    pub fn pow(&self, rhs: usize, deg: usize) -> Self {
        if let Some(k) = self.data.iter().position(|x| !x.is_zero()) {
            if k * rhs >= deg {
                Self::zeros(deg)
            } else {
                let mut x0 = self[k].clone();
                let rev = T::one() / x0.clone();
                let x = {
                    let mut x = T::one();
                    let mut y = rhs;
                    while y > 0 {
                        if y & 1 == 1 {
                            x *= x0.clone();
                        }
                        x0 *= x0.clone();
                        y >>= 1;
                    }
                    x
                };
                let mut f = (self.clone() * &rev) >> k;
                f = (f.log(deg) * &T::from(rhs)).exp(deg) * &x;
                f.truncate(deg - k * rhs);
                f <<= k * rhs;
                f
            }
        } else {
            Self::zeros(deg)
        }
    }
}

impl<T, Multiplier> FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficientSqrt,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    pub fn sqrt(&self, deg: usize) -> Option<Self> {
        if self[0].is_zero() {
            if let Some(k) = self.data.iter().position(|x| !x.is_zero()) {
                if k % 2 != 0 {
                    return None;
                } else if deg > k / 2 {
                    return Some((self >> k).sqrt(deg - k / 2)? << (k / 2));
                }
            }
        } else {
            let inv2 = T::one() / (T::one() + T::one());
            let mut f = Self::from(self[0].sqrt_coefficient()?);
            let mut i = 1;
            while i < deg {
                f = (&f + &(self.prefix(i * 2) * f.inv(i * 2))).prefix(i * 2) * &inv2;
                i *= 2;
            }
            f.truncate(deg);
            return Some(f);
        }
        Some(Self::zeros(deg))
    }
}

impl<T, Multiplier> FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    pub fn count_subset_sum<F>(&self, deg: usize, mut inverse: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        let n = self.data.len();
        let mut f = Self::zeros(n);
        for i in 1..n {
            if !self[i].is_zero() {
                for (j, d) in (0..n).step_by(i).enumerate().skip(1) {
                    if j & 1 != 0 {
                        f[d] += self[i].clone() * &inverse(j);
                    } else {
                        f[d] -= self[i].clone() * &inverse(j);
                    }
                }
            }
        }
        f.exp(deg)
    }
    pub fn count_multiset_sum<F>(&self, deg: usize, mut inverse: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        let n = self.data.len();
        let mut f = Self::zeros(n);
        for i in 1..n {
            if !self[i].is_zero() {
                for (j, d) in (0..n).step_by(i).enumerate().skip(1) {
                    f[d] += self[i].clone() * &inverse(j);
                }
            }
        }
        f.exp(deg)
    }
    pub fn bostan_mori(&self, rhs: &Self, mut n: usize) -> T {
        let mut p = self.clone();
        let mut q = rhs.clone();
        while n > 0 {
            let mut mq = q.clone();
            mq.data
                .iter_mut()
                .skip(1)
                .step_by(2)
                .for_each(|x| *x = -x.clone());
            let u = p * mq.clone();
            if n % 2 == 0 {
                p = u.even();
            } else {
                p = u.odd();
            }
            q = (q * mq).even();
            n /= 2;
        }
        p[0].clone() / q[0].clone()
    }
    pub fn multipoint_evaluation(&self, points: &[T]) -> Vec<T> {
        let n = points.len();
        if n <= 32 {
            return points.iter().map(|p| self.eval(p.clone())).collect();
        }
        let mut subproduct_tree = Vec::with_capacity(n * 2);
        subproduct_tree.resize_with(n, Zero::zero);
        for x in points {
            subproduct_tree.push(Self::from_vec(vec![-x.clone(), T::one()]));
        }
        for i in (1..n).rev() {
            subproduct_tree[i] = &subproduct_tree[i * 2] * &subproduct_tree[i * 2 + 1];
        }
        let mut subremainder_tree = Vec::with_capacity(n);
        subremainder_tree.resize_with(1, Zero::zero);
        subremainder_tree.push(self % &subproduct_tree[1]);
        for i in 2..n {
            subremainder_tree.push(&subremainder_tree[i / 2] % &subproduct_tree[i]);
        }
        let mut res = Vec::with_capacity(n);
        for i in n..n * 2 {
            let f = &subremainder_tree[i / 2] % &subproduct_tree[i];
            res.push((f).data.get(0).cloned().unwrap_or_else(Zero::zero));
        }
        res
    }
}
