// verify-helper: PROBLEM https://judge.yosupo.jp/problem/range_affine_range_sum

use competitive::algebra::effect::AnyMonoidEffect;
use competitive::algebra::operations::{AdditiveOperation, CartesianOperation, LinearOperation};
use competitive::data_structure::lazy_segment_tree::LazySegmentTree;
use competitive::math::modu32::{modulos::Modulo998244353, Modu32};
use competitive::{input, input_inner};
use std::io::Write;

type M = Modu32<Modulo998244353>;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, n, q, a: [M; n] };
    let mut seg = LazySegmentTree::from_vec(
        a.into_iter().map(|x| (x, 1u32)).collect::<_>(),
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        AnyMonoidEffect::<_, (_, u32), _>::new(LinearOperation::new(), |x, y| {
            (y.0 * x.0 + y.1 * M::new(x.1), x.1)
        }),
    );
    for _ in 0..q {
        input_inner! { iter, x };
        if x == 0 {
            input_inner! { iter, l, r, bc: (M, M) };
            seg.update(l, r, bc);
        } else {
            input_inner! { iter, l, r };
            writeln!(out, "{}", seg.fold(l, r).0)?;
        }
    }

    Ok(())
}
