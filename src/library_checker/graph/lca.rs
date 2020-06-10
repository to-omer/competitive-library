pub use crate::graph::graph::Graph;
pub use crate::scan;
pub use crate::tools::scanner::{read_all, Scanner};
pub use crate::tree::euler_tour::EulerTourForRichVertex;
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/lca")]
pub fn lca(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize; n - 1], uv: [(usize, usize); q]);
    let mut graph = Graph::new(n);
    for v in 0..n - 1 {
        graph.add_undirected_edge(v + 1, p[v]);
    }
    let mut euler = EulerTourForRichVertex::new(n);
    euler.vertex_tour(0, n, &graph);
    let lca = euler.gen_lca(&graph);
    for (u, v) in uv.into_iter() {
        writeln!(writer, "{}", lca.lca(u, v))?;
    }
    Ok(())
}
