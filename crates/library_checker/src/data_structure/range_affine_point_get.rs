use competitive::prelude::*;
use competitive::{
    algebra::LinearAct,
    data_structure::{DaryAffinePointKernel, DaryDualSegmentTree, DualSegmentTree},
    num::mint_basic::MInt998244353 as M,
};

#[codesnip::entry("Query", include("MInt"))]
competitive::define_enum_scan! {
    enum Query: u32 {
        0 => Update { l: u32, r: u32, bc: (M, M) }
        1 => Get { i: u32 }
    }
}

#[verify::library_checker("range_affine_point_get", include("DualSegmentTree", "Query"))]
pub fn range_affine_point_get(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [M; iter n]);
    let mut seg = DualSegmentTree::<LinearAct<_>>::from_keys(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { l, r, bc } => seg.update(l as usize..r as usize, bc),
            Query::Get { i } => {
                pp!(seg.get(i as usize));
            }
        };
    }
}

#[verify::library_checker("range_affine_point_get", include("DaryAffinePointKernel"), huge_pages)]
pub fn range_affine_point_get_dary(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, input: [u32; n + 2 * q; capacity n + 2 * q]);
    let (a, tokens) = input.split_at(n);
    let mut seg = DaryDualSegmentTree::<DaryAffinePointKernel>::from_keys(
        a.iter().copied().map(M::new_unchecked),
    );
    let mut tokens = tokens
        .iter()
        .copied()
        .chain(std::iter::repeat_with(|| sv!(u32)));
    let queries: Vec<_> = (0..q)
        .map(|_| {
            let mut read = || tokens.next().unwrap();
            let t = read();
            let l = read();
            if t == 0 {
                [t, l, read(), read(), read()]
            } else {
                [t, l, 0, 0, 0]
            }
        })
        .collect();
    drop(input);
    for (i, &[t, l, r, a, b]) in queries.iter().enumerate() {
        if let Some(&[t, l, r, ..]) = queries.get(i + 4) {
            seg.prefetch(l as usize);
            if t == 0 {
                seg.prefetch(r as usize - 1);
            }
        }
        if t == 0 {
            seg.update(
                l as usize..r as usize,
                (M::new_unchecked(a), M::new_unchecked(b)),
            );
        } else {
            pp!(seg.get(l as usize));
        }
    }
}
