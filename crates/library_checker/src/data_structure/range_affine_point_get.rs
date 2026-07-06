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
pub fn range_affine_point_get(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [MInt998244353; n]);
    let mut seg = LazySegmentTree::<RangeSumRangeLinear<_>>::from_keys(a.into_iter());
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Update { l, r, bc } => seg.update(l..r, bc),
            Query::Get { i } => {
                writeln!(writer, "{}", seg.fold(i..i + 1).0).ok();
            }
        };
    }
}
