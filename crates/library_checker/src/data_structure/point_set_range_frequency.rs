#[doc(no_inline)]
pub use competitive::data_structure::RangeFrequency;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Set { k: usize, v: i32 }
        1 => Query { l: usize, r: usize, x: i32 }
    }
}

#[verify::library_checker("point_set_range_frequency")]
pub fn point_set_range_frequency(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i32; n]);
    let mut rf = RangeFrequency::new(a);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Set { k, v } => {
                rf.set(k, v);
            }
            Query::Query { l, r, x } => {
                rf.query(l, r, x);
            }
        }
    }
    let results = rf.execute();
    iter_print!(writer, @lf @it results);
}
