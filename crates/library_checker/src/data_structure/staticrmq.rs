use competitive::prelude::*;
use competitive::{
    algebra::MinOperation,
    data_structure::{DisjointSparseTable, RangeMinimumQuery, SegmentTree, StaticRangeProduct},
};

#[verify::library_checker("staticrmq")]
pub fn staticrmq_disjoint_sparse_table(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u64; n], lr: [(usize, usize); iter q]);
    let table = DisjointSparseTable::<MinOperation<_>>::new(a);
    for (l, r) in lr {
        pp!(table.fold(l, r));
    }
}

#[verify::library_checker("staticrmq")]
pub fn staticrmq_segment_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u64; n], lr: [(usize, usize); iter q]);
    let seg = SegmentTree::<MinOperation<_>>::from_vec(a);
    for (l, r) in lr {
        pp!(seg.fold(l..r));
    }
}

#[verify::library_checker("staticrmq")]
pub fn staticrmq_range_minimum_query(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u64; n], lr: [(usize, usize); iter q]);
    let rmq = RangeMinimumQuery::new(a);
    for (l, r) in lr {
        pp!(rmq.fold(l, r));
    }
}

#[verify::library_checker("staticrmq")]
pub fn staticrmq_static_range_product(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u64; n], lr: [(usize, usize); iter q]);
    let table = StaticRangeProduct::<MinOperation<_>>::new(a);
    for (l, r) in lr {
        pp!(table.fold(l, r));
    }
}
