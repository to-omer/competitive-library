use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation,
    data_structure::{
        BinaryIndexedTree, DaryAddKernelI64, DaryPrefixGroup, DarySegmentTree, SegmentTree,
        SimdAddI64, SimdBinaryIndexedTree,
    },
};

#[codesnip::entry("Query")]
competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { p: usize, x: i64 }
        1 => Sum { l: usize, r: usize }
    }
}

#[verify::library_checker(
    "point_add_range_sum",
    include("AdditiveOperation", "BinaryIndexedTree", "Query")
)]
pub fn point_add_range_sum_binary_indexed_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; n]);
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::from_slice(&a);
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

#[verify::library_checker(
    "point_add_range_sum",
    include("AdditiveOperation", "Query", "SegmentTree")
)]
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

#[verify::library_checker("point_add_range_sum", include("DaryNumericKernel", "DarySegmentTree"))]
pub fn point_add_range_sum_dary(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u32; iter n]);
    let mut seg = DarySegmentTree::<DaryAddKernelI64>::from_keys(a.map(i64::from));
    sc!(queries: [u32; 3 * q; capacity 3 * q]);
    for &[kind, index, value] in queries.as_chunks().0 {
        let index = index as usize;
        match kind {
            0 => seg.update(index, i64::from(value)),
            1 => pp!(seg.fold_by_prefix(index..value as usize)),
            _ => unreachable!(),
        }
    }
}

#[verify::library_checker("point_add_range_sum", include("SimdAddI64", "DaryPrefixGroup"))]
pub fn point_add_range_sum_simd_prefix_group(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u32; iter n]);
    let mut seg = DarySegmentTree::<DaryPrefixGroup<SimdAddI64>>::from_keys(
        a.map(|x| std::num::Wrapping(i64::from(x))),
    );
    sc!(queries: [u32; 3 * q; capacity 3 * q]);
    let (queries, _) = queries.as_chunks();
    for (i, &[kind, index, value]) in queries.iter().enumerate() {
        if let Some(&[kind, index, value]) = queries.get(i + 4) {
            seg.prefetch(index as usize);
            let mask = 0u32.wrapping_sub(kind);
            let other = (index & !mask) | (value & mask);
            seg.prefetch(other as usize);
        }
        let index = index as usize;
        match kind {
            0 => seg.update(index, std::num::Wrapping(i64::from(value))),
            1 => pp!(seg.fold_by_prefix(index..value as usize).0),
            _ => unreachable!(),
        }
    }
}

#[verify::library_checker("point_add_range_sum", include("SimdAddI64", "SimdBinaryIndexedTree"))]
pub fn point_add_range_sum_simd_monoid_fenwick(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    sc!(a: [u32; iter n]);
    let mut tree =
        SimdBinaryIndexedTree::<SimdAddI64>::from_keys(a.map(|x| std::num::Wrapping(i64::from(x))));
    sc!(queries: [u32; 3 * q; capacity 3 * q]);
    let (queries, _) = queries.as_chunks();
    for (i, &[kind, index, value]) in queries.iter().enumerate() {
        if let Some(&[kind, index, value]) = queries.get(i + 8) {
            tree.prefetch(index as usize);
            let mask = 0u32.wrapping_sub(kind);
            let other = (index & !mask) | (value & mask);
            tree.prefetch(other as usize);
        }
        let index = index as usize;
        match kind {
            0 => tree.update(index, std::num::Wrapping(i64::from(value))),
            1 => pp!(tree.fold(index, value as usize).0),
            _ => unreachable!(),
        }
    }
}
