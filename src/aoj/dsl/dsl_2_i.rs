// verify-helper: PROBLEM https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_I

use competitive::algebra::effect::AnyMonoidEffect;
use competitive::algebra::operations::{AdditiveOperation, CartesianOperation, LastOperation};
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
            LastOperation::new(),
            |x: &(i64, i64), y| (x.1 * y.unwrap_or(x.0), x.1)
        )
    );
    for _ in 0..q {
        input_inner! { iter, x };
        if x == 0 {
            input_inner! { iter, s, t, x: i64 };
            seg.update(s, t + 1, Some(x));
        } else {
            input_inner! { iter, s, t };
            writeln!(out, "{}", seg.fold(s, t + 1).0)?;
        }
    }

    Ok(())
}
