use super::{
    AddMulOperation, Associative, BarrettReduction, Group, Invertible, Magma, Monoid, One, Ring,
    SemiRing, Unital, Wrapping, Zero, array,
};
use std::{
    marker::PhantomData,
    mem::swap,
    ops::{Add, Mul, Range},
};

fn choose2(n: Wrapping<u64>) -> Wrapping<u64> {
    if n.0 % 2 == 0 {
        n / 2 * (n - 1)
    } else {
        (n - 1) / 2 * n
    }
}

/// Sum of Floor of Linear mod 2^64
///
/// $$\sum_{i=0}^{n-1}\left\lfloor\frac{a\times i+b}{m}\right\rfloor$$
pub fn floor_sum(n: u64, a: u64, b: u64, m: u64) -> u64 {
    let mut ans = Wrapping(0u64);
    let (mut n, mut m, mut a, mut b) = (Wrapping(n), m, a, b);
    loop {
        let br = BarrettReduction::<u64>::new(m);
        if a >= m {
            let (q, r) = br.div_rem(a);
            ans += choose2(n) * q;
            a = r;
        }
        if b >= m {
            let (q, r) = br.div_rem(b);
            ans += n * q;
            b = r;
        }
        let y_max = (n * a + b).0;
        if y_max < m {
            break;
        }
        let (q, r) = br.div_rem(y_max);
        n = Wrapping(q);
        b = r;
        swap(&mut m, &mut a);
    }
    ans.0
}

/// Sum of Floor of Linear mod 2^64
///
/// $$\sum_{i=l}^{r-1}\left\lfloor\frac{a\times i+b}{m}\right\rfloor$$
pub fn floor_sum_i64(l: i64, r: i64, a: i64, b: i64, m: u64) -> i64 {
    let mut ans = Wrapping(0i64);
    let (n, m, a, b) = (
        Wrapping((r - l) as u64),
        m as i64,
        a,
        (Wrapping(l) * a + b).0,
    );
    let a = if a < 0 {
        let r = a.rem_euclid(m);
        let nc2 = choose2(n);
        ans -= Wrapping(nc2.0 as _) * ((Wrapping(r) - a) / m);
        r
    } else {
        a
    };
    let b = if b < 0 {
        let r = b.rem_euclid(m);
        ans -= Wrapping(n.0 as _) * ((Wrapping(r) - b) / m);
        r
    } else {
        b
    };
    (ans + floor_sum(n.0, a as u64, b as u64, m as u64) as i64).0
}

pub fn floor_sum_range_freq(l: i64, r: i64, a: i64, b: i64, m: u64, range: Range<i64>) -> i64 {
    if range.start >= range.end {
        return 0;
    }
    assert!(0 <= range.start && range.end <= m as i64);
    let x1 = floor_sum_i64(l, r, a, b - range.start, m);
    let x2 = floor_sum_i64(l, r, a, b - range.end, m);
    x1 - x2
}

struct FloorSum<R, const X: usize, const Y: usize>
where
    R: SemiRing,
{
    _marker: PhantomData<fn() -> R>,
}

#[derive(Debug)]
struct FloorSumData<R, const X: usize, const Y: usize>
where
    R: SemiRing,
{
    dp: [[R::T; Y]; X],
    dx: R::T,
    dy: R::T,
    _marker: PhantomData<fn() -> R>,
}

impl<R, const X: usize, const Y: usize> Clone for FloorSumData<R, X, Y>
where
    R: SemiRing,
{
    fn clone(&self) -> Self {
        Self {
            dp: self.dp.clone(),
            dx: self.dx.clone(),
            dy: self.dy.clone(),
            _marker: self._marker,
        }
    }
}

impl<R, const X: usize, const Y: usize> FloorSum<R, X, Y>
where
    R: SemiRing,
{
    fn to_x() -> FloorSumData<R, X, Y> {
        let mut dp = array![array![R::zero(); Y]; X];
        dp[0][0] = R::one();
        FloorSumData {
            dp,
            dx: R::one(),
            dy: R::zero(),
            _marker: PhantomData,
        }
    }
    fn to_y() -> FloorSumData<R, X, Y> {
        FloorSumData {
            dp: array![array![R::zero(); Y]; X],
            dx: R::zero(),
            dy: R::one(),
            _marker: PhantomData,
        }
    }
}

impl<R, const X: usize, const Y: usize> FloorSum<R, X, Y>
where
    R: Ring,
    R::Additive: Invertible,
{
    fn offset(x: i64, y: i64) -> FloorSumData<R, X, Y> {
        FloorSumData {
            dp: array![array![R::zero(); Y]; X],
            dx: R::Additive::signed_pow(R::one(), x),
            dy: R::Additive::signed_pow(R::one(), y),
            _marker: PhantomData,
        }
    }
}

impl<R, const X: usize, const Y: usize> Magma for FloorSum<R, X, Y>
where
    R: SemiRing,
{
    type T = FloorSumData<R, X, Y>;

    fn operate(a: &Self::T, b: &Self::T) -> Self::T {
        let mut a = a.clone();
        let mut b = b.clone();
        let mut pow_x = array![R::zero(); X];
        let mut pow_y = array![R::zero(); Y];
        pow_x[0] = R::one();
        pow_y[0] = R::one();
        for i in 1..X {
            pow_x[i] = R::mul(&pow_x[i - 1], &a.dx);
        }
        for j in 1..Y {
            pow_y[j] = R::mul(&pow_y[j - 1], &a.dy);
        }
        macro_rules! go {
            ($N:ident) => {
                let mut comb = array![array![R::zero(); $N]; $N];
                comb[0][0] = R::one();
                let mut i = 0;
                while i + 1 < $N {
                    let mut j = 0;
                    while j <= i {
                        let x = comb[i][j].clone();
                        R::add_assign(&mut comb[i + 1][j], &x);
                        R::add_assign(&mut comb[i + 1][j + 1], &x);
                        j += 1;
                    }
                    i += 1;
                }
                for i in 0..X {
                    for j in (0..Y).rev() {
                        for k in j + 1..Y {
                            let mut x = b.dp[i][j].clone();
                            R::mul_assign(&mut x, &comb[k][j]);
                            R::mul_assign(&mut x, &pow_y[k - j]);
                            R::add_assign(&mut b.dp[i][k], &x);
                        }
                    }
                }
                for j in 0..Y {
                    for i in (0..X).rev() {
                        for k in i..X {
                            let mut x = b.dp[i][j].clone();
                            R::mul_assign(&mut x, &comb[k][i]);
                            R::mul_assign(&mut x, &pow_x[k - i]);
                            R::add_assign(&mut a.dp[k][j], &x);
                        }
                    }
                }
            };
        }
        if X <= Y {
            go!(Y);
        } else {
            go!(X);
        }
        R::add_assign(&mut a.dx, &b.dx);
        R::add_assign(&mut a.dy, &b.dy);
        a
    }
}

impl<R, const X: usize, const Y: usize> Unital for FloorSum<R, X, Y>
where
    R: SemiRing,
{
    fn unit() -> Self::T {
        FloorSumData {
            dp: array![array![R::zero(); Y]; X],
            dx: R::zero(),
            dy: R::zero(),
            _marker: PhantomData,
        }
    }
}

impl<R, const X: usize, const Y: usize> Associative for FloorSum<R, X, Y> where R: SemiRing {}

fn floor_monoid_product<M>(
    mut x: M::T,
    mut y: M::T,
    mut n: u64,
    mut a: u64,
    mut b: u64,
    mut m: u64,
) -> M::T
where
    M: Monoid,
{
    let mut c = (a * n + b) / m;
    let mut pre = M::unit();
    let mut suf = M::unit();
    loop {
        let (p, q) = (a / m, b / m);
        a %= m;
        b %= m;
        x = M::operate(&x, &M::pow(y.clone(), p));
        pre = M::operate(&pre, &M::pow(y.clone(), q));
        c -= p * n + q;
        if c == 0 {
            break;
        }
        let d = (m * c - b - 1) / a + 1;
        suf = M::operate(&y, &M::operate(&M::pow(x.clone(), n - d), &suf));
        b = m - b - 1 + a;
        n = c - 1;
        c = d;
        swap(&mut m, &mut a);
        swap(&mut x, &mut y);
    }
    x = M::pow(x.clone(), n);
    M::operate(&M::operate(&pre, &x), &suf)
}

/// $$\sum_{i=0}^{n-1}i^X\left\lfloor\frac{a\times i+b}{m}\right\rfloor^Y$$
pub fn floor_sum_polynomial<T, const X: usize, const Y: usize>(
    n: u64,
    a: u64,
    b: u64,
    m: u64,
) -> [[T; Y]; X]
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>,
{
    debug_assert!(a == 0 || n < (u64::MAX - b) / a);
    floor_monoid_product::<FloorSum<AddMulOperation<T>, X, Y>>(
        FloorSum::<AddMulOperation<T>, X, Y>::to_x(),
        FloorSum::<AddMulOperation<T>, X, Y>::to_y(),
        n,
        a,
        b,
        m,
    )
    .dp
}

/// $$\sum_{i=l}^{r-1}i^X\left\lfloor\frac{a\times i+b}{m}\right\rfloor^Y$$
pub fn floor_sum_polynomial_i64<T, const X: usize, const Y: usize>(
    l: i64,
    r: i64,
    a: i64,
    b: i64,
    m: u64,
) -> [[T; Y]; X]
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>,
    <AddMulOperation<T> as SemiRing>::Additive: Invertible,
{
    assert!(l <= r);
    assert!(m > 0);

    if a < 0 {
        let mut ans = floor_sum_polynomial_i64::<T, X, Y>(-r + 1, -l + 1, -a, b, m);
        for i in (1..X).step_by(2) {
            for j in 0..Y {
                ans[i][j] = AddMulOperation::<T>::neg(&ans[i][j]);
            }
        }
        return ans;
    }

    let add_x = l;
    let n = (r - l) as u64;
    let b = a * add_x + b;

    let add_y = b.div_euclid(m as i64);
    let b = b.rem_euclid(m as i64);
    assert!(a >= 0);
    assert!(b >= 0);
    let data = floor_monoid_product::<FloorSum<AddMulOperation<T>, X, Y>>(
        FloorSum::<AddMulOperation<T>, X, Y>::to_x(),
        FloorSum::<AddMulOperation<T>, X, Y>::to_y(),
        n,
        a as u64,
        b as u64,
        m,
    );

    let offset = FloorSum::<AddMulOperation<T>, X, Y>::offset(add_x, add_y);
    FloorSum::<AddMulOperation<T>, X, Y>::operate(&offset, &data).dp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_floor_sum() {
        const A: u64 = 1_000;
        const B: i64 = 1_000;
        const Q: usize = 1_000;
        let mut rng = Xorshift::new();
        for _ in 0..Q {
            let (n, a, b, m) = rng.random((..A, ..A, ..A, 1..A));
            let expected: u64 = (0..n).map(|i| (a * i + b) / m).sum();
            let result = floor_sum(n, a, b, m);
            assert_eq!(expected, result);

            let (mut l, mut r, a, b) = rng.random((-B..B, -B..B, -B..B, -B..B));
            if l > r {
                swap(&mut l, &mut r);
            }
            let expected: i64 = (l..r).map(|i| (a * i + b).div_euclid(m as i64)).sum();
            let result = floor_sum_i64(l, r, a, b, m);
            assert_eq!(expected, result);

            let (mut lv, mut rv) = rng.random((0..m as i64, 0..m as i64));
            if lv > rv {
                swap(&mut lv, &mut rv);
            }
            let range = lv..rv + 1;
            let expected = (l..r)
                .filter(|&i| range.contains(&(a * i + b).rem_euclid(m as i64)))
                .count() as i64;
            let result = floor_sum_range_freq(l, r, a, b, m, range);
            assert_eq!(expected, result);
        }
    }

    #[test]
    fn test_floor_sum_polynomial() {
        const P: usize = 3;
        const A: u64 = 100;
        const B: i64 = 100;
        const Q: usize = 1_000;
        let mut rng = Xorshift::new();
        for _ in 0..Q {
            let (n, a, b, m) = rng.random((..A, ..A, ..A, 1..A));
            let mut expected: [[u64; P]; P] = [[0; P]; P];
            for (x, expected) in expected.iter_mut().enumerate() {
                for (y, expected) in expected.iter_mut().enumerate() {
                    *expected = (0..n)
                        .map(|i| i.pow(x as u32) * ((a * i + b) / m).pow(y as u32))
                        .sum();
                }
            }
            let result = floor_sum_polynomial::<u64, P, P>(n, a, b, m);
            assert_eq!(expected, result);

            let (mut l, mut r, a, b) = rng.random((-B..B, -B..B, -B..B, -B..B));
            if l > r {
                swap(&mut l, &mut r);
            }
            let mut expected: [[i64; P]; P] = [[0; P]; P];
            for (x, expected) in expected.iter_mut().enumerate() {
                for (y, expected) in expected.iter_mut().enumerate() {
                    *expected = (l..r)
                        .map(|i| i.pow(x as u32) * (a * i + b).div_euclid(m as i64).pow(y as u32))
                        .sum();
                }
            }
            let result = floor_sum_polynomial_i64::<i64, P, P>(l, r, a, b, m);
            assert_eq!(expected, result);
        }
    }
}
