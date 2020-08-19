use crate::math::factorial::MemorizedFactorial;
use crate::num::{MInt, Modulus, One, Zero};

pub fn lagrange_interpolation<M: Modulus>(x: &[MInt<M>], y: &[MInt<M>], t: MInt<M>) -> MInt<M> {
    let n = x.len();
    debug_assert!(n == y.len());
    x.iter().position(|&x| x == t).map_or_else(
        || {
            (0..n)
                .map(|i| {
                    y[i] * (0..n)
                        .filter(|&j| j != i)
                        .map(|j| (t - x[j]) / (x[i] - x[j]))
                        .product::<MInt<M>>()
                })
                .sum()
        },
        |i| y[i],
    )
}

#[cargo_snippet::snippet("lagrange_interpolation")]
#[cargo_snippet::snippet(include = "factorial")]
impl<M: Modulus> MemorizedFactorial<M> {
    /// Lagrange interpolation with (i, f(i)) (0 <= i <= n)
    pub fn lagrange_interpolation<F>(&self, n: usize, f: F, t: MInt<M>) -> MInt<M>
    where
        F: Fn(MInt<M>) -> MInt<M>,
    {
        debug_assert!(0 < n && n < M::get_modulus() as usize + 1);
        if t.inner() <= n as u32 {
            return f(t);
        }
        let mut left = vec![MInt::one(); n + 1];
        for i in 0..n {
            left[i + 1] = left[i] * (t - MInt::new_unchecked(i as u32));
        }
        let (mut res, mut right) = (MInt::zero(), MInt::one());
        for i in (0..=n).rev() {
            res += f(MInt::new_unchecked(i as u32))
                * left[i]
                * right
                * self.inv_fact[i]
                * self.inv_fact[n - i];
            right *= MInt::new_unchecked(i as u32) - t;
        }
        res
    }
}
