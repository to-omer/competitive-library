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
        fn hilbert_curve_order(mut x: usize, mut y: usize, m: usize) -> usize {
            let n = 1usize << m;
            let mut ord = 0usize;
            for k in (0..m).rev() {
                let rx = x >> k & 1;
                let ry = y >> k & 1;
                ord += (1 << k * 2) * (3 * rx ^ ry);
                if ry == 0 {
                    if rx == 1 {
                        x = n - x - 1;
                        y = n - y - 1;
                    }
                    ::std::mem::swap(&mut x, &mut y);
                }
            }
            ord
        }
        let lr: &[(usize, usize)] = $lr;
        let q = lr.len();
        let maxv = lr.iter().map(|&(l, r)| l.max(r)).max().unwrap_or_default();
        let mut m = 0usize;
        while maxv >= 1 << m {
            m += 1;
        }
        let mut idx: Vec<usize> = (0..q).collect();
        let ord: Vec<_> = lr
            .iter()
            .map(|&(l, r)| hilbert_curve_order(l, r, m))
            .collect();
        idx.sort_unstable_by_key(|&i| ord[i]);
        let (mut $l, mut $r) = (0usize, 0usize);
        for &$a in idx.iter() {
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
