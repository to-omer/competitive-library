pub use crate::algebra::magma::Monoid;
pub use crate::algebra::operations::AdditiveOperation;
pub use crate::graph::graph::Graph;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_A")]
pub fn grl_1_a(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let vs: usize = scanner.scan();
    let es: usize = scanner.scan();
    let r: usize = scanner.scan();
    let edges: Vec<(usize, usize, u64)> = scanner.scan_vec(es);

    let mut graph = Graph::new(vs);
    for &(s, t, _) in edges.iter() {
        graph.add_edge(s, t);
    }
    let cost = graph.dijkstra(r, AdditiveOperation::new(), |eid| edges[eid].2);
    for u in graph.vertices() {
        match cost[u] {
            Some(d) => writeln!(writer, "{}", d),
            None => writeln!(writer, "INF"),
        }?;
    }

    Ok(())
}
