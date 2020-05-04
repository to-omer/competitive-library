// verify-helper: PROBLEM https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_E

use competitive::algebra::effect::AnyMonoidEffect;
use competitive::algebra::operations::{AdditiveOperation, CartesianOperation};
use competitive::data_structure::lazy_segment_tree::LazySegmentTree;
use competitive::{input, input_inner};
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, n, q };
    let mut seg = LazySegmentTree::from_vec(
        vec![(0, 1); n],
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        AnyMonoidEffect::new(
            AdditiveOperation::new(),
            |x: &(u64, u64), &y| (x.0 + x.1 * y, x.1),
        )
    );
    for _ in 0..q {
        input_inner! { iter, x };
        if x == 0 {
            input_inner! { iter, s, t, x: u64 };
            seg.update(s - 1, t, x);
        } else {
            input_inner! { iter, i };
            writeln!(out, "{}", seg.fold(i - 1, i).0)?;
        }
    }

    Ok(())
}
