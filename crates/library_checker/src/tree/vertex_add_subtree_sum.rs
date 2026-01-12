use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::SegmentTree, graph::UndirectedSparseGraph,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { u: usize, x: u64 }
        1 => Sum { u: usize }
    }
}

#[verify::library_checker("vertex_add_subtree_sum")]
pub fn vertex_add_subtree_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], p: [usize]);
    let edges = p.take(n - 1).enumerate().map(|(i, p)| (i + 1, p)).collect();
    let tree = UndirectedSparseGraph::from_edges(n, edges);
    let (et, b) = tree.subtree_euler_tour_builder(0).build_with_rearrange(&a);
    let mut seg = SegmentTree::<AdditiveOperation<_>>::from_vec(b);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { u, x } => {
                et.update(u, x, |k, x| seg.update(k, x));
            }
            Query::Sum { u } => {
                writeln!(writer, "{}", et.fold(u, |r| seg.fold(r))).ok();
            }
        }
    }
}
