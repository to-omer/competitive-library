use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeMinRangeAdd, data_structure::LazySegmentTree};

#[verify::library_checker("range_add_range_min")]
pub fn range_add_range_min(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let mut seg = LazySegmentTree::<RangeMinRangeAdd<i64>>::from_vec(a);
    for _ in 0..q {
        scan!(scanner, t: u8);
        if t == 0 {
            scan!(scanner, l, r, x: i64);
            seg.update(l..r, x);
        } else {
            scan!(scanner, l, r);
            let ans = seg.fold(l..r);
            writeln!(writer, "{}", ans).ok();
        }
    }
}
