use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeSumRangeAdd, data_structure::LazySegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { s: usize, t: usize, x: u64 }
        1 => Sum { s: usize, t: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_G")]
pub fn dsl_2_g(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::<RangeSumRangeAdd<_>>::from_vec(vec![(0, 1); n]);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { s, t, x } => {
                seg.update(s - 1..t, x);
            }
            Query::Sum { s, t } => {
                writeln!(writer, "{}", seg.fold(s - 1..t).0).ok();
            }
        }
    }
}
