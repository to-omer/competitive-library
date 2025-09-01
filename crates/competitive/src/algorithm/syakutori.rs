/// arg:
/// - n: length of array
/// - (l, r): ident of left-bound and right-bound
/// - addable: |index| expr: return is addable element
/// - add: |index| expr: add element
/// - remove: |index| expr: remove element
/// - show: call n times for l in 0..n, with rightmost r
///
/// ```
/// # use competitive::syakutori;
/// let (a, w) = ([1, 2, 3, 4], 6);
/// let (mut ans, mut acc) = (0, 0);
/// syakutori!(
///     a.len(),
///     (l, r),
///     |i| acc + a[i] <= w,
///     |i| acc += a[i],
///     |i| acc -= a[i],
///     ans += r - l
/// );
/// assert_eq!(ans, 7);
/// ```
#[macro_export]
macro_rules! syakutori {
    (
        $n:expr,
        ($l:ident, $r:ident),
        |$i:ident| $addable:expr,
        |$j:ident| $add:expr,
        |$k:ident| $remove:expr,
        $show:expr $(,)?
    ) => {{
        let n: usize = $n;
        let mut $r: usize = 0;
        for $l in 0..n {
            while $r < n && {
                let $i: usize = $r;
                let cond: bool = $addable;
                cond
            } {
                let $j: usize = $r;
                $add;
                $r += 1;
            }
            $show;
            if $l == $r {
                $r += 1;
            } else {
                let $k: usize = $l;
                $remove;
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::{rand, tools::Xorshift};

    #[test]
    fn test_syakutori() {
        let mut rng = Xorshift::default();
        for _ in 0..50 {
            rand!(rng, n: 1..50, a: [1i64..1000; n], w: 1i64..10000);
            let mut ans = 0;
            let mut acc = 0;
            syakutori!(
                a.len(),
                (l, r),
                |i| acc + a[i] <= w,
                |i| acc += a[i],
                |i| acc -= a[i],
                ans += r - l
            );
            let mut exp = 0;
            for l in 0..n {
                for r in l..n {
                    if a[l..=r].iter().sum::<i64>() <= w {
                        exp += 1;
                    }
                }
            }
            assert_eq!(ans, exp);
        }
    }
}
