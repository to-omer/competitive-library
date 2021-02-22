use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::TreeGraphScanner,
    tree::HeavyLightDecomposition,
};

#[verify::verify("https://judge.yosupo.jp/problem/vertex_add_path_sum")]
pub fn vertex_add_path_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n], (mut graph, _): { TreeGraphScanner::<usize, ()>::new(n) });
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    let monoid = AdditiveOperation::new();
    let mut bit = BinaryIndexedTree::new(n, monoid);
    for (i, a) in a.iter().cloned().enumerate() {
        bit.update(hld.vidx[i], a);
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
            )
            .ok();
        }
    }
}
