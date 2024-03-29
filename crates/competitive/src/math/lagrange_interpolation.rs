use super::{MInt, MIntBase, MIntConvert, MemorizedFactorial, One, Zero};

pub fn lagrange_interpolation<M>(x: &[MInt<M>], y: &[MInt<M>], t: MInt<M>) -> MInt<M>
where
    M: MIntBase,
{
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

impl<M> MemorizedFactorial<M>
where
    M: MIntConvert<usize>,
{
    /// Lagrange interpolation with (i, f(i)) (0 <= i <= n)
    pub fn lagrange_interpolation<F>(&self, n: usize, f: F, t: MInt<M>) -> MInt<M>
    where
        F: Fn(MInt<M>) -> MInt<M>,
    {
        debug_assert!(0 < n && n < M::mod_into() + 1);
        if usize::from(t) <= n {
            return f(t);
        }
        let mut left = vec![MInt::one(); n + 1];
        for i in 0..n {
            left[i + 1] = left[i] * (t - MInt::from(i));
        }
        let (mut res, mut right) = (MInt::zero(), MInt::one());
        for i in (0..=n).rev() {
            res += f(MInt::from(i)) * left[i] * right * self.inv_fact[i] * self.inv_fact[n - i];
            right *= MInt::from(i) - t;
        }
        res
    }
}

pub fn lagrange_interpolation_polynomial<M>(x: &[MInt<M>], y: &[MInt<M>]) -> Vec<MInt<M>>
where
    M: MIntBase,
{
    let n = x.len() - 1;
    let mut dp = vec![MInt::zero(); n + 2];
    let mut ndp = vec![MInt::zero(); n + 2];
    dp[0] = -x[0];
    dp[1] = MInt::one();
    for x in x.iter().skip(1) {
        for j in 0..=n + 1 {
            ndp[j] = -dp[j] * x + if j >= 1 { dp[j - 1] } else { MInt::zero() };
        }
        std::mem::swap(&mut dp, &mut ndp);
    }
    let mut res = vec![MInt::zero(); n + 1];
    for i in 0..=n {
        let t = y[i]
            / (0..=n)
                .map(|j| if i != j { x[i] - x[j] } else { MInt::one() })
                .product::<MInt<M>>();
        if t.is_zero() {
            continue;
        } else if x[i].is_zero() {
            for j in 0..=n {
                res[j] += dp[j + 1] * t;
            }
        } else {
            let xinv = x[i].inv();
            let mut pre = MInt::zero();
            for j in 0..=n {
                let d = -(dp[j] - pre) * xinv;
                res[j] += d * t;
                pre = d;
            }
        }
    }
    res
}
