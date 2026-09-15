use competitive::prelude::*;
use competitive::{algebra::RangeSumRangeUpdate, data_structure::LazySegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { s: usize, t: usize, x: i64 }
        1 => Sum { s: usize, t: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_I")]
pub fn dsl_2_i(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut seg = LazySegmentTree::<RangeSumRangeUpdate<_>>::from_vec(vec![(0, 1); n]);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { s, t, x } => {
                seg.update(s..t + 1, Some(x));
            }
            Query::Sum { s, t } => {
                pp!(seg.fold(s..t + 1).0);
            }
        }
    }
}
