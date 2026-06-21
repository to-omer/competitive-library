/// solve with Mo's algorithm
///
/// arg:
/// - lr: slice of pair of usize
/// - (l, r): ident of current pair
/// - incl: |i| expr: del i, l+=1 (post)
/// - decl: |i| expr: add i, l-=1 (pre)
/// - incr: |i| expr: add i, r+=1 (post)
/// - decr: |i| expr: del i, r-=1 (pre)
/// - answer: |i| expr: answer i-th pair of lr
///
/// incr and decr can be omitted, if simultaneous
///
/// ```
/// # use competitive::mo_algorithm;
/// let (a, lr) = ([1, 2, 3], [(0, 1), (0, 2), (1, 3)]);
/// let (mut ans, mut acc) = (0, 0);
/// mo_algorithm!(
///     &lr,
///     (l, r),
///     |i| acc -= a[i],
///     |i| acc += a[i],
///     |i| acc += a[i],
///     |i| acc -= a[i],
///     |i| ans += acc
/// );
/// assert_eq!(ans, 9);
/// ```
#[macro_export]
macro_rules! mo_algorithm {
    (
        $lr:expr,
        ($l:ident, $r:ident),
        |$i:tt| $inc:expr,
        |$d:tt| $dec:expr,
        |$a:tt| $answer:expr $(,)?
    ) => {{
        $crate::mo_algorithm!(
            $lr,
            ($l, $r),
            |$i| $inc,
            |$d| $dec,
            |$d| $dec,
            |$i| $inc,
            |$a| $answer
        );
    }};
    (
        $lr:expr,
        ($l:ident, $r:ident),
        |$il:tt| $incl:expr,
        |$dl:tt| $decl:expr,
        |$ir:tt| $incr:expr,
        |$dr:tt| $decr:expr,
        |$a:tt| $answer:expr $(,)?
    ) => {{
        fn mo_order<const SHIFTED: bool>(
            lr: &[(usize, usize)],
            maxv: usize,
            width: usize,
        ) -> (usize, Vec<usize>) {
            let shift = usize::from(SHIFTED) * (width / 2);
            let bucket = |x: usize| (x + shift) / width;
            let buckets = bucket(maxv) + 1;
            let mut pos = vec![0usize; buckets + 1];
            for &(l, _) in lr {
                pos[bucket(l) + 1] += 1;
            }
            for i in 1..=buckets {
                pos[i] += pos[i - 1];
            }
            let mut idx = vec![0usize; lr.len()];
            for (i, &(l, _)) in lr.iter().enumerate() {
                let p = &mut pos[bucket(l)];
                idx[*p] = i;
                *p += 1;
            }
            idx[..pos[0]].sort_unstable_by_key(|&i| lr[i].1);
            for b in (2..buckets).step_by(2) {
                idx[pos[b - 1]..pos[b]].sort_unstable_by_key(|&i| lr[i].1);
            }
            for b in (1..buckets).step_by(2) {
                idx[pos[b - 1]..pos[b]].sort_unstable_by_key(|&i| ::std::cmp::Reverse(lr[i].1));
            }
            let (mut l, mut r, mut len) = (0usize, 0usize, 0usize);
            for &i in &idx {
                let (nl, nr) = lr[i];
                len += l.abs_diff(nl) + r.abs_diff(nr);
                l = nl;
                r = nr;
            }
            (len, idx)
        }
        let lr: &[(usize, usize)] = $lr;
        let maxv = lr.iter().map(|&(l, r)| l.max(r)).max().unwrap_or_default();
        let width = ((maxv as f64) / (lr.len().max(1) as f64).sqrt())
            .round()
            .max(1.0) as usize;
        let (len0, idx0) = mo_order::<false>(lr, maxv, width);
        let (len1, idx1) = mo_order::<true>(lr, maxv, width);
        let idx = if len0 <= len1 { idx0 } else { idx1 };
        let (mut $l, mut $r) = (0usize, 0usize);
        for &$a in &idx {
            let (nl, nr) = lr[$a];
            while $l > nl {
                $l -= 1;
                let $dl: usize = $l;
                $decl;
            }
            while $r < nr {
                let $ir: usize = $r;
                $incr;
                $r += 1;
            }
            while $l < nl {
                let $il: usize = $l;
                $incl;
                $l += 1;
            }
            while $r > nr {
                $r -= 1;
                let $dr: usize = $r;
                $decr;
            }
            $answer;
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::{rand, tools::NotEmptySegment as Nes, tools::Xorshift};

    #[test]
    fn test_mo_algorithm() {
        let mut rng = Xorshift::default();
        for _ in 0..50 {
            rand!(rng, n: 1..50, q: 1..100, a: [1i64..1000; n], lr: [Nes(n); q]);
            let mut ans = 0;
            let mut acc = 0;
            mo_algorithm!(
                &lr,
                (l, r),
                |i| acc -= a[i],
                |i| acc += a[i],
                |i| acc += a[i],
                |i| acc -= a[i],
                |i| ans += acc
            );
            let mut exp = 0;
            for (l, r) in lr {
                exp += a[l..r].iter().sum::<i64>();
            }
            assert_eq!(ans, exp);
        }
    }
}
