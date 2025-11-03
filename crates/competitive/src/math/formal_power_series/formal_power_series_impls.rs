use super::*;
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    iter::repeat_with,
    iter::{FromIterator, once},
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
    T: Zero + Clone,
{
    pub fn coeff(&self, deg: usize) -> T {
        self.data.get(deg).cloned().unwrap_or_else(T::zero)
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
    pub fn parity_inversion(mut self) -> Self {
        self.iter_mut()
            .skip(1)
            .step_by(2)
            .for_each(|x| *x = -x.clone());
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
        if self.data.iter().filter(|x| !x.is_zero()).count()
            <= deg.next_power_of_two().trailing_zeros() as usize * 6
        {
            let pos: Vec<_> = self
                .data
                .iter()
                .enumerate()
                .skip(1)
                .filter_map(|(i, x)| if x.is_zero() { None } else { Some(i) })
                .collect();
            let mut f = Self::zeros(deg);
            f[0] = T::one() / self[0].clone();
            for i in 1..deg {
                let mut tot = T::zero();
                for &j in &pos {
                    if j > i {
                        break;
                    }
                    tot += self[j].clone() * &f[i - j];
                }
                f[i] = -tot * &f[0];
            }
            return f;
        }
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
        if self.data.iter().filter(|x| !x.is_zero()).count()
            <= deg.next_power_of_two().trailing_zeros() as usize * 16
        {
            let diff = self.clone().diff();
            let pos: Vec<_> = diff
                .data
                .iter()
                .enumerate()
                .filter_map(|(i, x)| if x.is_zero() { None } else { Some(i) })
                .collect();
            let mf = T::memorized_factorial(deg);
            let mut f = Self::zeros(deg);
            f[0] = T::one();
            for i in 1..deg {
                let mut tot = T::zero();
                for &j in &pos {
                    if j > i - 1 {
                        break;
                    }
                    tot += f[i - 1 - j].clone() * &diff[j];
                }
                f[i] = tot * T::memorized_inv(&mf, i);
            }
            return f;
        }
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
            if k >= deg.div_ceil(rhs) {
                Self::zeros(deg)
            } else {
                let deg = deg - k * rhs;
                let x0 = self[k].clone();
                let mut f = (self >> k) / &x0;
                if f.data.iter().filter(|x| !x.is_zero()).count()
                    <= deg.next_power_of_two().trailing_zeros() as usize * 12
                {
                    f = f.pow_sparse1(T::from(rhs), deg);
                } else {
                    f = (f.log(deg) * &T::from(rhs)).exp(deg);
                }
                f *= x0.pow(rhs);
                f <<= k * rhs;
                f
            }
        } else {
            Self::zeros(deg)
        }
    }
    fn pow_sparse1(&self, rhs: T, deg: usize) -> Self {
        debug_assert!(!self[0].is_zero());
        let pos: Vec<_> = self
            .data
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(i, x)| if x.is_zero() { None } else { Some(i) })
            .collect();
        let mf = T::memorized_factorial(deg);
        let mut f = Self::zeros(deg);
        f[0] = T::one();
        for i in 1..deg {
            let mut tot = T::zero();
            for &j in &pos {
                if j > i {
                    break;
                }
                tot += (T::from(j) * &rhs - T::from(i - j)) * &self[j] * &f[i - j];
            }
            f[i] = tot * T::memorized_inv(&mf, i);
        }
        f
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
            let s = self[0].sqrt_coefficient()?;
            if self.data.iter().filter(|x| !x.is_zero()).count()
                <= deg.next_power_of_two().trailing_zeros() as usize * 4
            {
                let t = self[0].clone();
                let mut f = self / t;
                f = f.pow_sparse1(T::from(1) / T::from(2), deg);
                f *= s;
                return Some(f);
            }

            let mut f = Self::from(s);
            let inv2 = T::one() / (T::one() + T::one());
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
    /// [x^n] P(x) / Q(x)
    pub fn bostan_mori(mut self, mut rhs: Self, mut n: usize) -> T
    where
        C: NttReuse<T = Vec<T>>,
    {
        let mut res = T::zero();
        rhs.trim_tail_zeros();
        if self.length() >= rhs.length() {
            let r = &self / &rhs;
            if n < r.length() {
                res = r[n].clone();
            }
            self -= r * &rhs;
            self.trim_tail_zeros();
        }
        let k = rhs.length().next_power_of_two();
        let mut p = C::transform(self.data, k * 2);
        let mut q = C::transform(rhs.data, k * 2);
        while n > 0 {
            let t = C::even_mul_normal_neg(&q, &q);
            p = if n.is_multiple_of(2) {
                C::even_mul_normal_neg(&p, &q)
            } else {
                C::odd_mul_normal_neg(&p, &q)
            };
            q = t;
            n /= 2;
            if n != 0 {
                if C::MULTIPLE {
                    p = C::transform(C::inverse_transform(p, k), k * 2);
                    q = C::transform(C::inverse_transform(q, k), k * 2);
                } else {
                    p = C::ntt_doubling(p);
                    q = C::ntt_doubling(q);
                }
            }
        }
        let p = C::inverse_transform(p, k);
        let q = C::inverse_transform(q, k);
        res + p[0].clone() / q[0].clone()
    }
    /// return F(x) where [x^n] P(x) / Q(x) = [x^d-1] P(x) F(x)
    pub fn bostan_mori_msb(self, n: usize) -> Self {
        let d = self.length() - 1;
        if n == 0 {
            return (Self::one() << (d - 1)) / self[0].clone();
        }
        let q = self;
        let mq = q.clone().parity_inversion();
        let w = (q * &mq).even().bostan_mori_msb(n / 2);
        let mut s = Self::zeros(w.length() * 2 - (n % 2));
        for (i, x) in w.iter().enumerate() {
            s[i * 2 + (1 - n % 2)] = x.clone();
        }
        let len = 2 * d + 1;
        let ts = C::transform(s.prefix(len).data, len);
        mq.reversed().middle_product(&ts, len).prefix(d + 1)
    }
    /// x^n mod self
    pub fn pow_mod(self, n: usize) -> Self {
        let d = self.length() - 1;
        let q = self.reversed();
        let u = q.clone().bostan_mori_msb(n);
        let mut f = (u * q).prefix(d).reversed();
        f.trim_tail_zeros();
        f
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
    pub fn kth_term_of_linearly_recurrence(self, a: Vec<T>, k: usize) -> T
    where
        C: NttReuse<T = Vec<T>>,
    {
        if let Some(x) = a.get(k) {
            return x.clone();
        }
        let p = (Self::from_vec(a).prefix(self.length() - 1) * &self).prefix(self.length() - 1);
        p.bostan_mori(self, k)
    }
    pub fn kth_term(a: Vec<T>, k: usize) -> T
    where
        C: NttReuse<T = Vec<T>>,
    {
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
    /// sum_i (a_i x)^j
    pub fn sum_of_powers<I>(iter: I, deg: usize) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut n = T::zero();
        let prod = Self::product_all(
            iter.into_iter().map(|a| {
                n += T::one();
                Self::from_vec(vec![T::one(), -a])
            }),
            deg,
        );
        (-prod.log(deg).diff() << 1) + Self::from_vec(vec![n])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{num::mint_basic::Modulo1000000009, rand, tools::Xorshift};

    #[test]
    fn test_bostan_mori() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 0..200, m: 1..200, t: 0usize..=1, k: 0..[10, 1_000][t]);
            let f = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            let g = Fps998244353::from_vec(rng.random_iter(..).take(m).collect());
            let expected = f.clone().bostan_mori(g.clone(), k);
            let result = (f * g.inv(k + 1)).data.get(k).cloned().unwrap_or_default();
            assert_eq!(result, expected);

            let f = Fps::<Modulo1000000009>::from_vec(rng.random_iter(..).take(n).collect());
            let g = Fps::<Modulo1000000009>::from_vec(rng.random_iter(..).take(m).collect());
            let expected = f.clone().bostan_mori(g.clone(), k);
            let result = (f * g.inv(k + 1)).data.get(k).cloned().unwrap_or_default();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_bostan_mori_msb() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 2..20, t: 0usize..=1, k: 0..[10, 1_000_000_000][t]);
            let f = Fps998244353::from_vec(rng.random_iter(..).take(n - 1).collect());
            let g = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            let expected = f.clone().bostan_mori(g.clone(), k);
            let result = (f * g.bostan_mori_msb(k))[n - 2];
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_pow_mod() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 2..20, t: 0usize..=1, k: 0..[10, 1_000_000_000][t]);
            let f = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            let mut expected = Fps998244353::one();
            {
                let mut p = Fps998244353::one() << 1;
                let mut k = k;
                while k > 0 {
                    if k & 1 == 1 {
                        expected = (expected * &p) % &f;
                    }
                    p = (&p * &p) % &f;
                    k >>= 1;
                }
            }

            let result = f.pow_mod(k);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_sum_of_powers() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 0..100, m: 0..10);
            let a: Vec<_> = rng.random_iter(..).take(n).collect();
            let result = Fps998244353::sum_of_powers(a.iter().cloned(), m + 1);
            for k in 0..=m {
                let mut expected = MInt998244353::zero();
                for &x in &a {
                    expected += x.pow(k);
                }
                assert_eq!(result[k], expected);
            }
        }
    }
}
