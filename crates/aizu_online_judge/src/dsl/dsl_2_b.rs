use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, data_structure::SegmentTree};

#[verify::aizu_online_judge("DSL_2_B")]
pub fn dsl_2_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = SegmentTree::<AdditiveOperation<_>>::new(n);
    for _ in 0..q {
        scan!(scanner, ty, x, y);
        if ty == 0 {
            seg.update(x - 1, y as i32);
        } else {
            writeln!(writer, "{}", seg.fold(x - 1, y)).ok();
        }
    }
}
