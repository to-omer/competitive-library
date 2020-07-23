pub use crate::algebra::AdditiveOperation;
pub use crate::data_structure::BinaryIndexedTree;
pub use crate::graph::GraphScanner;
use crate::scan;
use crate::tools::{read_all, Scanner};
pub use crate::tree::HeavyLightDecomposition;
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/vertex_add_path_sum")]
pub fn vertex_add_path_sum(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let (mut graph, _) = scanner.mscan(GraphScanner::<usize, ()>::new(n, n - 1, false));
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    let monoid = AdditiveOperation::new();
    let mut bit = BinaryIndexedTree::new(n, monoid.clone());
    for i in 0..n {
        bit.update(hld.vidx[i], a[i]);
    }
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, p, x: i64);
            bit.update(hld.vidx[p], x);
        } else {
            scan!(scanner, u, v);
            writeln!(
                writer,
                "{}",
                hld.query(u, v, false, |l, r| bit.fold(l, r), &monoid)
            )?;
        }
    }
    Ok(())
}
