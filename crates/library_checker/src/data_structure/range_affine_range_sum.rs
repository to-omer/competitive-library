use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeLinear,
    data_structure::{LazySegmentTree, SqrtLazyVec},
    num::mint_basic::MInt998244353 as M,
};

#[codesnip::entry("Query", include("MInt"))]
competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { l: usize, r: usize, bc: (M, M) }
        1 => Fold { l: usize, r: usize }
    }
}

#[verify::library_checker("range_affine_range_sum", include("LazySegmentTree", "Query"))]
pub fn range_affine_range_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [M; iter n]);
    let mut seg = LazySegmentTree::<RangeSumRangeLinear<_>>::from_keys(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { l, r, bc } => {
                seg.update(l..r, bc);
            }
            Query::Fold { l, r } => {
                pp!(seg.fold(l..r).0);
            }
        }
    }
}

#[verify::library_checker("range_affine_range_sum", include("SqrtTree", "Query"))]
pub fn range_affine_range_sum_sqrt_vec(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [M; iter n]);
    let mut seq: SqrtLazyVec<RangeSumRangeLinear<_>> = a.collect();
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { l, r, bc } => seq.update(l..r, bc),
            Query::Fold { l, r } => pp!(seq.fold(l..r).0),
        }
    }
}
