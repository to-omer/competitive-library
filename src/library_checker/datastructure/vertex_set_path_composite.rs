pub use crate::algebra::{LinearOperation, ReverseOperation};
pub use crate::data_structure::SegmentTree;
pub use crate::graph::GraphScanner;
pub use crate::num::{modulus::Modulo998244353, MInt};
use crate::scan;
use crate::tools::{read_all, Scanner};
pub use crate::tree::HeavyLightDecomposition;
use std::io::{self, Read, Write};

type M = MInt<Modulo998244353>;

#[verify_attr::verify("https://judge.yosupo.jp/problem/vertex_set_path_composite")]
pub fn vertex_set_path_composite(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, ab: [(M, M); n]);
    let (mut graph, _) = scanner.mscan(GraphScanner::<usize, ()>::new(n, n - 1, false));
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    let monoid = LinearOperation::new();
    let mut nab = vec![(M::zero(), M::zero()); n];
    for i in 0..n {
        nab[hld.vidx[i]] = ab[i];
    }
    let mut seg1 = SegmentTree::from_vec(nab.clone(), monoid.clone());
    let mut seg2 = SegmentTree::from_vec(nab, ReverseOperation::new(monoid.clone()));
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, p, cd: (M, M));
            seg1.set(hld.vidx[p], cd);
            seg2.set(hld.vidx[p], cd);
        } else {
            scan!(scanner, u, v, x: M);
            let (a, b) = hld.query_noncom(
                u,
                v,
                false,
                |l, r| seg1.fold(l, r),
                |l, r| seg2.fold(l, r),
                &monoid,
            );
            writeln!(writer, "{}", a * x + b)?;
        }
    }

    Ok(())
}
