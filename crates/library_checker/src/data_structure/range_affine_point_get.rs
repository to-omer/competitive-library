use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeLinear, data_structure::LazySegmentTree, num::mint_basic::MInt998244353,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { l: usize, r: usize, bc: (MInt998244353, MInt998244353) }
        1 => Get { i: usize }
    }
}

#[verify::library_checker("range_affine_point_get")]
pub fn range_affine_point_get(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [MInt998244353; n]);
    let mut seg = LazySegmentTree::<RangeSumRangeLinear<_>>::from_keys(a.into_iter());
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { l, r, bc } => seg.update(l..r, bc),
            Query::Get { i } => {
                pp!(seg.fold(i..i + 1).0);
            }
        };
    }
}
