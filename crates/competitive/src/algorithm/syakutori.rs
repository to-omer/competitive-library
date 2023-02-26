/// arg:
/// - n: length of array
/// - (l, r): ident of left-bound and right-bound
/// - addable: |index| expr: return is addable element
/// - add: |index| expr: add element
/// - remove: |index| expr: remove element
/// - show: call n times for l in 0..n, with rightmost r
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
