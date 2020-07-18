pub use crate::algebra::{AdditiveOperation, Monoid};
pub use crate::graph::{Graph, GraphScanner};
pub use crate::scan;
pub use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_B")]
pub fn grl_1_b(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r);
    let (graph, d) = scanner.mscan(GraphScanner::<usize, i64>::new(vs, es, true));
    let (cost, is_neg) = graph.bellman_ford(r, AdditiveOperation::new(), |eid| d[eid]);
    if is_neg {
        writeln!(writer, "NEGATIVE CYCLE")?;
    } else {
        for u in graph.vertices() {
            match cost[u] {
                Some(d) => writeln!(writer, "{}", d),
                None => writeln!(writer, "INF"),
            }?;
        }
    }

    Ok(())
}
