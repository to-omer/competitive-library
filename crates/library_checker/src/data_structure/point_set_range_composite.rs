use competitive::prelude::*;
use competitive::{
    algebra::LinearOperation, data_structure::SegmentTree, num::mint_basic::MInt998244353,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Set { p: usize, cd: (MInt998244353, MInt998244353) }
        1 => Apply { l: usize, r: usize, x: MInt998244353 }
    }
}

#[verify::library_checker("point_set_range_composite")]
pub fn point_set_range_composite(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, ab: [(MInt998244353, MInt998244353); n]);
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
