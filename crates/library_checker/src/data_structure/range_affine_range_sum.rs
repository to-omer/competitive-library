use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::RangeSumRangeLinear,
    data_structure::LazySegmentTree,
    num::{MInt, One, mint_basic::MInt998244353},
};

#[verify::library_checker("range_affine_range_sum")]
pub fn range_affine_range_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [MInt998244353]);
    let mut seg = LazySegmentTree::<RangeSumRangeLinear<_>>::from_vec(
        a.take(n).map(|x| (x, MInt998244353::one())).collect::<_>(),
    );
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, l, r, bc: (MInt998244353, MInt998244353));
                seg.update(l..r, bc);
            }
            1 => {
                scan!(scanner, l, r);
                writeln!(writer, "{}", seg.fold(l..r).0).ok();
            }
            _ => unreachable!("unknown query"),
        }
    }
}
