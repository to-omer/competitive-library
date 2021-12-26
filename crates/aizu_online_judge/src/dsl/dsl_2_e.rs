use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeSumRangeAdd, data_structure::LazySegmentTree};

#[cfg_attr(
    nightly,
    verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_E")
)]
pub fn dsl_2_e(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::<RangeSumRangeAdd<_>>::from_vec(vec![(0, 1); n]);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, s, t, x: u64);
                seg.update(s - 1, t, x);
            }
            1 => {
                scan!(scanner, i);
                writeln!(writer, "{}", seg.fold(i - 1, i).0).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
