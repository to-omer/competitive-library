pub use crate::algebra::effect::AnyMonoidEffect;
pub use crate::algebra::operations::{AdditiveOperation, CartesianOperation, LinearOperation};
pub use crate::data_structure::lazy_segment_tree::LazySegmentTree;
pub use crate::math::modu32::{modulos::Modulo998244353, Modu32};
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

type M = Modu32<Modulo998244353>;

#[verify_attr::verify("https://judge.yosupo.jp/problem/range_affine_range_sum")]
pub fn range_affine_range_sum(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let n: usize = scanner.scan();
    let q: usize = scanner.scan();
    let a: Vec<M> = scanner.scan_vec(n);
    let mut seg = LazySegmentTree::from_vec(
        a.into_iter().map(|x| (x, 1u32)).collect::<_>(),
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        AnyMonoidEffect::<_, (_, u32), _>::new(LinearOperation::new(), |x, y| {
            (y.0 * x.0 + y.1 * M::new(x.1), x.1)
        }),
    );
    for _ in 0..q {
        let ty: usize = scanner.scan();
        if ty == 0 {
            let (l, r, bc): (usize, usize, (M, M)) = scanner.scan();
            seg.update(l, r, bc);
        } else {
            let (l, r): (usize, usize) = scanner.scan();
            writeln!(writer, "{}", seg.fold(l, r).0)?;
        }
    }
    Ok(())
}
