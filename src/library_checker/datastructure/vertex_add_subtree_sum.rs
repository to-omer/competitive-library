pub use crate::algebra::AdditiveOperation;
pub use crate::data_structure::SegmentTree;
pub use crate::graph::Graph;
use crate::prelude::*;
pub use crate::tree::EulerTourForVertex;

#[verify_attr::verify("https://judge.yosupo.jp/problem/vertex_add_subtree_sum")]
pub fn vertex_add_subtree_sum(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n]);
    let mut graph = Graph::new(n);
    for i in 1..n {
        graph.add_undirected_edge(i, scanner.scan::<usize>());
    }
    let mut et = EulerTourForVertex::new(n);
    et.subtree_vertex_tour(0, n, &graph);
    let mut b = vec![0; n];
    for i in 0..n {
        b[et.vidx[i].0] = a[i];
    }
    let mut seg = SegmentTree::from_vec(b, AdditiveOperation::new());
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, u, x: u64);
            et.subtree_update(u, x, |k, x| seg.update(k, x));
        } else {
            scan!(scanner, u);
            writeln!(writer, "{}", et.subtree_query(u, |l, r| seg.fold(l, r))).ok();
        }
    }
}
