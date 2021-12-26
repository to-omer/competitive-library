use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeMinRangeAdd, data_structure::LazySegmentTree};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_H")]
pub fn dsl_2_h(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::<RangeMinRangeAdd<_>>::from_vec(vec![0; n]);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, s, t, x: i64);
                seg.update(s, t + 1, x);
            }
            1 => {
                scan!(scanner, s, t);
                writeln!(writer, "{}", seg.fold(s, t + 1)).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
