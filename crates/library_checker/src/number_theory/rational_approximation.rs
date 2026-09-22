use competitive::prelude::*;
use competitive::{algorithm::rational_binary_search, num::URational};

#[verify::library_checker("rational_approximation")]
pub fn rational_approximation(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t);
    for _ in 0..t {
        sc!(n: u64, x: u64, y: u64);
        let x = URational::new_unchecked(x, y);
        let sbt = rational_binary_search::<u64>(|&a| a <= x, n);
        if matches!(sbt.l.cmp(&x), std::cmp::Ordering::Equal) {
            pp!(sbt.l.num, sbt.l.den, sbt.l.num, sbt.l.den);
        } else {
            pp!(sbt.l.num, sbt.l.den, sbt.r.num, sbt.r.den);
        }
    }
}
