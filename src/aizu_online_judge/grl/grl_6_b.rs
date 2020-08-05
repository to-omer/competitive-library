pub use crate::algebra::AdditiveOperation;
pub use crate::graph::PrimalDual;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/6/GRL_6_B")]
pub fn grl_6_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, f: u64);
    let mut pri = PrimalDual::new(vs);
    for _ in 0..es {
        scan!(scanner, u, v, c: u64, d: i64);
        pri.add_edge(u, v, c, d);
    }
    writeln!(
        writer,
        "{}",
        pri.minimum_cost_flow(0, vs - 1, f).unwrap_or(-1)
    )
    .ok();
}
