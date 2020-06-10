pub use crate::graph::strongly_connected_component::StronglyConnectedComponent;
pub use crate::scan;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/scc")]
pub fn scc(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, ab: [(usize, usize); m]);
    let mut scc = StronglyConnectedComponent::new(n);
    for (a, b) in ab.into_iter() {
        scc.add_edge(a, b);
    }
    scc.build();
    let comp = scc.component();
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
