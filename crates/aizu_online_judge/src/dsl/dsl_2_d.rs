use competitive::prelude::*;
use competitive::{algebra::RangeMinRangeUpdate, data_structure::LazySegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { s: usize, t: usize, x: i32 }
        1 => Get { i: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_D")]
pub fn dsl_2_d(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut seg = LazySegmentTree::<RangeMinRangeUpdate<_>>::new(n);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { s, t, x } => {
                seg.update(s..t + 1, Some(x));
            }
            Query::Get { i } => {
                pp!(seg.fold(i..i + 1));
            }
        }
    }
}
