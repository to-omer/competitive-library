use competitive::prelude::*;
use competitive::{
    algebra::LinearOperation,
    data_structure::{
        AffineFraction, DaryInvertibleCompositeKernel, DarySegmentTree, InvertibleAffine,
        SegmentTree,
    },
    num::mint_basic::MInt998244353 as M,
};

#[codesnip::entry("Query", include("MInt"))]
competitive::define_enum_scan! {
    enum Query: usize {
        0 => Set { p: usize, cd: (M, M) }
        1 => Apply { l: usize, r: usize, x: M }
    }
}

#[verify::library_checker(
    "point_set_range_composite",
    include("LinearOperation", "Query", "SegmentTree")
)]
pub fn point_set_range_composite(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, ab: [(M, M); n]);
    let mut seg = SegmentTree::<LinearOperation<_>>::from_vec(ab);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Set { p, cd } => {
                seg.set(p, cd);
            }
            Query::Apply { l, r, x } => {
                let (a, b) = seg.fold(l..r);
                pp!(a * x + b);
            }
        }
    }
}

#[verify::library_checker(
    "point_set_range_composite",
    include("DaryInvertibleCompositeKernel"),
    huge_pages
)]
pub fn point_set_range_composite_dary(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, initial: [u32; 2 * n; capacity 2 * n], queries: [u32; 4 * q; capacity 4 * q]);
    let (initial, _): (&[[u32; 2]], _) = initial.as_chunks();
    let (queries, _): (&[[u32; 4]], _) = queries.as_chunks();
    let count = queries.iter().filter(|query| query[0] == 0).count();
    let mut keys = Vec::with_capacity(n + count);
    keys.extend(
        initial
            .iter()
            .map(|ab| InvertibleAffine::new(M::new_unchecked(ab[0]), M::new_unchecked(ab[1]))),
    );
    keys.extend(queries.iter().filter(|query| query[0] == 0).map(|query| {
        InvertibleAffine::new(M::new_unchecked(query[2]), M::new_unchecked(query[3]))
    }));
    InvertibleAffine::prepare(&mut keys);
    let mut seg =
        DarySegmentTree::<DaryInvertibleCompositeKernel>::from_keys(keys[..n].iter().copied());
    let mut update = n;
    let answers =
        AffineFraction::apply_many(queries.iter().enumerate().filter_map(|(i, query)| {
            if let Some(next) = queries.get(i + 4) {
                seg.prefetch(next[1] as usize);
                if next[0] == 1 {
                    seg.prefetch(next[2] as usize);
                }
            }
            if query[0] == 0 {
                seg.set(query[1] as usize, keys[update]);
                update += 1;
                None
            } else {
                Some((
                    seg.fold_by_prefix(query[1] as usize..query[2] as usize),
                    M::new_unchecked(query[3]),
                ))
            }
        }));
    for value in answers {
        pp!(value);
    }
}
