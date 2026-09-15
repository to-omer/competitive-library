use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeLinear,
    data_structure::LazySegmentTree,
    num::{One, mint_basic::MInt998244353},
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { l: usize, r: usize, bc: (MInt998244353, MInt998244353) }
        1 => Fold { l: usize, r: usize }
    }
}

#[verify::library_checker("range_affine_range_sum")]
pub fn range_affine_range_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [MInt998244353]);
    let mut seg = LazySegmentTree::<RangeSumRangeLinear<_>>::from_vec(
        a.take(n).map(|x| (x, MInt998244353::one())).collect::<_>(),
    );
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
