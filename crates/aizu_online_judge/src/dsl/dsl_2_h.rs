use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeMinRangeAdd, data_structure::LazySegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { s: usize, t: usize, x: i64 }
        1 => Min { s: usize, t: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_H")]
pub fn dsl_2_h(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::<RangeMinRangeAdd<_>>::from_vec(vec![0; n]);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { s, t, x } => {
                seg.update(s..t + 1, x);
            }
            Query::Min { s, t } => {
                writeln!(writer, "{}", seg.fold(s..t + 1)).ok();
            }
        }
    }
}
