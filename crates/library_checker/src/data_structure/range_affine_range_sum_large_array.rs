use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeLinear,
    algorithm::SliceSortExt,
    data_structure::{LazySegmentTree, StaticSearch},
    num::{Zero, mint_basic::MInt998244353 as M},
};

competitive::define_enum_scan! {
    #[derive(Clone, Copy)]
    enum Query: u8 {
        0 => Update { l: u32, r: u32, bc: (M, M) }
        1 => Fold { l: u32, r: u32 }
    }
}

#[verify::library_checker("range_affine_range_sum_large_array")]
pub fn range_affine_range_sum_large_array(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n: u32, q, queries: [Query; q]);
    let mut values: Vec<_> = [0, n]
        .into_iter()
        .chain(queries.iter().flat_map(|&query| {
            let (Query::Update { l, r, .. } | Query::Fold { l, r }) = query;
            [l, r]
        }))
        .collect();
    values.radix_sort_by_key(|&x| x);
    values.dedup();
    let search = StaticSearch::from_sorted(&values);
    let mut seg = LazySegmentTree::<RangeSumRangeLinear<M>>::from_vec(
        values
            .windows(2)
            .map(|w| (M::zero(), M::from(w[1] - w[0])))
            .collect(),
    );
    let endpoints: Vec<_> = queries
        .iter()
        .flat_map(|&query| {
            let (Query::Update { l, r, .. } | Query::Fold { l, r }) = query;
            [l, r]
        })
        .collect();
    let mut positions = vec![0; endpoints.len()];
    search.lower_bound_batch(&endpoints, &mut positions);
    for (query, &[l, r]) in queries.into_iter().zip(positions.as_chunks::<2>().0) {
        match query {
            Query::Update { bc, .. } => {
                seg.update(l..r, bc);
            }
            Query::Fold { .. } => {
                pp!(seg.fold(l..r).0);
            }
        }
    }
}
