use super::BarrettReduction;
use std::{mem::swap, num::Wrapping, ops::Range};

/// Sum of Floor of Linear mod 2^64
///
/// $$\sum_{i=0}^{n-1}\left\lfloor\frac{a\times i+b}{m}\right\rfloor$$
pub fn floor_sum(n: u64, a: u64, b: u64, m: u64) -> u64 {
    let mut ans = Wrapping(0u64);
    let (mut n, mut m, mut a, mut b) = (Wrapping(n), Wrapping(m), Wrapping(a), Wrapping(b));
    loop {
        let br = BarrettReduction::<u64>::new(m.0);
        if a >= m {
            let (q, r) = br.div_rem(a.0);
            let nc2 = if n.0 % 2 == 0 {
                n / Wrapping(2) * (n - Wrapping(1))
            } else {
                (n - Wrapping(1)) / Wrapping(2) * n
            };
            ans += nc2 * Wrapping(q);
            a = Wrapping(r);
        }
        if b >= m {
            let (q, r) = br.div_rem(b.0);
            ans += n * Wrapping(q);
            b = Wrapping(r);
        }
        let y_max = a * n + b;
        if y_max < m {
            break;
        }
        let (q, r) = br.div_rem(y_max.0);
        n = Wrapping(q);
        b = Wrapping(r);
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
        Wrapping(r - l),
        Wrapping(m as i64),
        Wrapping(a),
        Wrapping(b) + Wrapping(l) * Wrapping(a),
    );
    let a = (if a.0 < 0 {
        let r = a.0.rem_euclid(m.0);
        let nc2 = if n.0 % 2 == 0 {
            n / Wrapping(2) * (n - Wrapping(1))
        } else {
            (n - Wrapping(1)) / Wrapping(2) * n
        };
        ans -= nc2 * ((Wrapping(r) - a) / m);
        r
    } else {
        a.0
    }) as u64;
    let b = (if b.0 < 0 {
        let r = b.0.rem_euclid(m.0);
        ans -= n * ((Wrapping(r) - b) / m);
        r
    } else {
        b.0
    }) as u64;
    (ans + Wrapping(floor_sum(n.0 as u64, a, b, m.0 as u64) as i64)).0
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
    let mut rng = Xorshift::time();
    for _ in 0..Q {
        let (n, a, b, m) = rng.gen((..A, ..A, ..A, 1..A));
        let expected: u64 = (0..n).map(|i| (a * i + b) / m).sum();
        let result = floor_sum(n, a, b, m);
        assert_eq!(expected, result);

        let (mut l, mut r, a, b) = rng.gen((-B..B, -B..B, -B..B, -B..B));
        if l > r {
            swap(&mut l, &mut r);
        }
        let expected: i64 = (l..r).map(|i| (a * i + b).div_euclid(m as i64)).sum();
        let result = floor_sum_i64(l, r, a, b, m);
        assert_eq!(expected, result);

        let (mut lv, mut rv) = rng.gen((0..m as i64, 0..m as i64));
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
