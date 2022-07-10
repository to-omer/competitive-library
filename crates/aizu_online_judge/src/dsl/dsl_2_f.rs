use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeMinRangeUpdate, data_structure::LazySegmentTree};

#[verify::aizu_online_judge("DSL_2_F")]
pub fn dsl_2_f(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::<RangeMinRangeUpdate<_>>::new(n);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, s, t, x: i32);
                seg.update(s, t + 1, Some(x));
            }
            1 => {
                scan!(scanner, s, t);
                writeln!(writer, "{}", seg.fold(s, t + 1)).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
