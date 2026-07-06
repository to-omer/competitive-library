use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeAdd,
    data_structure::{ImplicitTreap, SplaySequence},
};

competitive::define_enum_scan! {
    enum Query: u8 {
        0 => Reverse { l: usize, r: usize }
        1 => Sum { l: usize, r: usize }
    }
}

#[verify::library_checker("range_reverse_range_sum")]
pub fn range_reverse_range_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let mut seq = ImplicitTreap::<RangeSumRangeAdd<i64>>::with_capacity(n);
    seq.extend(a);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Reverse { l, r } => {
                seq.reverse(l..r);
            }
            Query::Sum { l, r } => {
                let ans = seq.fold(l..r).0;
                writeln!(writer, "{}", ans).ok();
            }
        }
    }
}

#[verify::library_checker("range_reverse_range_sum")]
pub fn range_reverse_range_sum_splay_sequence(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let mut seq = SplaySequence::<RangeSumRangeAdd<i64>>::with_capacity(n);
    seq.extend(a);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Reverse { l, r } => {
                seq.reverse(l..r);
            }
            Query::Sum { l, r } => {
                let ans = seq.fold(l..r).0;
                writeln!(writer, "{}", ans).ok();
            }
        }
    }
}
