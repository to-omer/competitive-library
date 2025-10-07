use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algorithm::rational_binary_search, num::URational};
use std::cmp::Ordering;

#[verify::library_checker("rational_approximation")]
pub fn rational_approximation(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t);
    for _ in 0..t {
        scan!(scanner, n: u64, x: u64, y: u64);
        let x = URational::new_unchecked(x, y);
        let sbt = rational_binary_search::<u64>(|&a| a <= x, n);
        if matches!(sbt.l.cmp(&x), Ordering::Equal) {
            iter_print!(writer, sbt.l.num, sbt.l.den, sbt.l.num, sbt.l.den);
        } else {
            iter_print!(writer, sbt.l.num, sbt.l.den, sbt.r.num, sbt.r.den);
        }
    }
}
