use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeLinear,
    data_structure::{DaryAffineSumKernel, DaryLazySegmentTree, LazySegmentTree},
    num::mint_basic::MInt998244353 as M,
};

#[codesnip::entry("Query", include("MInt"))]
competitive::define_enum_scan! {
    enum Query: u32 {
        0 => Update { l: u32, r: u32, bc: (M, M) }
        1 => Fold { l: u32, r: u32 }
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
                seg.update(l as usize..r as usize, bc);
            }
            Query::Fold { l, r } => {
                pp!(seg.fold(l as usize..r as usize).0);
            }
        }
    }
}

#[verify::library_checker("range_affine_range_sum", include("DaryAffineSumKernel", "Query"))]
pub fn range_affine_range_sum_dary(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [M; iter n]);
    let mut seg = DaryLazySegmentTree::<DaryAffineSumKernel>::from_keys(a);
    sc!(queries: [Query; q]);
    for (i, query) in queries.iter().enumerate() {
        if let Some(&Query::Update { l, r, .. } | &Query::Fold { l, r }) = queries.get(i + 4) {
            seg.prefetch(l as usize);
            seg.prefetch(r.saturating_sub(1) as usize);
        }
        match *query {
            Query::Update { l, r, bc } => seg.update(l as usize..r as usize, bc),
            Query::Fold { l, r } => pp!(seg.fold(l as usize..r as usize)),
        }
    }
}
