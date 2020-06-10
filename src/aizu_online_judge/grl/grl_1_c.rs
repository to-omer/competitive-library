pub use crate::algebra::magma::Monoid;
pub use crate::algebra::operations::AdditiveOperation;
pub use crate::graph::graph::Graph;
pub use crate::scan;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_C")]
pub fn grl_1_c(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, edges: [(usize, usize, i64); es]);
    let mut graph = Graph::new(vs);
    for &(s, t, _) in edges.iter() {
        graph.add_edge(s, t);
    }
    let cost = graph.warshall_floyd(AdditiveOperation::new(), |eid| edges[eid].2);
    if graph.vertices().any(|u| cost[u][u].unwrap() < 0) {
        writeln!(writer, "NEGATIVE CYCLE")?;
    } else {
        for u in graph.vertices() {
            for v in graph.vertices() {
                match cost[u][v] {
                    Some(d) => write!(writer, "{}", d),
                    None => write!(writer, "INF"),
                }?;
                write!(writer, "{}", if v + 1 == vs { '\n' } else { ' ' })?;
            }
        }
    }

    Ok(())
}
