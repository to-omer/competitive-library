use competitive::prelude::*;
use competitive::{algebra::MinOperation, data_structure::SegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { x: usize, y: usize }
        1 => Fold { x: usize, y: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_A")]
pub fn dsl_2_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut seg = SegmentTree::<MinOperation<_>>::new(n);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { x, y } => {
                seg.set(x, y as i32);
            }
            Query::Fold { x, y } => {
                pp!(seg.fold(x..=y));
            }
        }
    }
}
