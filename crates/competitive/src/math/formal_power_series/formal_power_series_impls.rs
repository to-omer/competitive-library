use super::*;
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
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
    pub fn resized(mut self, deg: usize) -> Self {
        self.resize(deg);
        self
    }
    pub fn reversed(mut self) -> Self {
        self.data.reverse();
        self
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
impl<'a, T, C> IntoIterator for &'a FormalPowerSeries<T, C> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}
impl<'a, T, C> IntoIterator for &'a mut FormalPowerSeries<T, C> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
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
    pub fn prefix_ref(&self, deg: usize) -> Self {
        if deg < self.length() {
            Self::from_vec(self.data[..deg].to_vec())
        } else {
            self.clone()
        }
    }
    pub fn prefix(mut self, deg: usize) -> Self {
        self.data.truncate(deg);
        self
    }
    pub fn even(mut self) -> Self {
        let mut keep = false;
        self.data.retain(|_| {
            keep = !keep;
            keep
        });
        self
    }
    pub fn odd(mut self) -> Self {
        let mut keep = true;
        self.data.retain(|_| {
            keep = !keep;
            keep
        });
        self
    }
    pub fn diff(mut self) -> Self {
        let mut c = T::one();
        for x in self.iter_mut().skip(1) {
            *x *= &c;
            c += T::one();
        }
        if self.length() > 0 {
            self.data.remove(0);
        }
        self
    }
    pub fn integral(mut self) -> Self {
        let n = self.length();
        self.data.insert(0, Zero::zero());
        let mut fact = Vec::with_capacity(n + 1);
        let mut c = T::one();
        fact.push(c.clone());
        for _ in 1..n {
            fact.push(fact.last().cloned().unwrap() * c.clone());
            c += T::one();
        }
        let mut invf = T::one() / (fact.last().cloned().unwrap() * c.clone());
        for x in self.iter_mut().skip(1).rev() {
            *x *= invf.clone() * fact.pop().unwrap();
            invf *= c.clone();
            c -= T::one();
        }
        self
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
            let g = self.prefix_ref((i * 2).min(deg));
            let h = f.clone();
            let mut g = C::transform(g.data, 2 * i);
            let h = C::transform(h.data, 2 * i);
            C::multiply(&mut g, &h);
            let mut g = Self::from_vec(C::inverse_transform(g, 2 * i));
            g >>= i;
            let mut g = C::transform(g.data, 2 * i);
            C::multiply(&mut g, &h);
            let g = Self::from_vec(C::inverse_transform(g, 2 * i));
            f.data.extend((-g).into_iter().take(i));
            i *= 2;
        }
        f.truncate(deg);
        f
    }
    pub fn exp(&self, deg: usize) -> Self {
        debug_assert!(self[0].is_zero());
        let mut f = Self::one();
        let mut i = 1;
        while i < deg {
            let mut g = -f.log(i * 2);
            g[0] += T::one();
            for (g, x) in g.iter_mut().zip(self.iter().take(i * 2)) {
                *g += x.clone();
            }
            f = (f * g).prefix(i * 2);
            i *= 2;
        }
        f.prefix(deg)
    }
    pub fn log(&self, deg: usize) -> Self {
        (self.inv(deg) * self.clone().diff()).integral().prefix(deg)
    }
    pub fn pow(&self, rhs: usize, deg: usize) -> Self {
        if rhs == 0 {
            return Self::from_vec(
                once(T::one())
                    .chain(repeat_with(T::zero))
                    .take(deg)
                    .collect(),
            );
        }
        if let Some(k) = self.iter().position(|x| !x.is_zero()) {
            if k >= (deg + rhs - 1) / rhs {
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
                f = (&f + &(self.prefix_ref(i * 2) * f.inv(i * 2))).prefix(i * 2) * &inv2;
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
    pub fn bostan_mori(self, rhs: Self, mut n: usize) -> T {
        let mut p = self;
        let mut q = rhs;
        while n > 0 {
            let mut mq = q.clone();
            mq.iter_mut()
                .skip(1)
                .step_by(2)
                .for_each(|x| *x = -x.clone());
            let u = p * mq.clone();
            p = if n % 2 == 0 { u.even() } else { u.odd() };
            q = (q * mq).even();
            n /= 2;
        }
        p[0].clone() / q[0].clone()
    }
    fn middle_product(self, other: &C::F, deg: usize) -> Self {
        let n = self.length();
        let mut s = C::transform(self.reversed().data, deg);
        C::multiply(&mut s, other);
        Self::from_vec((C::inverse_transform(s, deg))[n - 1..].to_vec())
    }
    pub fn multipoint_evaluation(self, points: &[T]) -> Vec<T> {
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
        subproduct_tree.reverse();
        subproduct_tree.pop();
        let m = self.length();
        let v = subproduct_tree.pop().unwrap().reversed().resized(m);
        let s = C::transform(self.data, m * 2);
        uptree_t.push(v.inv(m).middle_product(&s, m * 2).resized(n).reversed());
        for i in 1..n {
            let subl = subproduct_tree.pop().unwrap();
            let subr = subproduct_tree.pop().unwrap();
            let (dl, dr) = (subl.length(), subr.length());
            let len = dl.max(dr) + uptree_t[i].length();
            let s = C::transform(uptree_t[i].data.to_vec(), len);
            uptree_t.push(subr.middle_product(&s, len).prefix(dl));
            uptree_t.push(subl.middle_product(&s, len).prefix(dr));
        }
        uptree_t[n..]
            .iter()
            .map(|u| u.data.first().cloned().unwrap_or_else(Zero::zero))
            .collect()
    }
    pub fn product_all<I>(iter: I, deg: usize) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        let mut heap: BinaryHeap<_> = iter
            .into_iter()
            .map(|f| PartialIgnoredOrd(Reverse(f.length()), f))
            .collect();
        while let Some(PartialIgnoredOrd(_, x)) = heap.pop() {
            if let Some(PartialIgnoredOrd(_, y)) = heap.pop() {
                let z = (x * y).prefix(deg);
                heap.push(PartialIgnoredOrd(Reverse(z.length()), z));
            } else {
                return x;
            }
        }
        Self::one()
    }
    pub fn sum_all_rational<I>(iter: I, deg: usize) -> (Self, Self)
    where
        I: IntoIterator<Item = (Self, Self)>,
    {
        let mut heap: BinaryHeap<_> = iter
            .into_iter()
            .map(|(f, g)| PartialIgnoredOrd(Reverse(f.length().max(g.length())), (f, g)))
            .collect();
        while let Some(PartialIgnoredOrd(_, (xa, xb))) = heap.pop() {
            if let Some(PartialIgnoredOrd(_, (ya, yb))) = heap.pop() {
                let zb = (&xb * &yb).prefix(deg);
                let za = (xa * yb + ya * xb).prefix(deg);
                heap.push(PartialIgnoredOrd(
                    Reverse(za.length().max(zb.length())),
                    (za, zb),
                ));
            } else {
                return (xa, xb);
            }
        }
        (Self::zero(), Self::one())
    }
    pub fn kth_term_of_linearly_recurrence(self, a: Vec<T>, k: usize) -> T {
        if let Some(x) = a.get(k) {
            return x.clone();
        }
        let p = (Self::from_vec(a).prefix(self.length() - 1) * &self).prefix(self.length() - 1);
        p.bostan_mori(self, k)
    }
    pub fn kth_term(a: Vec<T>, k: usize) -> T {
        if let Some(x) = a.get(k) {
            return x.clone();
        }
        Self::from_vec(berlekamp_massey(&a)).kth_term_of_linearly_recurrence(a, k)
    }
    /// sum_i a_i exp(b_i x)
    pub fn linear_sum_of_exp<I, F>(iter: I, deg: usize, mut inv_fact: F) -> Self
    where
        I: IntoIterator<Item = (T, T)>,
        F: FnMut(usize) -> T,
    {
        let (p, q) = Self::sum_all_rational(
            iter.into_iter()
                .map(|(a, b)| (Self::from_vec(vec![a]), Self::from_vec(vec![T::one(), -b]))),
            deg,
        );
        let mut f = (p * q.inv(deg)).prefix(deg);
        for i in 0..f.length() {
            f[i] *= inv_fact(i);
        }
        f
    }
}

impl<M, C> FormalPowerSeries<MInt<M>, C>
where
    M: MIntConvert<usize>,
    C: ConvolveSteps<T = Vec<MInt<M>>>,
{
    /// f(x) <- f(x + a)
    pub fn taylor_shift(mut self, a: MInt<M>, f: &MemorizedFactorial<M>) -> Self {
        let n = self.length();
        for i in 0..n {
            self.data[i] *= f.fact[i];
        }
        self.data.reverse();
        let mut b = a;
        let mut g = Self::from_vec(f.inv_fact[..n].to_vec());
        for i in 1..n {
            g[i] *= b;
            b *= a;
        }
        self *= g;
        self.truncate(n);
        self.data.reverse();
        for i in 0..n {
            self.data[i] *= f.inv_fact[i];
        }
        self
    }
}
