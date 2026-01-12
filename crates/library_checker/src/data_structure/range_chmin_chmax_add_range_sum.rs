use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::{RangeChminChmaxAdd, RangeSumRangeChminChmaxAdd},
    data_structure::LazySegmentTree,
    num::Saturating,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Chmin { l: usize, r: usize, b: Saturating<i64> }
        1 => Chmax { l: usize, r: usize, b: Saturating<i64> }
        2 => Add { l: usize, r: usize, b: Saturating<i64> }
        3 => Sum { l: usize, r: usize }
    }
}

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
        scan!(scanner, query: Query);
        match query {
            Query::Chmin { l, r, b } => {
                seg.update(l..r, RangeChminChmaxAdd::chmin(b));
            }
            Query::Chmax { l, r, b } => {
                seg.update(l..r, RangeChminChmaxAdd::chmax(b));
            }
            Query::Add { l, r, b } => {
                seg.update(l..r, RangeChminChmaxAdd::add(b));
            }
            Query::Sum { l, r } => {
                writeln!(writer, "{}", seg.fold(l..r).sum).ok();
            }
        }
    }
}
