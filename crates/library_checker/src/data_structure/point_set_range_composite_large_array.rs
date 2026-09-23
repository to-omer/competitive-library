use competitive::prelude::*;
use competitive::{
    algebra::LinearOperation,
    algorithm::SliceSortExt,
    data_structure::{SegmentTree, StaticSearch},
    num::mint_basic::MInt998244353 as M,
};

competitive::define_enum_scan! {
    #[derive(Clone, Copy)]
    enum Query: u8 {
        0 => Set { p: u32, cd: (M, M) }
        1 => Apply { l: u32, r: u32, x: M }
    }
}

#[verify::library_checker("point_set_range_composite_large_array")]
pub fn point_set_range_composite_large_array(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(_n: u32, q, queries: [Query; q]);
    let mut values: Vec<_> = queries
        .iter()
        .filter_map(|&query| match query {
            Query::Set { p, .. } => Some(p),
            _ => None,
        })
        .collect();
    values.radix_sort_by_key(|&x| x);
    values.dedup();
    let search = StaticSearch::from_sorted(&values);
    let mut seg = SegmentTree::<LinearOperation<M>>::new(values.len());
    let mut endpoints = Vec::with_capacity(2 * q);
    for &query in &queries {
        match query {
            Query::Set { p, .. } => endpoints.push(p),
            Query::Apply { l, r, .. } => endpoints.extend([l, r]),
        }
    }
    let mut positions = vec![0; endpoints.len()];
    search.lower_bound_batch(&endpoints, &mut positions);
    let mut positions = positions.into_iter();
    for query in queries {
        match query {
            Query::Set { cd, .. } => seg.set(positions.next().unwrap(), cd),
            Query::Apply { x, .. } => {
                let l = positions.next().unwrap();
                let r = positions.next().unwrap();
                let (a, b) = seg.fold(l..r);
                pp!(a * x + b);
            }
        }
    }
}
