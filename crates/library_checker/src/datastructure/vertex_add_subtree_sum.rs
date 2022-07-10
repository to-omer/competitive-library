use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::SegmentTree, graph::UndirectedSparseGraph,
    tree::EulerTourForVertex,
};

#[verify::library_checker("vertex_add_subtree_sum")]
pub fn vertex_add_subtree_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], p: [usize]);
    let edges = p.take(n - 1).enumerate().map(|(i, p)| (i + 1, p)).collect();
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let mut et = EulerTourForVertex::new(&graph);
    et.subtree_vertex_tour(0, n);
    let mut b = vec![0; n];
    for i in 0..n {
        b[et.vidx[i].0] = a[i];
    }
    let mut seg = SegmentTree::<AdditiveOperation<_>>::from_vec(b);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, u, x: u64);
                et.subtree_update(u, x, |k, x| seg.update(k, x));
            }
            1 => {
                scan!(scanner, u);
                writeln!(writer, "{}", et.subtree_query(u, |l, r| seg.fold(l, r))).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
