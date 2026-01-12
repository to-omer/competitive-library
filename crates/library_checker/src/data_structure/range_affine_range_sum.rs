use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::RangeSumRangeLinear,
    data_structure::LazySegmentTree,
    num::{MInt, One, mint_basic::MInt998244353},
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { l: usize, r: usize, bc: (MInt998244353, MInt998244353) }
        1 => Fold { l: usize, r: usize }
    }
}

#[verify::library_checker("range_affine_range_sum")]
pub fn range_affine_range_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [MInt998244353]);
    let mut seg = LazySegmentTree::<RangeSumRangeLinear<_>>::from_vec(
        a.take(n).map(|x| (x, MInt998244353::one())).collect::<_>(),
    );
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Update { l, r, bc } => {
                seg.update(l..r, bc);
            }
            Query::Fold { l, r } => {
                writeln!(writer, "{}", seg.fold(l..r).0).ok();
            }
        }
    }
}
