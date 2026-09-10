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
    pub fn trimed(mut self) -> Self {
        self.trim_tail_zeros();
        self
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
        for i in 1..self.length() {
            self.data[i - 1] = self.data[i].clone() * &c;
            c += T::one();
        }
        self.data.pop();
        self
    }
    pub fn integral(mut self) -> Self {
        let n = self.length();
        let mut fact = Vec::with_capacity(n + 1);
        let mut c = T::one();
        fact.push(c.clone());
        for _ in 1..n {
            fact.push(fact.last().cloned().unwrap() * c.clone());
            c += T::one();
        }
        let mut invf = T::one() / (fact.last().cloned().unwrap() * c.clone());
        self.data.push(T::zero());
        for i in (1..=n).rev() {
            self.data[i] = self.data[i - 1].clone() * (invf.clone() * fact.pop().unwrap());
            invf *= c.clone();
            c -= T::one();
        }
        self.data[0] = T::zero();
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
        self.iter()
            .rev()
            .fold(T::zero(), |sum, a| x.clone() * sum + a.clone())
    }
}

impl<T, C> FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    #[inline]
    fn is_sparse(&self, deg: usize, factor: usize) -> bool {
        self.data
            .iter()
            .take(deg)
            .filter(|x| !x.is_zero())
            .nth(deg.next_power_of_two().trailing_zeros() as usize * factor)
            .is_none()
    }
    pub fn inv(&self, deg: usize) -> Self {
        if deg == 0 {
            return Self::zero();
        }
        debug_assert!(!self[0].is_zero());
        if self.is_sparse(deg, 6) {
            let inv = T::one() / self[0].clone();
            let pos: Vec<_> = self
                .data
                .iter()
                .take(deg)
                .enumerate()
                .skip(1)
                .filter(|(_, x)| !x.is_zero())
                .map(|(i, x)| (i, -x.clone() * &inv))
                .collect();
            let mut f = Self::zeros(deg);
            f[0] = inv;
            for i in pos.first().map_or(deg, |x| x.0)..deg {
                let mut tot = T::zero();
                for (j, coefficient) in &pos {
                    if *j > i {
                        break;
                    }
                    tot += coefficient.clone() * &f[i - *j];
                }
                f[i] = tot;
            }
            return f;
        }
        let mut f = Self::from(T::one() / self[0].clone());
        f.data.reserve(deg.saturating_sub(1));
        let extend = |f: &mut Self, end| {
            for i in f.length()..end {
                let mut tot = T::zero();
                for j in 1..=i.min(self.length() - 1) {
                    tot += self[j].clone() * &f[i - j];
                }
                f.data.push(-tot * &f[0]);
            }
        };
        extend(&mut f, deg.min(32));
        let mut error = Vec::new();
        let mut i = f.length();
        while i < deg {
            if deg - i <= 4 {
                extend(&mut f, deg);
                break;
            }
            error.clear();
            error.extend(
                self.data[..(i * 2).min(deg).min(self.length())]
                    .iter()
                    .cloned(),
            );
            let factor = C::transform(f.data.clone(), 2 * i);
            let mut error_fft = C::transform(error, 2 * i);
            C::multiply(&mut error_fft, &factor);
            error = C::inverse_transform(error_fft, 2 * i);
            error.drain(..i);
            let mut error_fft = C::transform(error, 2 * i);
            C::multiply(&mut error_fft, &factor);
            error = C::inverse_transform(error_fft, 2 * i);
            error.truncate(i.min(deg - i));
            f.data.extend(error.drain(..).map(Neg::neg));
            i *= 2;
        }
        f
    }
    pub fn exp(&self, deg: usize) -> Self
    where
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
    {
        if deg == 0 {
            return Self::zero();
        }
        debug_assert!(self[0].is_zero());
        if self.is_sparse(deg, if deg <= 256 { 16 } else { 8 }) {
            let diff = self.prefix_ref(deg).diff();
            let pos: Vec<_> = diff
                .data
                .iter()
                .enumerate()
                .filter_map(|(i, x)| if x.is_zero() { None } else { Some(i) })
                .collect();
            let mut f = Self::zeros(deg);
            f[0] = T::one();
            if pos.is_empty() {
                return f;
            }
            let mf = T::memorized_factorial(deg);
            for i in pos.first().map_or(deg, |j| j + 1)..deg {
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
        self.exp_or_pow(None, deg)
    }

    fn sum_products(f: &[C::F], g: &[C::F], len: usize) -> Vec<T>
    where
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
    {
        let chunk = C::max_product_sum_count(&f[0]);
        f.rchunks(chunk)
            .zip(g.chunks(chunk))
            .map(|(f, g)| {
                let mut sum = f[f.len() - 1].clone();
                C::multiply_prefix(&mut sum, &g[0]);
                for (f, g) in f.iter().rev().skip(1).zip(&g[1..]) {
                    C::multiply_add(&mut sum, f, g);
                }
                C::inverse_transform_ntt(sum, len)
            })
            .reduce(|mut sum, part| {
                for (sum, value) in sum.iter_mut().zip(part) {
                    *sum += value;
                }
                sum
            })
            .unwrap()
    }

    fn exp_or_pow(&self, power: Option<T>, deg: usize) -> Self
    where
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
    {
        if deg == 1 {
            return Self::one();
        }
        let indices: Vec<_> = (0..=deg).map(T::from).collect();
        let modulus = <T::Base as MIntConvert<usize>>::mod_into();
        let mut inv = vec![T::zero(); deg + 1];
        inv[1] = T::one();
        for i in 2..=deg {
            inv[i] = -T::from(modulus / i) * &inv[modulus % i];
        }
        let block = deg.next_power_of_two() / 16;
        let logarithm = if let Some(rhs) = &power {
            self.prefix_ref(block).log(block) * rhs
        } else {
            self.prefix_ref(block)
        };
        let (kernel, mut kernel_inverse, previous_inverse_fft) =
            logarithm.exp_newton(block, &indices, &inv);
        if power.is_some() {
            kernel_inverse = (self.prefix_ref(block) * &kernel).inv(block);
        } else {
            let mut error_fft = C::transform_ntt(kernel.data.clone(), block);
            C::multiply_prefix(&mut error_fft, &previous_inverse_fft);
            let error = C::inverse_transform_ntt(error_fft, block);
            let mut error_fft =
                C::transform_ntt(error.into_iter().skip(block / 2).collect(), block);
            C::multiply_prefix(&mut error_fft, &previous_inverse_fft);
            let error = C::inverse_transform_ntt(error_fft, block / 2);
            kernel_inverse
                .data
                .extend(error.into_iter().take(block / 2).map(Neg::neg));
        }
        let kernel_data = kernel.data;
        let kernel_inverse_data = kernel_inverse.data;
        let kernel_inverse = C::transform(kernel_inverse_data, block * 2);
        let kernel = C::transform(kernel_data.clone(), block * 2);
        let blocks = deg.div_ceil(block);
        let mut derivative_ffts = Vec::with_capacity(blocks - 1);
        let mut polynomial_ffts = Vec::with_capacity(if power.is_some() { blocks - 1 } else { 0 });
        for q in 1..blocks {
            let mut values = Self::zeros(block * 2);
            for (i, values) in values.data.chunks_mut(block).enumerate() {
                let start = (q - i) * block;
                for (value, x) in values
                    .iter_mut()
                    .zip(self.iter().skip(start).take(deg - start))
                {
                    *value = x.clone();
                }
            }
            if power.is_some() {
                polynomial_ffts.push(C::transform_ntt(values.data.clone(), block * 2));
            }
            for (i, values) in values.data.chunks_mut(block).enumerate() {
                let start = (q - i) * block;
                for (value, index) in values.iter_mut().zip(&indices[start..]) {
                    *value *= index;
                }
            }
            derivative_ffts.push(C::transform_ntt(values.data, block * 2));
        }
        let mut result = kernel_data.clone();
        result.reserve(deg - block);
        let mut result_ffts = Vec::with_capacity(blocks - 1);
        for q in 1..blocks {
            result_ffts.push(C::transform_ntt(
                result[(q - 1) * block..q * block].to_vec(),
                block * 2,
            ));
            let mut values = Self::sum_products(&derivative_ffts[..q], &result_ffts, block);
            if let Some(rhs) = &power {
                let product = Self::sum_products(&polynomial_ffts[..q], &result_ffts, block);
                let factor = rhs.clone() + T::one();
                // The power satisfies f g' = rhs f' g.
                for (i, value) in values.iter_mut().take(deg - q * block).enumerate() {
                    *value = value.clone() * &factor - product[i].clone() * &indices[q * block + i];
                }
            }
            let mut values = C::transform(values, block * 2);
            C::multiply(&mut values, &kernel_inverse);
            let mut values = C::inverse_transform(values, block * 2);
            values.truncate(block);
            let len = block.min(deg - q * block);
            for (i, value) in values.iter_mut().take(len).enumerate() {
                *value *= &inv[q * block + i];
            }
            values[len..].fill(T::zero());
            let mut values = C::transform(values, block * 2);
            C::multiply(&mut values, &kernel);
            let mut values = C::inverse_transform(values, block * 2);
            values.truncate(len);
            result.extend(values);
        }
        Self::from_vec(result)
    }

    fn exp_newton(&self, deg: usize, indices: &[T], inv: &[T]) -> (Self, Self, C::F)
    where
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
    {
        if deg == 1 {
            let one = Self::one();
            return (one.clone(), one.clone(), C::transform_ntt(one.data, 1));
        }
        let mut f = Self::from_vec(vec![T::one(), self.coeff(1)]);
        let mut inverse = Self::one();
        let mut inverse_fft = C::transform_ntt(inverse.data.clone(), 2);
        let mut m = 2;
        while m < deg {
            let f_fft = C::transform_ntt(f.data.clone(), 2 * m);

            let previous_inverse_fft = inverse_fft;
            let mut error_fft = previous_inverse_fft.clone();
            C::multiply_prefix(&mut error_fft, &f_fft);
            let mut error = C::inverse_transform_ntt(error_fft, m);
            error[..m / 2].fill(T::zero());
            let mut error_fft = C::transform_ntt(error, m);
            C::multiply_prefix(&mut error_fft, &previous_inverse_fft);
            let error = C::inverse_transform_ntt(error_fft, m);
            inverse
                .data
                .extend(error.into_iter().skip(m / 2).map(Neg::neg));
            inverse_fft = C::transform_ntt(inverse.data.clone(), 2 * m);

            let mut delta = Self::from_vec(
                self.data
                    .iter()
                    .take(m)
                    .enumerate()
                    .skip(1)
                    .map(|(i, value)| value.clone() * &indices[i])
                    .collect(),
            );
            delta.resize(m);
            let mut delta_fft = C::transform_ntt(delta.data, m);
            C::multiply_prefix(&mut delta_fft, &f_fft);
            let mut delta = Self::from_vec(C::inverse_transform_ntt(delta_fft, m));
            for i in 1..f.length() {
                delta[i - 1] -= f[i].clone() * &indices[i];
            }
            delta.resize(2 * m);
            for i in (0..m - 1).rev() {
                delta.data[m + i] = delta.data[i].clone();
            }
            delta.data[..m - 1].fill(T::zero());
            let mut delta_fft = C::transform_ntt(delta.data, 2 * m);
            C::multiply_prefix(&mut delta_fft, &inverse_fft);
            let mut delta = C::inverse_transform_ntt(delta_fft, 2 * m);
            delta.pop();
            delta.push(T::zero());
            let target = (2 * m).min(deg);
            for i in (1..target).rev() {
                delta[i] = delta[i - 1].clone() * &inv[i];
            }
            delta[0] = T::zero();
            delta[target..].fill(T::zero());
            for i in m..(2 * m).min(self.length()) {
                delta[i] += self[i].clone();
            }
            delta[..m].fill(T::zero());
            let mut delta_fft = C::transform_ntt(delta, 2 * m);
            C::multiply_prefix(&mut delta_fft, &f_fft);
            let delta = C::inverse_transform_ntt(delta_fft, 2 * m);
            f.data
                .extend(delta.into_iter().skip(m).take((deg - m).min(m)));
            m *= 2;
        }
        (f, inverse, inverse_fft)
    }
    pub fn log(&self, deg: usize) -> Self {
        if deg == 0 {
            return Self::zero();
        }
        debug_assert!(!self[0].is_zero());
        if deg == 1 {
            return Self::zeros(1);
        }
        if self.is_sparse(deg, 2) {
            let pos: Vec<_> = self
                .iter()
                .take(deg)
                .enumerate()
                .skip(1)
                .filter_map(|(i, x)| (!x.is_zero()).then_some(i))
                .collect();
            let mut derivative = Self::zeros(deg);
            let inverse = T::one() / self[0].clone();
            for i in pos.first().copied().unwrap_or(deg)..deg {
                let mut value = self.coeff(i) * T::from(i);
                for &j in &pos {
                    if j >= i {
                        break;
                    }
                    value -= self[j].clone() * &derivative[i - j];
                }
                derivative[i] = value * &inverse;
            }
            if pos.is_empty() {
                return derivative;
            }
            derivative.data.remove(0);
            return derivative.integral();
        }
        let n = deg - 1;
        if n <= 64 {
            return (self.inv(deg) * self.prefix_ref(deg).diff())
                .prefix(n)
                .integral();
        }
        let half = n.next_power_of_two() / 2;
        let derivative = self.prefix_ref(deg).diff();
        let inverse = C::transform(self.inv(half).data, half * 2);
        let mut quotient = C::transform(derivative.prefix_ref(half).data, half * 2);
        C::multiply(&mut quotient, &inverse);
        let mut result = C::inverse_transform(quotient, half * 2);
        result.truncate(half);
        if n - half <= 4 {
            let inverse = T::one() / self[0].clone();
            for i in half..n {
                let mut value = derivative.coeff(i);
                for j in 1..=i.min(self.length() - 1) {
                    value -= self[j].clone() * &result[i - j];
                }
                result.push(value * &inverse);
            }
            return Self::from_vec(result).integral();
        }
        let quotient = C::transform(result.clone(), half * 2);
        let mut error = C::transform(self.prefix_ref(n).data, half * 2);
        C::multiply(&mut error, &quotient);
        let mut error = C::inverse_transform(error, half * 2);
        for i in 0..n - half {
            error[i] = derivative.coeff(half + i) - &error[half + i];
        }
        error.truncate(n - half);
        let mut error = C::transform(error, half * 2);
        C::multiply(&mut error, &inverse);
        let error = C::inverse_transform(error, half * 2);
        result.extend(error.into_iter().take(n - half));
        Self::from_vec(result).integral()
    }
    pub fn pow(&self, rhs: usize, deg: usize) -> Self
    where
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
    {
        if rhs == 0 {
            return Self::from_vec(
                once(T::one())
                    .chain(repeat_with(T::zero))
                    .take(deg)
                    .collect(),
            );
        }
        if rhs == 1 {
            return self.prefix_ref(deg).resized(deg);
        }
        if let Some(k) = self
            .iter()
            .take(deg.div_ceil(rhs))
            .position(|x| !x.is_zero())
        {
            let deg = deg - k * rhs;
            let x0 = self[k].clone();
            let mut f = (self.prefix_ref(k + deg) >> k) / &x0;
            if f.is_sparse(deg, 12) {
                f = f.pow_sparse1(T::from(rhs), deg);
            } else if rhs <= 4 {
                let squared = (&f * &f).prefix(deg);
                f = match rhs {
                    2 => squared,
                    3 => (squared * f).prefix(deg),
                    _ => (&squared * &squared).prefix(deg),
                }
                .resized(deg);
            } else {
                f = f.exp_or_pow(Some(T::from(rhs)), deg);
            }
            f *= x0.pow(rhs);
            f <<= k * rhs;
            f
        } else {
            Self::zeros(deg)
        }
    }
    fn pow_sparse1(&self, rhs: T, deg: usize) -> Self {
        debug_assert!(!self[0].is_zero());
        let mut pos: Vec<_> = self
            .data
            .iter()
            .take(deg)
            .enumerate()
            .skip(1)
            .filter(|(_, x)| !x.is_zero())
            .map(|(i, x)| (i, x.clone(), T::from(i) * &rhs * x))
            .collect();
        let mut f = Self::zeros(deg);
        f[0] = T::one();
        if pos.is_empty() {
            return f;
        }
        let mf = T::memorized_factorial(deg);
        for i in pos.first().map_or(deg, |x| x.0)..deg {
            let mut tot = T::zero();
            for (j, coefficient, weight) in &mut pos {
                if *j > i {
                    break;
                }
                tot += weight.clone() * &f[i - *j];
                *weight -= &*coefficient;
            }
            f[i] = tot * T::memorized_inv(&mf, i);
        }
        f
    }

    fn sparse_fold(&self, sparse: impl IntoIterator<Item = (usize, T)>, deg: usize) -> T {
        sparse
            .into_iter()
            .take_while(|&(i, _)| i <= deg)
            .fold(T::zero(), |sum, (i, x)| sum + x * self.coeff(deg - i))
    }

    /// solve: $X(QF)'=\alpha P'(QF)+\beta P(Q'F)$ in $O(deg * max(nz(P), nz(Q), nz(X)))$
    pub fn solve_sparse_differential2(
        p: &Self,
        q: &Self,
        x: &Self,
        alpha: T,
        beta: T,
        deg: usize,
    ) -> Self {
        if deg == 0 {
            return Self::zero();
        }
        let collect_sparse = |p: &Self| -> Vec<(usize, T)> {
            p.iter()
                .enumerate()
                .filter(|&(_, x)| !x.is_zero())
                .map(|(i, x)| (i, x.clone()))
                .collect()
        };
        assert!(q.coeff(0).is_one());
        assert!(x.coeff(0).is_one());
        let p = collect_sparse(p);
        let q = collect_sparse(q);
        let x = collect_sparse(x);
        let diff = |p: &[(usize, T)]| -> Vec<(usize, T)> {
            p.iter()
                .filter(|&&(i, _)| i > 0)
                .map(|&(i, ref x)| (i - 1, x.clone() * T::from(i)))
                .collect()
        };
        let dp = diff(&p);
        let dq = diff(&q);

        let mf = T::memorized_factorial(deg);
        let mut f = Self::zeros(deg);
        let mut qf = Self::zeros(deg);
        let mut dq_f = Self::zeros(deg);
        let mut d_qf = Self::zeros(deg);
        f[0] = T::one();
        for i in 0..deg - 1 {
            qf[i] = f.sparse_fold(q.iter().cloned(), i);
            dq_f[i] = f.sparse_fold(dq.iter().cloned(), i);
            let dp_qf_i = qf.sparse_fold(dp.iter().cloned(), i);
            let p_dq_f_i = dq_f.sparse_fold(p.iter().cloned(), i);
            let x_d_qf_i = d_qf.sparse_fold(
                x.iter()
                    .map(|&(i, ref x)| (i, x.clone() - T::from((i == 0) as usize))),
                i,
            );
            d_qf[i] = alpha.clone() * dp_qf_i + beta.clone() * p_dq_f_i - x_d_qf_i;

            let mut f_ip1 = d_qf[i].clone();
            for &(j, ref q) in q.iter().take_while(|&&(j, _)| j <= i) {
                if j > 0 {
                    f_ip1 -= q.clone() * &f[i - (j - 1)] * T::from(i - (j - 1));
                }
            }
            f[i + 1] = f_ip1 * T::memorized_inv(&mf, i + 1);
        }
        f
    }

    /// P^exp_p * Q^exp_q
    pub fn mul_of_pow_sparse(&self, q: &Self, exp_p: isize, exp_q: isize, deg: usize) -> Self {
        if deg == 0 {
            return Self::zero();
        }
        if exp_p == 0 && exp_q == 0 {
            return Self::from_vec(
                once(T::one())
                    .chain(repeat_with(T::zero))
                    .take(deg)
                    .collect(),
            );
        }
        if exp_p != 0 && self.iter().all(|x| x.is_zero()) {
            assert!(exp_p > 0);
            return Self::zeros(deg);
        }
        if exp_q != 0 && q.iter().all(|x| x.is_zero()) {
            assert!(exp_q > 0);
            return Self::zeros(deg);
        }

        let normalize = |f: &Self, exp: isize| {
            if exp == 0 {
                return (0usize, T::one(), Self::from_vec(vec![T::one()]));
            }
            let k = f.iter().position(|value| !value.is_zero()).unwrap();
            assert!(
                exp >= 0 || k == 0,
                "Negative exponent with zero constant term"
            );
            let c = f[k].clone();
            let f = (f.clone() >> k) / &c;
            (k, c, f)
        };
        let (sp, cp, mut p) = normalize(self, exp_p);
        let (sq, cq, mut q) = normalize(q, exp_q);

        let shift = exp_p
            .saturating_mul(sp as _)
            .saturating_add(exp_q.saturating_mul(sq as _)) as usize;
        if shift >= deg {
            return Self::zeros(deg);
        }
        p.truncate(deg - shift);
        q.truncate(deg - shift);

        let mut f = Self::solve_sparse_differential2(
            &p,
            &q,
            &p,
            T::from(exp_p),
            T::from(exp_q),
            deg - shift,
        );
        f *= cp.signed_pow(exp_p) * cq.signed_pow(exp_q);
        if shift > 0 {
            f <<= shift;
        }
        f.prefix(deg)
    }

    /// exp(P/Q)
    pub fn exp_of_div_sparse(&self, q: &Self, deg: usize) -> Self {
        if deg == 0 {
            return Self::zero();
        }
        let shift_q = q
            .iter()
            .position(|value| !value.is_zero())
            .expect("Zero denominator");
        let shift_p = self.iter().position(|value| !value.is_zero()).unwrap_or(!0);
        assert!(shift_p > shift_q);

        let mut p = self >> shift_q;
        let mut q = q >> shift_q;
        assert!(!q.coeff(0).is_zero());

        let c = q[0].clone();
        p /= c.clone();
        q /= c;

        Self::solve_sparse_differential2(&p, &q, &q, T::one(), -T::one(), deg)
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
            if deg <= 1 {
                return Some(Self::from(s).prefix(deg));
            }
            if self.is_sparse(deg, 4) {
                let t = self[0].clone();
                let mut f = self.prefix_ref(deg) / t;
                f = f.pow_sparse1(T::one() / T::from(2usize), deg);
                f *= s;
                return Some(f);
            }

            let mut f = Self::from(s);
            let inv2 = T::one() / (T::one() + T::one());
            let inv2s = inv2.clone() / &f[0];
            let extend = |f: &mut Self, end| {
                for i in f.length()..end {
                    let mut value = self.coeff(i);
                    for j in 1..i {
                        value -= f[j].clone() * &f[i - j];
                    }
                    f.data.push(value * &inv2s);
                }
            };
            extend(&mut f, deg.min(32));
            f.truncate(deg);
            if f.length() == deg {
                return Some(f);
            }
            let mut inverse = f.inv(f.length());
            let mut i = f.length();
            while i < deg {
                if deg - i <= 4 {
                    extend(&mut f, deg);
                    break;
                }
                let len = (i * 2).min(deg);
                let factor = C::transform(inverse.data.clone(), i * 2);
                let error = if !C::CYCLIC || i < 128 {
                    (self.prefix_ref(len) - &f * &f) >> i
                } else {
                    let square = C::square(f.data.clone(), i);
                    // The cyclic square folds its high half into the already known low half.
                    Self::from_vec(
                        square
                            .into_iter()
                            .take(len - i)
                            .enumerate()
                            .map(|(j, value)| self.coeff(i + j) + self.coeff(j) - value)
                            .collect(),
                    )
                };
                let mut error_fft = C::transform(error.data, i * 2);
                C::multiply(&mut error_fft, &factor);
                let delta = C::inverse_transform(error_fft, i * 2);
                f.data
                    .extend(delta.into_iter().take(len - i).map(|x| x * &inv2));
                if i * 2 + 4 < deg {
                    let mut error_fft = C::transform(f.data.clone(), i * 2);
                    C::multiply(&mut error_fft, &factor);
                    let error = C::inverse_transform(error_fft, i * 2);
                    let mut error_fft = C::transform(error.into_iter().skip(i).collect(), i * 2);
                    C::multiply(&mut error_fft, &factor);
                    let error = C::inverse_transform(error_fft, i * 2);
                    inverse.data.extend(error.into_iter().take(i).map(Neg::neg));
                }
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
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
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
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
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
        let mut k = rhs.length().next_power_of_two();
        let mut p = C::transform_ntt(self.data, k * 2);
        let mut q = C::transform_ntt(rhs.data, k * 2);
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
                if n < k / 2 {
                    p = C::transform_ntt(C::inverse_transform_ntt(p, k / 2), k);
                    q = C::transform_ntt(C::inverse_transform_ntt(q, k / 2), k);
                    k /= 2;
                } else if C::MULTIPLE {
                    p = C::transform_ntt(C::inverse_transform_ntt(p, k), k * 2);
                    q = C::transform_ntt(C::inverse_transform_ntt(q, k), k * 2);
                } else {
                    p = C::ntt_doubling(p, false);
                    q = C::ntt_doubling(q, false);
                }
            }
        }
        let p = C::inverse_transform_ntt(p, k);
        let q = C::inverse_transform_ntt(q, k);
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
    pub fn multipoint_evaluation(self, points: &[T]) -> Vec<T>
    where
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
    {
        let n = points.len();
        if n <= 32 || self.length() <= 32 {
            return points.iter().map(|p| self.eval(p.clone())).collect();
        }
        let size = n.next_power_of_two();
        let block = 16;
        let leaves = size / block;
        let mut subproduct_tree = Vec::with_capacity(leaves * 2);
        subproduct_tree.resize_with(leaves * 2, || None);
        let mut leaf_products = Vec::with_capacity(leaves);
        for i in 0..leaves {
            let mut product = vec![T::one()];
            for j in 0..block {
                let x = points.get(i * block + j).cloned().unwrap_or_else(T::zero);
                product.push(T::one());
                for k in (1..=j).rev() {
                    product[k] = product[k - 1].clone() - x.clone() * &product[k];
                }
                product[0] *= -x;
            }
            subproduct_tree[leaves + i] = Some(C::transform_ntt(product.clone(), block * 2));
            leaf_products.push(product);
        }
        for i in (1..leaves).rev() {
            let mut product = subproduct_tree[i * 2].as_ref().unwrap().clone();
            C::multiply_prefix(&mut product, subproduct_tree[i * 2 + 1].as_ref().unwrap());
            if i > 1 {
                product = C::ntt_doubling(product, true);
            }
            subproduct_tree[i] = Some(product);
        }
        let mut product = C::inverse_transform_ntt(subproduct_tree[1].take().unwrap(), size);
        product[0] -= T::one();
        product.push(T::one());
        let mut uptree_t = Vec::with_capacity(leaves * 2);
        uptree_t.resize_with(1, Zero::zero);
        let m = self.length();
        let v = Self::from_vec(product).reversed().resized(m);
        let s = C::transform(self.data, m * 2);
        uptree_t.push(v.inv(m).middle_product(&s, m * 2).resized(size));
        for i in 1..leaves {
            let degree = uptree_t[i].length();
            let spectrum = C::transform_ntt(std::mem::take(&mut uptree_t[i].data), degree);
            let left = subproduct_tree[i * 2].take().unwrap();
            let right = subproduct_tree[i * 2 + 1].take().unwrap();
            let mut child = spectrum.clone();
            C::multiply_prefix(&mut child, &right);
            let mut child = C::inverse_transform_ntt(child, degree);
            child.drain(..degree / 2);
            uptree_t.push(Self::from_vec(child));
            let mut child = spectrum;
            C::multiply_prefix(&mut child, &left);
            let mut child = C::inverse_transform_ntt(child, degree);
            child.drain(..degree / 2);
            uptree_t.push(Self::from_vec(child));
        }
        let mut result = Vec::with_capacity(n);
        for ((values, product), points) in uptree_t[leaves..]
            .iter()
            .zip(leaf_products)
            .zip(points.chunks(block))
        {
            let mut remainder = Self::zeros(block);
            for (j, value) in values.iter().enumerate() {
                for (r, p) in remainder.data[..=j].iter_mut().zip(&product[block - j..]) {
                    *r += value.clone() * p;
                }
            }
            result.extend(points.iter().map(|p| remainder.eval(p.clone())));
        }
        result
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
        C::F: Clone,
    {
        if let Some(x) = a.get(k) {
            return x.clone();
        }
        Self::berlekamp_massey(&a).kth_term_of_linearly_recurrence(a, k)
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

    pub fn power_projection(&self, w: &[T], m: usize) -> Self
    where
        C: NttReuse<T = Vec<T>>,
    {
        if w.is_empty() {
            return Self::zeros(m);
        }
        if m <= 1 {
            return Self::from_vec(vec![w[0].clone(); m]);
        }

        let n0 = w.len();
        let mut n = n0.next_power_of_two();
        let mut f = self.prefix_ref(n);
        f.resize(n);

        let base = n * 2;
        let mut p_flat = vec![T::zero(); base];
        for (i, wi) in w.iter().enumerate() {
            p_flat[n - 1 - i] = wi.clone();
        }
        let mut q_flat = vec![T::zero(); base * 2];
        q_flat[0] = T::one();
        let q_offset = base;
        for (i, fi) in f.iter().enumerate() {
            q_flat[q_offset + i] = -fi.clone();
        }
        let mut py = 1usize;
        let mut qy = 2usize;

        let y_limit = m;
        while n > 1 {
            let (mut p, mut q) = C::power_projection_step(p_flat, q_flat, n, py, qy);
            let new_py = (py + qy - 1).min(y_limit);
            let new_qy = (qy + qy - 1).min(y_limit);
            p.resize_with(n * new_py, T::zero);
            q.resize_with(n * new_qy, T::zero);

            let n2 = n / 2;
            for row in p.chunks_exact_mut(n) {
                row[n2..].fill_with(T::zero);
            }
            for row in q.chunks_exact_mut(n) {
                row[n2..].fill_with(T::zero);
            }
            p_flat = p;
            q_flat = q;
            py = new_py;
            qy = new_qy;
            n = n2;
        }

        let base = 2;
        let mut p_y = Vec::with_capacity(py);
        for y in 0..py {
            p_y.push(p_flat[base * y].clone());
        }
        let mut q_y = Vec::with_capacity(qy);
        for y in 0..qy {
            q_y.push(q_flat[base * y].clone());
        }
        (Self::from_vec(p_y) * Self::from_vec(q_y).inv(m)).prefix(m)
    }

    pub fn compositional_inverse(&self, deg: usize) -> Self
    where
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
    {
        if deg == 0 {
            return Self::zero();
        }
        if deg == 1 {
            return Self::from_vec(vec![T::zero()]);
        }
        debug_assert!(self[0].is_zero());
        debug_assert!(!self[1].is_zero());

        let mut f = self.prefix_ref(deg);
        f.resize(deg);
        let c = f[1].clone();
        f /= c.clone();

        let mut w = vec![T::zero(); deg];
        w[deg - 1] = T::one();
        let s = f.power_projection(&w, deg);

        let n = deg - 1;
        let n_t = T::from(n);
        let mut h = vec![T::zero(); n];
        for i in 1..=n {
            h[n - i] = s[i].clone() * &n_t / T::from(i);
        }

        let h_fps = Self::from_vec(h);
        let inv_n = T::one() / n_t;
        let mut t = h_fps.log(n);
        t *= -inv_n;
        let g_over_x = t.exp(n);
        let mut g = (g_over_x << 1).prefix(deg);

        let inv_c = T::one() / c;
        let mut pow = T::one();
        for coef in g.iter_mut() {
            *coef *= pow.clone();
            pow *= inv_c.clone();
        }
        g
    }
    /// f(x) <- f(x + a)
    pub fn taylor_shift(mut self, a: T) -> Self {
        let f = T::memorized_factorial(self.length());
        let n = self.length();
        for (i, coef) in self.data.iter_mut().enumerate() {
            *coef *= T::memorized_fact(&f)[i].clone();
        }
        self.data.reverse();
        let mut b = a.clone();
        let mut g = Self::from_vec(T::memorized_inv_fact(&f)[..n].to_vec());
        for i in 1..n {
            g[i] *= b.clone();
            b *= a.clone();
        }
        self *= g;
        self.truncate(n);
        self.data.reverse();
        for (i, coef) in self.data.iter_mut().enumerate() {
            *coef *= T::memorized_inv_fact(&f)[i].clone();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{num::mint_basic::Modulo1000000009, rand, tools::Xorshift};

    #[test]
    fn test_diff_integral() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.random(1..=300);
            let f = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            let expected_diff = Fps998244353::from_vec(
                (1..n)
                    .map(|i| f[i] * MInt998244353::from(i as u32))
                    .collect(),
            );
            let expected_integral = Fps998244353::from_vec(
                std::iter::once(MInt998244353::zero())
                    .chain(
                        f.iter()
                            .enumerate()
                            .map(|(i, &x)| x / MInt998244353::from(i as u32 + 1)),
                    )
                    .collect(),
            );
            assert_eq!(expected_diff, f.clone().diff());
            assert_eq!(expected_integral, f.integral());
        }
    }

    #[test]
    fn test_inv() {
        let mut rng = Xorshift::default();
        let degrees: Vec<_> = (0..=33)
            .chain((6..=9).flat_map(|k| (1 << k) - 1..=(1 << k) + 5))
            .chain((0..40).map(|_| rng.random(0usize..=600)))
            .collect();
        for deg in degrees {
            for n in [
                rng.random(1..=deg.max(1)),
                deg.max(1),
                rng.random(deg + 1..=deg + 100),
            ] {
                let mut f = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
                f[0] = MInt998244353::from(rng.random(1u32..998244353));
                let mut expected = Fps998244353::zeros(deg);
                if deg > 0 {
                    expected[0] = f[0].inv();
                }
                for i in 1..deg {
                    let mut sum = MInt998244353::zero();
                    for j in 1..=i.min(n - 1) {
                        sum += f[j] * expected[i - j];
                    }
                    expected[i] = -sum / f[0];
                }
                assert_eq!(f.inv(deg), expected);
            }
        }
    }

    #[test]
    fn test_sqrt() {
        let mut rng = Xorshift::default();
        let degrees: Vec<_> = (0..=33)
            .chain((6..=9).flat_map(|k| (1 << k) - 1..=(1 << k) + 5))
            .chain((0..40).map(|_| rng.random(0usize..=600)))
            .collect();
        for deg in degrees {
            for n in [
                rng.random(1..=deg.max(1)),
                deg.max(1),
                rng.random(deg + 1..=deg + 100),
            ] {
                let mut f = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
                for value in &mut f.data[1..] {
                    if rng.random(0..2) == 0 {
                        *value = MInt998244353::zero();
                    }
                }
                let root = MInt998244353::from(rng.random(1u32..998244353));
                f[0] = root * root;
                let mut expected = Fps998244353::zeros(deg);
                if deg > 0 {
                    expected[0] = f[0].sqrt().unwrap();
                }
                for i in 1..deg {
                    let mut sum = MInt998244353::zero();
                    for j in 1..i {
                        sum += expected[j] * expected[i - j];
                    }
                    expected[i] = (f.coeff(i) - sum) / (expected[0] * MInt998244353::from(2));
                }
                assert_eq!(f.sqrt(deg), Some(expected));
            }
        }
        for coefficient in (0..32u32).chain((0..100).map(|_| rng.random(0u32..998244353))) {
            for deg in [0, 1] {
                let coefficient = MInt998244353::from(coefficient);
                let f = Fps998244353::from_vec(vec![coefficient]);
                let expected = coefficient.sqrt().map(|root| vec![root; deg]);
                assert_eq!(f.sqrt(deg).map(|f| f.data), expected);
            }
        }

        use crate::{
            math::{Convolve, number_theoretic_transform::Montgomery32NttModulus},
            num::{
                mint_basic::{DynMIntU32, DynModuloU32},
                montgomery::MontgomeryReduction32,
            },
        };

        fn check<T, C>()
        where
            T: FormalPowerSeriesCoefficientSqrt + std::fmt::Debug,
            C: ConvolveSteps<T = Vec<T>>,
        {
            let mut rng = Xorshift::default();
            for _ in 0..16 {
                // More than four terms after 128 must exercise the transform-based update.
                let deg = rng.random(133..=256);
                let mut expected: Vec<_> = (0..deg)
                    .map(|_| T::from(rng.random(-2502isize..0)))
                    .collect();
                expected[0] = T::one();
                let mut f = FormalPowerSeries::<T, C>::zeros(deg);
                for i in 0..deg {
                    for j in 0..deg - i {
                        f[i + j] += expected[i].clone() * &expected[j];
                    }
                }
                assert_eq!(f.sqrt(deg).unwrap().data, expected);
            }
        }

        struct Mod<const P: u32>;
        impl<const P: u32> MontgomeryReduction32 for Mod<P> {
            const MOD: u32 = P;
        }
        impl<const P: u32> Montgomery32NttModulus for Mod<P> {}

        struct LinearTruncated;
        impl ConvolveSteps for LinearTruncated {
            type T = Vec<MInt998244353>;
            type F = Vec<MInt998244353>;
            fn length(t: &Self::T) -> usize {
                t.len()
            }
            fn transform(mut t: Self::T, len: usize) -> Self::F {
                t.resize(len, MInt998244353::zero());
                t
            }
            fn inverse_transform(mut f: Self::F, len: usize) -> Self::T {
                f.resize(len, MInt998244353::zero());
                f
            }
            fn multiply(f: &mut Self::F, g: &Self::F) {
                let mut result = vec![MInt998244353::zero(); f.len()];
                for (i, x) in f.iter().enumerate() {
                    for (j, y) in g.iter().enumerate().take(result.len() - i) {
                        result[i + j] += *x * *y;
                    }
                }
                *f = result;
            }
        }

        check::<MInt998244353, Convolve998244353>();
        DynMIntU32::set_mod(2503);
        check::<DynMIntU32, Convolve<(DynModuloU32, (Mod<257>, Mod<769>, Mod<3329>))>>();
        check::<MInt998244353, LinearTruncated>();
    }

    #[test]
    fn test_log() {
        let mut rng = Xorshift::default();
        let degrees: Vec<_> = (0..=33)
            .chain((6..=9).flat_map(|k| (1 << k) - 1..=(1 << k) + 5))
            .chain((0..40).map(|_| rng.random(0usize..=600)))
            .collect();
        for deg in degrees {
            for n in [
                rng.random(1..=deg.max(1)),
                deg.max(1),
                rng.random(deg + 1..=deg + 100),
            ] {
                for step in [1, rng.random(1..=n)] {
                    let mut f = Fps998244353::zeros(n);
                    for value in &mut f.data {
                        if rng.random(0..step) == 0 {
                            *value = rng.random(..);
                        }
                    }
                    f[0] = MInt998244353::from(rng.random(1u32..998244353));
                    let mut expected = Fps998244353::zeros(deg);
                    for i in 1..deg {
                        let mut value = f.coeff(i) * MInt998244353::from(i);
                        for j in 1..i.min(n) {
                            value -= f[j] * MInt998244353::from(i - j) * expected[i - j];
                        }
                        expected[i] = value / (f[0] * MInt998244353::from(i));
                    }
                    assert_eq!(f.log(deg), expected, "{n}/{deg}/{step}");
                }
            }
        }
    }

    #[test]
    fn test_exp() {
        let mut rng = Xorshift::default();
        for _ in 0..30 {
            let deg = rng.random(0usize..=600);
            let n = rng.random(deg.max(1)..=deg + 100);
            let mut f = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            assert_eq!(Fps998244353::zero(), f.inv(0));
            f[0] = MInt998244353::zero();
            assert_eq!(Fps998244353::zero(), f.exp(0));
            let mut expected = Fps998244353::zeros(deg);
            if deg > 0 {
                expected[0] = MInt998244353::one();
            }
            for i in 1..deg {
                let mut value = MInt998244353::zero();
                for j in 1..=i {
                    value += f[j] * MInt998244353::from(j as u32) * expected[i - j];
                }
                expected[i] = value / MInt998244353::from(i as u32);
            }
            assert_eq!(expected, f.exp(deg));
        }
    }

    #[test]
    fn test_pow() {
        let mut rng = Xorshift::default();
        let sizes: Vec<_> = (0..=16)
            .flat_map(|n| (0..=16).map(move |deg| (n, deg)))
            .chain((0..80).map(|_| (rng.random(17..=300), rng.random(17..=300))))
            .collect();
        for (n, deg) in sizes {
            for step in [1, rng.random(1..=n.max(1))] {
                for shift in [0, rng.random(0..=n)] {
                    let mut f = Fps998244353::zeros(n);
                    for value in &mut f.data[shift..] {
                        if rng.random(0..step) == 0 {
                            *value = rng.random(..);
                        }
                    }
                    let mut expected = vec![MInt998244353::zero(); deg];
                    if deg > 0 {
                        expected[0] = MInt998244353::one();
                    }
                    for rhs in 0..=7 {
                        assert_eq!(
                            f.pow(rhs, deg).data,
                            expected,
                            "{n}/{deg}/{step}/{shift}/{rhs}"
                        );
                        let mut next = vec![MInt998244353::zero(); deg];
                        for i in 0..deg {
                            for j in 0..n.min(deg - i) {
                                next[i + j] += expected[i] * f[j];
                            }
                        }
                        expected = next;
                    }
                }
            }
        }
        let powers: Vec<_> = [0, 1, 998244352, 998244353, 998244354, usize::MAX]
            .into_iter()
            .chain((0..30).map(|_| rng.random(..)))
            .collect();
        for rhs in powers {
            let deg = rng.random(33..=600);
            let c = MInt998244353::from(rng.random(1u32..998244353));
            for geometric in [false, true] {
                let mut f = vec![MInt998244353::one(); if geometric { deg } else { 2 }];
                for i in 1..f.len() {
                    f[i] = f[i - 1] * c;
                }
                let mut expected = vec![MInt998244353::one(); deg];
                // Coefficients of (1 + cx)^rhs and (1 - cx)^(-rhs).
                for i in 1..deg {
                    let factor = if geometric {
                        MInt998244353::from(rhs) + MInt998244353::from(i - 1)
                    } else {
                        MInt998244353::from(rhs) - MInt998244353::from(i - 1)
                    };
                    expected[i] = expected[i - 1] * c * factor / MInt998244353::from(i);
                }
                assert_eq!(Fps998244353::from_vec(f).pow(rhs, deg).data, expected);
            }
        }
    }

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
    fn test_multipoint_evaluation() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..100, m: 0..100);
            let f = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            let points: Vec<_> = rng.random_iter(..).take(m).collect();
            let expected = points.iter().map(|&x| f.eval(x)).collect::<Vec<_>>();
            assert_eq!(expected, f.multipoint_evaluation(&points));

            let f = Fps::<Modulo1000000009>::from_vec(rng.random_iter(..).take(n).collect());
            let points: Vec<_> = rng.random_iter(..).take(m).collect();
            let expected = points.iter().map(|&x| f.eval(x)).collect::<Vec<_>>();
            assert_eq!(expected, f.multipoint_evaluation(&points));
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

    #[test]
    fn test_power_projection() {
        macro_rules! check {
            ($mint:ty, $fps:ty, $rng:expr, $n:expr, $m:expr) => {{
                let f: Vec<$mint> = $rng.random_iter(..).take($n).collect();
                let w: Vec<$mint> = $rng.random_iter(..).take($n).collect();
                let mut power = vec![<$mint>::zero(); $n];
                power[0] = <$mint>::one();
                let mut expected = Vec::with_capacity($m);
                for _ in 0..$m {
                    expected.push(
                        w.iter()
                            .zip(&power)
                            .map(|(&w, &coefficient)| w * coefficient)
                            .sum(),
                    );
                    let mut next = vec![<$mint>::zero(); $n];
                    for (i, &left) in power.iter().enumerate() {
                        for (j, &right) in f[..$n - i].iter().enumerate() {
                            next[i + j] += left * right;
                        }
                    }
                    power = next;
                }
                assert_eq!(
                    <$fps>::from_vec(expected),
                    <$fps>::from_vec(f).power_projection(&w, $m)
                );
            }};
        }

        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.random(1usize..=40);
            let m = rng.random(0usize..=50);
            check!(MInt998244353, Fps998244353, rng, n, m);
            check!(MInt<Modulo1000000009>, Fps<Modulo1000000009>, rng, n, m);
        }
    }

    #[test]
    fn test_mul_of_pow_sparse() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(0usize..100);
            let prob = rng.random(0u32..100) as f64 / 100.0;
            let exp_p = rng.random(-5isize..5);
            let exp_q = rng.random(-5isize..5);
            let mut p = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            let mut q = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            for i in 0..n {
                if exp_p >= 0 && rng.gen_bool(prob) {
                    p[i] = MInt998244353::zero();
                }
                if exp_q >= 0 && rng.gen_bool(prob) {
                    q[i] = MInt998244353::zero();
                }
            }
            let mut expected = Fps998244353::one();
            if exp_p >= 0 {
                expected *= p.pow(exp_p as usize, n);
            } else {
                expected *= p.inv(n).pow((-exp_p) as usize, n);
            }
            if exp_q >= 0 {
                expected *= q.pow(exp_q as usize, n);
            } else {
                expected *= q.inv(n).pow((-exp_q) as usize, n);
            }
            expected.truncate(n);
            let result = p.mul_of_pow_sparse(&q, exp_p, exp_q, n);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_exp_of_div_sparse() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.random(1usize..100);
            let prob = rng.random(0u32..100) as f64 / 100.0;
            let mut p = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            let mut q = Fps998244353::from_vec(rng.random_iter(..).take(n).collect());
            for i in 0..n - 1 {
                if rng.gen_bool(prob) {
                    p[i] = MInt998244353::zero();
                    q[i] = MInt998244353::zero();
                }
                if rng.gen_bool(prob) {
                    p[i] = MInt998244353::zero();
                }
            }
            let k = q.iter().position(|x| !x.is_zero()).unwrap();
            p[k] = MInt998244353::zero();
            let expected = ((&p >> k) * (&q >> k).inv(n)).prefix(n).exp(n);
            let result = p.exp_of_div_sparse(&q, n);
            assert_eq!(result, expected);
        }
    }
}
