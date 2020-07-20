pub use crate::graph::{GraphScanner, LowLink};
pub use crate::scan;
pub use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/3/GRL_3_B")]
pub fn grl_3_b(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es);
    let (graph, _) = scanner.mscan(GraphScanner::<usize, ()>::new(vs, es, false));
    let mut bridge = LowLink::new(&graph).bridge;
    bridge.sort();
    for (u, v) in bridge.into_iter() {
        writeln!(writer, "{} {}", u, v)?;
    }

    Ok(())
}
