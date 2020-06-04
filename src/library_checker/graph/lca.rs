pub use crate::graph::graph::Graph;
pub use crate::tools::scanner::{read_all, Scanner};
pub use crate::tree::euler_tour::EulerTourForRichVertex;
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/lca")]
pub fn lca(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let n: usize = scanner.scan();
    let q: usize = scanner.scan();
    let p: Vec<usize> = scanner.scan_vec(n - 1);
    let uv: Vec<(usize, usize)> = scanner.scan_vec(q);
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
