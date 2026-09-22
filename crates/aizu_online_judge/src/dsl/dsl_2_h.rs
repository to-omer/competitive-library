use competitive::prelude::*;
use competitive::{algebra::RangeMinRangeAdd, data_structure::LazySegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { s: usize, t: usize, x: i64 }
        1 => Min { s: usize, t: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_H")]
pub fn dsl_2_h(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut seg = LazySegmentTree::<RangeMinRangeAdd<_>>::from_keys(std::iter::repeat_n(0, n));
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { s, t, x } => {
                seg.update(s..t + 1, x);
            }
            Query::Min { s, t } => {
                pp!(seg.fold(s..t + 1));
            }
        }
    }
}
