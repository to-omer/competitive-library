use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeMinRangeUpdate, data_structure::LazySegmentTree};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_F")]
pub fn dsl_2_f(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::<RangeMinRangeUpdate<_>>::new(n);
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, s, t, x: i32);
            seg.update(s, t + 1, Some(x));
        } else {
            scan!(scanner, s, t);
            writeln!(writer, "{}", seg.fold(s, t + 1)).ok();
        }
    }
}
