use super::{FormalPowerSeries, FormalPowerSeriesCoefficient, NttReuse, One, Zero};
use std::mem::{replace, swap};

#[derive(Clone)]
struct FpsMatrix<T, C> {
    a00: FormalPowerSeries<T, C>,
    a01: FormalPowerSeries<T, C>,
    a10: FormalPowerSeries<T, C>,
    a11: FormalPowerSeries<T, C>,
}

struct FrequencyMatrix<C>
where
    C: NttReuse,
{
    a00: C::F,
    a01: C::F,
    a10: C::F,
    a11: C::F,
}

impl<C> Clone for FrequencyMatrix<C>
where
    C: NttReuse,
    C::F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            a00: self.a00.clone(),
            a01: self.a01.clone(),
            a10: self.a10.clone(),
            a11: self.a11.clone(),
        }
    }
}

impl<T, C> FpsMatrix<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: NttReuse<T = Vec<T>>,
    C::F: Clone,
{
    fn identity() -> Self {
        Self {
            a00: FormalPowerSeries::one(),
            a01: FormalPowerSeries::zero(),
            a10: FormalPowerSeries::zero(),
            a11: FormalPowerSeries::one(),
        }
    }

    fn multiply_vector(
        &self,
        p: &FormalPowerSeries<T, C>,
        q: &FormalPowerSeries<T, C>,
    ) -> (FormalPowerSeries<T, C>, FormalPowerSeries<T, C>) {
        (
            add(&self.a00 * p, &self.a01 * q),
            add(&self.a10 * p, &self.a11 * q),
        )
    }

    fn left_multiply_step(&mut self, quotient: &[T]) {
        swap(&mut self.a00, &mut self.a10);
        swap(&mut self.a01, &mut self.a11);
        let quotient: FormalPowerSeries<T, C> = FormalPowerSeries::from_vec(quotient.to_vec());
        self.a10 = add(
            replace(&mut self.a10, FormalPowerSeries::zero()),
            quotient.clone() * &self.a00,
        );
        self.a11 = add(
            replace(&mut self.a11, FormalPowerSeries::zero()),
            quotient * &self.a01,
        );
    }

    fn transform(&self, length: usize) -> FrequencyMatrix<C> {
        FrequencyMatrix {
            a00: reduced_transform(&self.a00, length),
            a01: reduced_transform(&self.a01, length),
            a10: reduced_transform(&self.a10, length),
            a11: reduced_transform(&self.a11, length),
        }
    }

    fn extend_transform(&self, frequency: FrequencyMatrix<C>, length: usize) -> FrequencyMatrix<C> {
        fn extend<T, C>(fps: &FormalPowerSeries<T, C>, frequency: C::F, length: usize) -> C::F
        where
            T: FormalPowerSeriesCoefficient,
            C: NttReuse<T = Vec<T>>,
            C::F: Clone,
        {
            if fps.length() <= length / 2 {
                C::ntt_doubling(frequency)
            } else {
                reduced_transform(fps, length)
            }
        }

        FrequencyMatrix {
            a00: extend(&self.a00, frequency.a00, length),
            a01: extend(&self.a01, frequency.a01, length),
            a10: extend(&self.a10, frequency.a10, length),
            a11: extend(&self.a11, frequency.a11, length),
        }
    }
}

impl<T, C> FrequencyMatrix<C>
where
    T: FormalPowerSeriesCoefficient,
    C: NttReuse<T = Vec<T>>,
    C::F: Clone,
{
    fn product_sum(left_a: &C::F, right_a: &C::F, left_b: &C::F, right_b: &C::F) -> C::F {
        let mut result = left_a.clone();
        C::multiply_prefix(&mut result, right_a);
        C::multiply_add(&mut result, left_b, right_b);
        result
    }

    fn multiply(&self, right: &Self) -> Self {
        Self {
            a00: Self::product_sum(&self.a00, &right.a00, &self.a01, &right.a10),
            a01: Self::product_sum(&self.a00, &right.a01, &self.a01, &right.a11),
            a10: Self::product_sum(&self.a10, &right.a00, &self.a11, &right.a10),
            a11: Self::product_sum(&self.a10, &right.a01, &self.a11, &right.a11),
        }
    }

    fn apply(&self, p: &C::F, q: &C::F, length: usize) -> (Vec<T>, Vec<T>) {
        (
            C::inverse_transform_ntt(Self::product_sum(p, &self.a00, q, &self.a01), length),
            C::inverse_transform_ntt(Self::product_sum(p, &self.a10, q, &self.a11), length),
        )
    }

    fn left_multiply_step(self, quotient: &FormalPowerSeries<T, C>, length: usize) -> Self {
        let negative_quotient = reduced_transform(&(-quotient), length);
        let mut a10 = self.a00;
        C::multiply_add(&mut a10, &negative_quotient, &self.a10);
        let mut a11 = self.a01;
        C::multiply_add(&mut a11, &negative_quotient, &self.a11);
        let result = Self {
            a00: self.a10,
            a01: self.a11,
            a10,
            a11,
        };
        if C::MULTIPLE {
            result.inverse_transform(length).transform(length)
        } else {
            result
        }
    }

    fn inverse_transform(self, length: usize) -> FpsMatrix<T, C> {
        FpsMatrix {
            a00: FormalPowerSeries::from_vec(C::inverse_transform_ntt(self.a00, length)),
            a01: FormalPowerSeries::from_vec(C::inverse_transform_ntt(self.a01, length)),
            a10: FormalPowerSeries::from_vec(C::inverse_transform_ntt(self.a10, length)),
            a11: FormalPowerSeries::from_vec(C::inverse_transform_ntt(self.a11, length)),
        }
    }
}

fn berlekamp_massey_naive<T>(a: &[T], max_work: usize) -> Option<Vec<T>>
where
    T: FormalPowerSeriesCoefficient,
{
    let n = a.len();
    let mut b = Vec::with_capacity(n + 1);
    let mut c = Vec::with_capacity(n + 1);
    let mut temporary = Vec::with_capacity(n + 1);
    b.push(T::one());
    c.push(T::one());
    let mut y = T::one();
    let mut work = 0usize;
    for k in 1..=n {
        let c_len = c.len();
        work = work.saturating_add(c_len);
        if work > max_work {
            return None;
        }
        let mut x = T::zero();
        for (c, a) in c.iter().zip(&a[k - c_len..]) {
            x += c.clone() * a.clone();
        }
        b.push(T::zero());
        let b_len = b.len();
        if x.is_zero() {
            continue;
        }
        let frequency = x.clone() / y.clone();
        if c_len < b_len {
            swap(&mut c, &mut temporary);
            c.clear();
            c.resize_with(b_len - c_len, T::zero);
            c.extend(temporary.iter().cloned());
            for (c, b) in c.iter_mut().rev().zip(b.iter().rev()) {
                *c -= frequency.clone() * b.clone();
            }
            swap(&mut b, &mut temporary);
            y = x;
        } else {
            for (c, b) in c.iter_mut().rev().zip(b.iter().rev()) {
                *c -= frequency.clone() * b.clone();
            }
        }
    }
    c.reverse();
    Some(c)
}

impl<T, C> FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: NttReuse<T = Vec<T>>,
    C::F: Clone,
{
    pub fn berlekamp_massey(input: &[T]) -> Self {
        if input.last().is_none_or(|value| value.is_zero())
            && input.iter().all(|value| value.is_zero())
        {
            return Self::one();
        }
        let max_work = if input.len() <= 1536 {
            usize::MAX
        } else {
            input.len().saturating_mul(2)
        };
        if let Some(recurrence) = berlekamp_massey_naive(input, max_work) {
            return Self::from_vec(recurrence);
        }
        let n = input.len();
        let leading_zeros = input.iter().take_while(|value| value.is_zero()).count();
        let sequence = Self::from_vec(input.to_vec()).trimed();
        let mut modulus = Self::zeros(n + 1);
        modulus[n] = T::one();
        let (matrix, _) = half_gcd(&modulus, &sequence, n / 2, n.max(1).next_power_of_two());
        let (x, y) = matrix.multiply_vector(&modulus, &sequence);
        let mut recurrence = if y.length() == 0 {
            matrix.a01.clone()
        } else {
            matrix.a11.clone()
        };
        let recurrence_leading_zeros = recurrence
            .iter()
            .take_while(|value| value.is_zero())
            .count();
        if recurrence_leading_zeros > 0 {
            let (division, _) = x.div_rem(y.clone());
            recurrence = add(recurrence * division, matrix.a01);
        }
        let inverse = T::one() / &recurrence[0];
        for value in recurrence.iter_mut() {
            *value *= &inverse;
        }
        let minimum_length = (leading_zeros + 2).max(y.length() + 1);
        if recurrence.length() < minimum_length {
            recurrence.resize(minimum_length);
        }
        recurrence
    }
}

fn degree<T, C>(fps: &FormalPowerSeries<T, C>) -> isize {
    fps.length() as isize - 1
}

fn add<T, C>(
    left: FormalPowerSeries<T, C>,
    right: FormalPowerSeries<T, C>,
) -> FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    (left + right).trimed()
}

fn tail<T, C>(fps: &FormalPowerSeries<T, C>, start: isize) -> FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    let start = start.max(0) as usize;
    if start >= fps.length() {
        FormalPowerSeries::zero()
    } else {
        FormalPowerSeries::from_vec(fps.data[start..].to_vec())
    }
}

fn coefficient<T, C>(fps: &FormalPowerSeries<T, C>, index: isize) -> T
where
    T: FormalPowerSeriesCoefficient,
{
    if index < 0 {
        T::zero()
    } else {
        fps.coeff(index as usize)
    }
}

fn brute_force<T, C>(
    mut p: FormalPowerSeries<T, C>,
    mut q: FormalPowerSeries<T, C>,
    k: usize,
) -> FpsMatrix<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: NttReuse<T = Vec<T>>,
    C::F: Clone,
{
    let threshold = degree(&p) - k as isize;
    let mut matrix = FpsMatrix::identity();
    while q.length() as isize > threshold {
        let q_degree = q.length() - 1;
        let mut negative_quotient = vec![T::zero(); p.length() - q.length() + 1];
        let inverse = -T::one() / &q[q_degree];
        for i in (0..negative_quotient.len()).rev() {
            negative_quotient[i] = p[i + q_degree].clone() * &inverse;
            p[i + q_degree] = T::zero();
            for j in 0..q_degree {
                let value = negative_quotient[i].clone() * &q[j];
                p[i + j] += &value;
            }
        }
        matrix.left_multiply_step(&negative_quotient);
        p.truncate(q_degree);
        p.trim_tail_zeros();
        swap(&mut p, &mut q);
    }
    matrix
}

fn reduced_transform<T, C>(fps: &FormalPowerSeries<T, C>, length: usize) -> C::F
where
    T: FormalPowerSeriesCoefficient,
    C: NttReuse<T = Vec<T>>,
{
    let mut coefficients = vec![T::zero(); length];
    for (i, value) in fps.iter().enumerate() {
        coefficients[i & (length - 1)] += value;
    }
    C::transform_ntt(coefficients, length)
}

fn transform_window<T, C>(fps: &FormalPowerSeries<T, C>, end: isize, length: usize) -> C::F
where
    T: FormalPowerSeriesCoefficient,
    C: NttReuse<T = Vec<T>>,
{
    let start = end - length as isize;
    let coefficients = (start..end).map(|index| coefficient(fps, index)).collect();
    C::transform_ntt(coefficients, length)
}

fn half_gcd<T, C>(
    p: &FormalPowerSeries<T, C>,
    q: &FormalPowerSeries<T, C>,
    k: usize,
    length: usize,
) -> (FpsMatrix<T, C>, FrequencyMatrix<C>)
where
    T: FormalPowerSeriesCoefficient,
    C: NttReuse<T = Vec<T>>,
    C::F: Clone,
{
    let d = degree(p);
    if degree(q) < d - k as isize {
        let matrix = FpsMatrix::identity();
        let frequency = matrix.transform(length);
        return (matrix, frequency);
    }
    if k == 1 {
        let matrix = FpsMatrix {
            a00: FormalPowerSeries::zero(),
            a01: FormalPowerSeries::one(),
            a10: FormalPowerSeries::one(),
            a11: -(tail(p, d - 2) / tail(q, d - 2)),
        };
        let frequency = matrix.transform(length);
        return (matrix, frequency);
    }
    if p.length().min(q.length()) <= 32 {
        let matrix = brute_force(p.clone(), q.clone(), k);
        let frequency = matrix.transform(length);
        return (matrix, frequency);
    }

    let half = length / 2;
    if k <= half {
        let (matrix, frequency) = half_gcd(p, q, k, half);
        let frequency = matrix.extend_transform(frequency, length);
        return (matrix, frequency);
    }

    let (matrix, mut matrix_frequency) = half_gcd(
        &tail(p, d - 2 * half as isize),
        &tail(q, d - 2 * half as isize),
        half,
        length,
    );
    let degeneracy = half as isize - degree(&matrix.a11);

    let (p0, q0) = matrix_frequency.apply(
        &transform_window(p, d - half as isize + degeneracy, length),
        &transform_window(q, d - half as isize + degeneracy, length),
        length,
    );
    let (p1, q1) = matrix_frequency.apply(
        &transform_window(p, d - 2 * half as isize, length),
        &transform_window(q, d - 2 * half as isize, length),
        length,
    );
    let part_length = (half as isize + degeneracy) as usize;
    let mut p_reduced = p1[length - part_length..].to_vec();
    p_reduced.extend_from_slice(&p0[length - part_length..]);
    let mut q_reduced = q1[length - part_length..].to_vec();
    q_reduced.extend_from_slice(&q0[length - part_length..]);
    let mut q_reduced: FormalPowerSeries<T, C> = FormalPowerSeries::from_vec(q_reduced).trimed();

    let position = d - half as isize + degeneracy;
    let mut leading = T::zero();
    for i in 0..=position {
        leading += coefficient(p, i) * coefficient(&matrix.a00, position - i)
            + coefficient(q, i) * coefficient(&matrix.a01, position - i);
    }
    p_reduced.push(leading);
    let mut p_reduced: FormalPowerSeries<T, C> = FormalPowerSeries::from_vec(p_reduced);
    if degree(&q_reduced) < 3 * half as isize + degeneracy - k as isize {
        return (matrix, matrix_frequency);
    }

    let mut remaining = k as isize - degree(&matrix.a11);
    let mut top_product = matrix.a11.data.last().unwrap().clone();
    let mut product_degree = degree(&matrix.a11);
    if degeneracy > 0 {
        let skip = (2 * half as isize + 2 * degeneracy - (d - half as isize + degeneracy)).max(0);
        let (division, remainder) = tail(&p_reduced, skip).div_rem(tail(&q_reduced, skip));
        remaining -= degree(&division);
        top_product *= -division.data.last().unwrap().clone();
        product_degree += degree(&division);
        matrix_frequency = matrix_frequency.left_multiply_step(&division, length);
        swap(&mut p_reduced, &mut q_reduced);
        q_reduced = FormalPowerSeries::zeros(skip as usize);
        q_reduced.data.extend(remainder.data);
    }

    let start = 3 * half as isize + degeneracy - k as isize - remaining;
    let (right_matrix, right_frequency) = half_gcd(
        &tail(&p_reduced, start),
        &tail(&q_reduced, start),
        remaining as usize,
        length,
    );
    let product_frequency = right_frequency.multiply(&matrix_frequency);
    let mut product = product_frequency.clone().inverse_transform(length);
    product.a00.truncate(k);
    product.a00.trim_tail_zeros();
    product.a01.truncate(k);
    product.a01.trim_tail_zeros();
    product.a10.truncate(k);
    product.a10.trim_tail_zeros();
    product_degree += degree(&right_matrix.a11);
    if product_degree == length as isize {
        product.a11.resize(k + 1);
        let highest = top_product * right_matrix.a11.data.last().unwrap();
        product.a11[k] = highest.clone();
        product.a11[0] -= highest;
    }
    product.a11.trim_tail_zeros();
    let product_frequency = if C::MULTIPLE {
        product.transform(length)
    } else {
        product_frequency
    };
    (product, product_frequency)
}

#[cfg(test)]
mod tests {
    use crate::math::formal_power_series::berlekamp_massey::berlekamp_massey_naive;
    use crate::{
        math::{FormalPowerSeries, FormalPowerSeriesCoefficient, Fps, Fps998244353, NttReuse},
        num::{MInt, One, Zero, mint_basic::Modulo1000000009, montgomery::MInt998244353},
        tools::Xorshift,
    };

    fn verify<T, C>(sequence: &[T], _: &FormalPowerSeries<T, C>)
    where
        T: Copy + FormalPowerSeriesCoefficient + std::fmt::Debug,
        C: NttReuse<T = Vec<T>>,
        C::F: Clone,
    {
        let expected = berlekamp_massey_naive(sequence, usize::MAX).unwrap();
        let actual: FormalPowerSeries<T, C> = FormalPowerSeries::berlekamp_massey(sequence);
        assert_eq!(actual.length(), expected.len());
        for i in actual.length() - 1..sequence.len() {
            let value = actual
                .iter()
                .enumerate()
                .fold(T::zero(), |sum, (j, &coefficient)| {
                    sum + coefficient * sequence[i - j]
                });
            assert!(value.is_zero());
        }
    }

    #[test]
    fn berlekamp_massey_random() {
        let mut rng = Xorshift::default();
        let direct_marker: Fps998244353 = FormalPowerSeries::zero();
        let arbitrary_marker: Fps<Modulo1000000009> = FormalPowerSeries::zero();
        for iteration in 0..20 {
            let n = if iteration < 5 {
                rng.random(0..=256)
            } else if iteration == 15 {
                1537
            } else {
                rng.random(257..=600)
            };
            let mut direct: Vec<MInt998244353> = rng.random_iter(..).take(n).collect();
            if iteration >= 5 && iteration % 5 == 0 {
                let degree = rng.random(n * 5 / 8..n * 7 / 8);
                let ratio: MInt998244353 = rng.random(2u32..998244351).into();
                let mut recurrence: Vec<MInt998244353> =
                    rng.random_iter(..).take(degree + 1).collect();
                if recurrence[degree].is_zero() {
                    recurrence[degree] = MInt998244353::one();
                }
                direct[0] = MInt998244353::one();
                for i in 1..degree {
                    direct[i] = direct[i - 1] * ratio;
                }
                for i in degree..n {
                    direct[i] = (1..=degree).fold(MInt998244353::zero(), |sum, j| {
                        sum + recurrence[j] * direct[i - j]
                    });
                }
            } else if iteration % 3 == 0 {
                direct
                    .iter_mut()
                    .skip(n * 3 / 4)
                    .for_each(|x| *x = Zero::zero());
            }
            verify(&direct, &direct_marker);

            let mut arbitrary: Vec<MInt<Modulo1000000009>> = rng.random_iter(..).take(n).collect();
            if iteration % 4 == 0 {
                arbitrary
                    .iter_mut()
                    .take(n / 4)
                    .for_each(|x| *x = Zero::zero());
            }
            verify(&arbitrary, &arbitrary_marker);
        }
    }
}
