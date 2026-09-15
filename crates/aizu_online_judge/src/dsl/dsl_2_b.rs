use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, data_structure::SegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { x: Usize1, y: usize }
        1 => Fold { x: Usize1, y: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_B")]
pub fn dsl_2_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut seg = SegmentTree::<AdditiveOperation<_>>::new(n);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { x, y } => {
                seg.update(x, y as i32);
            }
            Query::Fold { x, y } => {
                pp!(seg.fold(x..y));
            }
        }
    }
}
