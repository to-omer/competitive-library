// verify-helper: PROBLEM https://judge.yosupo.jp/problem/point_set_range_composite

use competitive::algebra::operations::LinearOperation;
use competitive::data_structure::segment_tree::SegmentTree;
use competitive::math::modu32::{modulos::Modulo998244353, Modu32};
use competitive::{input, input_inner};
use std::io::Write;

type M = Modu32<Modulo998244353>;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, n, q, ab: [(M, M); n] };
    let mut seg = SegmentTree::from_vec(ab, LinearOperation::new());
    for _ in 0..q {
        input_inner! { iter, x };
        if x == 0 {
            input_inner! { iter, p, cd: (M, M) };
            seg.set(p, cd);
        } else {
            input_inner! { iter, l, r, x: M };
            let (a, b) = seg.fold(l, r);
            writeln!(out, "{}", a * x + b)?;
        }
    }

    Ok(())
}
