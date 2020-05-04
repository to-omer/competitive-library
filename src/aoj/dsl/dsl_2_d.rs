// verify-helper: PROBLEM https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_D

use competitive::algebra::effect::AnyMonoidEffect;
use competitive::algebra::operations::{LastOperation, MinOperation};
use competitive::data_structure::lazy_segment_tree::LazySegmentTree;
use competitive::{input, input_inner};
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, n, q };
    let mut seg = LazySegmentTree::new(
        n,
        MinOperation::new(),
        AnyMonoidEffect::new(LastOperation::new(), |&x, y| y.unwrap_or(x)),
    );
    for _ in 0..q {
        input_inner! { iter, x };
        if x == 0 {
            input_inner! { iter, s, t, x: i32 };
            seg.update(s, t + 1, Some(x));
        } else {
            input_inner! { iter, i };
            writeln!(out, "{}", seg.fold(i, i + 1))?;
        }
    }

    Ok(())
}
