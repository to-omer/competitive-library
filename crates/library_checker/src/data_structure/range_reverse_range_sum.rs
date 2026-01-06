use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::RangeSumRangeAdd, data_structure::SplaySequence};

#[verify::library_checker("range_reverse_range_sum")]
pub fn range_reverse_range_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let mut seq = SplaySequence::<RangeSumRangeAdd<i64>>::with_capacity(n);
    seq.extend(a);
    for _ in 0..q {
        scan!(scanner, t: u8);
        match t {
            0 => {
                scan!(scanner, l, r);
                seq.reverse(l..r);
            }
            1 => {
                scan!(scanner, l, r);
                let ans = seq.fold(l..r).0;
                writeln!(writer, "{}", ans).ok();
            }
            _ => unreachable!("unknown query"),
        }
    }
}
