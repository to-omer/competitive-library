pub use crate::algebra::operations::LinearOperation;
pub use crate::data_structure::segment_tree::SegmentTree;
pub use crate::math::modu32::{modulos::Modulo998244353, Modu32};
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

type M = Modu32<Modulo998244353>;

#[verify_attr::verify("https://judge.yosupo.jp/problem/point_set_range_composite")]
pub fn point_set_range_composite(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let n: usize = scanner.scan();
    let q: usize = scanner.scan();
    let ab: Vec<(M, M)> = scanner.scan_vec(n);
    let mut seg = SegmentTree::from_vec(ab, LinearOperation::new());
    for _ in 0..q {
        let ty: usize = scanner.scan();
        if ty == 0 {
            let (p, cd): (usize, (M, M)) = scanner.scan();
            seg.set(p, cd);
        } else {
            let (l, r, x): (usize, usize, M) = scanner.scan();
            let (a, b) = seg.fold(l, r);
            writeln!(writer, "{}", a * x + b)?;
        }
    }
    Ok(())
}
