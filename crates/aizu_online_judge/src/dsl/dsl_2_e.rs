use competitive::prelude::*;
use competitive::{algebra::RangeSumRangeAdd, data_structure::LazySegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { s: usize, t: usize, x: u64 }
        1 => Get { i: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_E")]
pub fn dsl_2_e(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut seg = LazySegmentTree::<RangeSumRangeAdd<_>>::from_vec(vec![(0, 1); n]);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { s, t, x } => {
                seg.update(s - 1..t, x);
            }
            Query::Get { i } => {
                pp!(seg.fold(i - 1..i).0);
            }
        }
    }
}
