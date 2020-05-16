pub use crate::graph::maximum_flow::{Dinic, RevEdge};
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/bipartitematching")]
pub fn bipartitematching(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let (l, r, m): (usize, usize, usize) = scanner.scan();
    let ab: Vec<(usize, usize)> = scanner.scan_vec(m);
    let mut dinic = Dinic::new(l + r + 2);
    for a in 0..l {
        dinic.add_edge(0, a + 1, 1);
    }
    for b in 0..r {
        dinic.add_edge(l + b + 1, l + r + 1, 1);
    }
    for (a, b) in ab.into_iter() {
        dinic.add_edge(a + 1, l + b + 1, 1);
    }
    let f = dinic.maximum_flow(0, l + r + 1);
    writeln!(writer, "{}", f)?;
    for a in 0..l {
        for i in 0..dinic.graph[a + 1].len() {
            let RevEdge { to, cap, rev: _ } = dinic.graph[a + 1][i];
            if l < to && cap == 0 {
                writeln!(writer, "{} {}", a, to - l - 1)?;
            }
        }
    }
    Ok(())
}
