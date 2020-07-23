pub use crate::algebra::AdditiveOperation;
pub use crate::data_structure::BinaryIndexedTree;
pub use crate::graph::Graph;
use crate::scan;
use crate::tools::{read_all, Scanner};
pub use crate::tree::EulerTourForEdge;
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_D")]
pub fn grl_5_d(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n);
    let mut graph = Graph::new(n);
    for u in graph.vertices() {
        scan!(scanner, k);
        for v in scanner.iter::<usize>().take(k) {
            graph.add_undirected_edge(u, v);
        }
    }
    let mut et = EulerTourForEdge::new(n);
    et.edge_tour(0, n, &graph);
    let mut bit = BinaryIndexedTree::new(et.len(), AdditiveOperation::new());

    scan!(scanner, q);
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, v, w: i64);
            let (l, r) = et.eidx[et.par[v]];
            bit.update(l, w);
            bit.update(r, -w);
        } else {
            scan!(scanner, u);
            let ans = if u > 0 {
                bit.accumulate(et.eidx[et.par[u]].0)
            } else {
                0
            };
            writeln!(writer, "{}", ans)?;
        }
    }

    Ok(())
}
