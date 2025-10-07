#[doc(no_inline)]
pub use competitive::data_structure::RangeFrequency;
use competitive::prelude::*;

#[verify::library_checker("point_set_range_frequency")]
pub fn point_set_range_frequency(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i32; n]);
    let mut rf = RangeFrequency::new(a);
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, k, v: i32);
            rf.set(k, v);
        } else {
            scan!(scanner, l, r, x: i32);
            rf.query(l, r, x);
        }
    }
    let results = rf.execute();
    iter_print!(writer, @lf @it results);
}
