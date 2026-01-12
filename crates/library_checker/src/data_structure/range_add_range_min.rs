use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeMinRangeAdd, data_structure::LazySegmentTree};

competitive::define_enum_scan! {
    enum Query: u8 {
        0 => Add { l: usize, r: usize, x: i64 }
        1 => Min { l: usize, r: usize }
    }
}

#[verify::library_checker("range_add_range_min")]
pub fn range_add_range_min(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let mut seg = LazySegmentTree::<RangeMinRangeAdd<i64>>::from_vec(a);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { l, r, x } => {
                seg.update(l..r, x);
            }
            Query::Min { l, r } => {
                let ans = seg.fold(l..r);
                writeln!(writer, "{}", ans).ok();
            }
        }
    }
}
