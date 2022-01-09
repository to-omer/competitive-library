use super::*;
use std::{
    iter::repeat_with,
    iter::{once, FromIterator},
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

impl<T, C> FormalPowerSeries<T, C> {
    pub fn from_vec(data: Vec<T>) -> Self {
        Self {
            data,
            _marker: PhantomData,
        }
    }
    pub fn length(&self) -> usize {
        self.data.len()
    }
    pub fn truncate(&mut self, deg: usize) {
        self.data.truncate(deg)
    }
    pub fn iter(&self) -> Iter<'_, T> {
        self.data.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.data.iter_mut()
    }
}

impl<T, C> Clone for FormalPowerSeries<T, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::from_vec(self.data.clone())
    }
}
impl<T, C> PartialEq for FormalPowerSeries<T, C>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}
impl<T, C> Eq for FormalPowerSeries<T, C> where T: PartialEq {}

impl<T, C> FormalPowerSeries<T, C>
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

impl<T, C> FormalPowerSeries<T, C>
where
    T: Zero + PartialEq,
{
    pub fn trim_tail_zeros(&mut self) {
        let mut len = self.length();
        while len > 0 {
            if self.data[len - 1].is_zero() {
                len -= 1;
            } else {
                break;
            }
        }
        self.truncate(len);
    }
}

impl<T, C> FormalPowerSeries<T, C>
where
    T: Clone,
{
    pub fn prefix(&self, deg: usize) -> Self {
        if deg < self.length() {
            Self::from_vec(self.data[..deg].to_vec())
        } else {
            self.clone()
        }
    }
    pub fn prefix_inplace(mut self, deg: usize) -> Self {
        self.data.truncate(deg);
        self
    }
    pub fn even(&self) -> Self {
        self.iter().cloned().step_by(2).collect()
    }
    pub fn odd(&self) -> Self {
        self.iter().cloned().skip(1).step_by(2).collect()
    }
}

impl<T, C> Zero for FormalPowerSeries<T, C>
where
    T: PartialEq,
{
    fn zero() -> Self {
        Self::from_vec(Vec::new())
    }
}
impl<T, C> One for FormalPowerSeries<T, C>
where
    T: PartialEq + One,
{
    fn one() -> Self {
        Self::from(T::one())
    }
}

impl<T, C> IntoIterator for FormalPowerSeries<T, C> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<T, C> FromIterator<T> for FormalPowerSeries<T, C> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl<T, C> Index<usize> for FormalPowerSeries<T, C> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<T, C> IndexMut<usize> for FormalPowerSeries<T, C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T, C> From<T> for FormalPowerSeries<T, C> {
    fn from(x: T) -> Self {
        once(x).collect()
    }
}
impl<T, C> From<Vec<T>> for FormalPowerSeries<T, C> {
    fn from(data: Vec<T>) -> Self {
        Self::from_vec(data)
    }
}

impl<T, C> FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    pub fn diff(&self) -> Self {
        self.iter()
            .enumerate()
            .skip(1)
            .map(|(i, x)| x.clone() * T::from(i))
            .collect()
    }
    pub fn integral(&self) -> Self {
        once(T::zero())
            .chain(
                self.iter()
                    .enumerate()
                    .map(|(i, x)| x.clone() / T::from(i + 1)),
            )
            .collect()
    }
    pub fn eval(&self, x: T) -> T {
        let mut base = T::one();
        let mut res = T::zero();
        for a in self.iter() {
            res += base.clone() * a.clone();
            base *= x.clone();
        }
        res
    }
}

impl<T, C> FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    pub fn inv(&self, deg: usize) -> Self {
        debug_assert!(!self[0].is_zero());
        let mut f = Self::from(T::one() / self[0].clone());
        let mut i = 1;
        while i < deg {
            // let mut g = self.prefix((i * 2).min(deg));
            // let mut h = f.clone();

            f = (&f + &f - &f * &f * self.prefix(i * 2)).prefix(i * 2);
            i *= 2;
        }
        f.prefix_inplace(deg)
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
        if let Some(k) = self.iter().position(|x| !x.is_zero()) {
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

impl<T, C> FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficientSqrt,
    C: ConvolveSteps<T = Vec<T>>,
{
    pub fn sqrt(&self, deg: usize) -> Option<Self> {
        if self[0].is_zero() {
            if let Some(k) = self.iter().position(|x| !x.is_zero()) {
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

impl<T, C> FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    pub fn count_subset_sum<F>(&self, deg: usize, mut inverse: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        let n = self.length();
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
        let n = self.length();
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
            mq.iter_mut()
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
    fn middle_product(&self, other: &Self) -> Self {
        let n = self.length();
        let mut x = self.clone();
        x.data.reverse();
        let res = &x * other;
        Self::from_vec(res.data[n - 1..].to_vec())
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
        let mut uptree_t = Vec::with_capacity(n * 2);
        uptree_t.resize_with(1, Zero::zero);
        let mut v = subproduct_tree[1].clone();
        v.data.reverse();
        v.data.resize_with(self.length(), Zero::zero);
        v = v.inv(self.length()).middle_product(self);
        v.data.resize_with(n, Zero::zero);
        v.data.reverse();
        uptree_t.push(v);
        for i in 1..n {
            uptree_t.push(
                subproduct_tree[i * 2 + 1]
                    .middle_product(&uptree_t[i])
                    .prefix(subproduct_tree[i * 2].length()),
            );
            uptree_t.push(
                subproduct_tree[i * 2]
                    .middle_product(&uptree_t[i])
                    .prefix(subproduct_tree[i * 2 + 1].length()),
            );
        }
        uptree_t[n..]
            .iter()
            .map(|u| u.data.get(0).cloned().unwrap_or_else(Zero::zero))
            .collect()
    }
}
