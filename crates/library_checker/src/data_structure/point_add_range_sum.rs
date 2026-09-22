use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation,
    data_structure::{BinaryIndexedTree, SegmentTree},
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { p: usize, x: i64 }
        1 => Sum { l: usize, r: usize }
    }
}

#[verify::library_checker("point_add_range_sum")]
pub fn point_add_range_sum_binary_indexed_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; iter n]);
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(n);
    for (i, a) in a.enumerate() {
        bit.update(i, a);
    }
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { p, x } => {
                bit.update(p, x);
            }
            Query::Sum { l, r } => {
                pp!(bit.fold(l, r));
            }
        }
    }
}

#[verify::library_checker("point_add_range_sum")]
pub fn point_add_range_sum_segment_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; n]);
    let mut seg = SegmentTree::<AdditiveOperation<_>>::from_vec(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { p, x } => {
                seg.update(p, x);
            }
            Query::Sum { l, r } => {
                pp!(seg.fold(l..r));
            }
        }
    }
}
