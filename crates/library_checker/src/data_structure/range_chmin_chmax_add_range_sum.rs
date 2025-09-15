use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::{RangeChminChmaxAdd, RangeSumRangeChminChmaxAdd},
    data_structure::LazySegmentTree,
    num::Saturating,
};

#[verify::library_checker("range_chmin_chmax_add_range_sum")]
pub fn range_chmin_chmax_add_range_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [Saturating<i64>; n]);
    let mut seg = LazySegmentTree::<RangeSumRangeChminChmaxAdd<Saturating<i64>>>::from_vec(
        a.iter()
            .map(|&a| RangeSumRangeChminChmaxAdd::single(a, Saturating(1)))
            .collect(),
    );
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, l, r, b: Saturating<i64>);
                seg.update(l..r, RangeChminChmaxAdd::chmin(b));
            }
            1 => {
                scan!(scanner, l, r, b: Saturating<i64>);
                seg.update(l..r, RangeChminChmaxAdd::chmax(b));
            }
            2 => {
                scan!(scanner, l, r, b: Saturating<i64>);
                seg.update(l..r, RangeChminChmaxAdd::add(b));
            }
            3 => {
                scan!(scanner, l, r);
                writeln!(writer, "{}", seg.fold(l..r).sum).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
