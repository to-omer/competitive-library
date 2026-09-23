use competitive::prelude::*;
use competitive::{
    algebra::LinearAct, data_structure::DualSegmentTree, num::mint_basic::MInt998244353 as M,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { l: usize, r: usize, bc: (M, M) }
        1 => Get { i: usize }
    }
}

#[verify::library_checker("range_affine_point_get")]
pub fn range_affine_point_get(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [M; iter n]);
    let mut seg = DualSegmentTree::<LinearAct<_>>::from_keys(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { l, r, bc } => seg.update(l..r, bc),
            Query::Get { i } => {
                pp!(seg.get(i));
            }
        };
    }
}
