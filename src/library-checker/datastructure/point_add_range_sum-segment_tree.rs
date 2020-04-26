// verify-helper: PROBLEM https://judge.yosupo.jp/problem/point_add_range_sum

use competitive_library::algebra::operations::AdditiveOperation;
use competitive_library::data_structure::segment_tree::SegmentTree;
use competitive_library::{input, input_inner};
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, n, q, a: [u64; n] };
    let mut seg = SegmentTree::from_vec(a, AdditiveOperation::new());
    for _ in 0..q {
        input_inner! { iter, ty };
        if ty == 0 {
            input_inner! { iter, p, x: u64 };
            seg.update(p, x);
        } else {
            input_inner! { iter, l, r };
            writeln!(out, "{}", seg.fold(l, r))?;
        }
    }

    Ok(())
}
