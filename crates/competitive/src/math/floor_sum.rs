use super::{BarrettReduction, Wrapping};
use std::{mem::swap, ops::Range};

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

#[test]
fn test_floor_sum() {
    use crate::tools::Xorshift;
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
