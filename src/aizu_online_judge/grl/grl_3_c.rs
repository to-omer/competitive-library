pub use crate::graph::{Graph, RevGraphScanner, StronglyConnectedComponent};
pub use crate::scan;
pub use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/3/GRL_3_C")]
pub fn grl_3_c(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es);
    let (graph, _) = scanner.mscan(RevGraphScanner::<usize, ()>::new(vs, es));
    let scc = StronglyConnectedComponent::new(&graph);
    scan!(scanner, q);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", (scc[u] == scc[v]) as u32)?;
    }

    Ok(())
}
