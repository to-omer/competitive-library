pub use crate::algebra::{AdditiveOperation, CartesianOperation};
pub use crate::data_structure::LazySegmentTree;
pub use crate::graph::Graph;
use crate::scan;
use crate::tools::{read_all, Scanner};
pub use crate::tree::HeavyLightDecomposition;
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_E")]
pub fn grl_5_e(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
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
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    let monoid = CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new());
    let mut seg = LazySegmentTree::from_vec(
        vec![(0u64, 1u64); n],
        monoid.clone(),
        AdditiveOperation::new(),
        |x, &y| (x.0 + y * x.1, x.1),
    );

    scan!(scanner, q);
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, v, w: u64);
            hld.update(0, v, true, |l, r| seg.update(l, r, w));
        } else {
            scan!(scanner, u);
            let ans = hld.query(0, u, true, |l, r| seg.fold(l, r), &monoid).0;
            writeln!(writer, "{}", ans)?;
        }
    }

    Ok(())
}
