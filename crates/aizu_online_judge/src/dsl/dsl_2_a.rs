use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::MinOperation, data_structure::SegmentTree};

#[verify::aizu_online_judge("DSL_2_A")]
pub fn dsl_2_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = SegmentTree::<MinOperation<_>>::new(n);
    for _ in 0..q {
        scan!(scanner, ty, x, y);
        if ty == 0 {
            seg.set(x, y as i32);
        } else {
            writeln!(writer, "{}", seg.fold(x, y + 1)).ok();
        }
    }
}
