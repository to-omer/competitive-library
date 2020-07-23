pub use crate::graph::{RevGraph, TwoSatisfiability};
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/two_sat")]
pub fn two_sat(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(
        scanner,
        _p: chars,
        _cnf: chars,
        n,
        m,
        ab: [(i64, i64, i64); m]
    );
    let mut graph = RevGraph::new(n * 2);
    for (a, b, _) in ab.into_iter() {
        let u = (a.abs() as usize - 1) * 2;
        let v = (b.abs() as usize - 1) * 2;
        let na = (a < 0) as usize;
        let nb = (b < 0) as usize;
        TwoSatisfiability::add_inner(&mut graph, u ^ na ^ 1, v ^ nb);
    }
    if let Some(v) = TwoSatisfiability::build(n, &graph) {
        writeln!(writer, "s SATISFIABLE")?;
        write!(writer, "v")?;
        for i in 0..n {
            write!(
                writer,
                " {}",
                if v[i] { i as i64 + 1 } else { -(i as i64 + 1) }
            )?;
        }
        write!(writer, " 0")?;
    } else {
        writeln!(writer, "s UNSATISFIABLE")?;
    }
    Ok(())
}
