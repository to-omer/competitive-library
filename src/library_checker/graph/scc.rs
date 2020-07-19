pub use crate::graph::{RevGraphScanner, StronglyConnectedComponent};
pub use crate::scan;
pub use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/scc")]
pub fn scc(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es);
    let (graph, _) = scanner.mscan(RevGraphScanner::<usize, ()>::new(vs, es));
    let scc = StronglyConnectedComponent::new(&graph);
    let comp = scc.components();
    writeln!(writer, "{}", comp.len())?;
    for vs in comp.into_iter() {
        write!(writer, "{}", vs.len())?;
        for v in vs.into_iter() {
            write!(writer, " {}", v)?;
        }
        writeln!(writer, "")?;
    }
    Ok(())
}
