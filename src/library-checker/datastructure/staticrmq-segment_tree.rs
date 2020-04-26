// verify-helper: PROBLEM https://judge.yosupo.jp/problem/staticrmq

use competitive_library::algebra::operations::MinOperation;
use competitive_library::data_structure::segment_tree::SegmentTree;
use competitive_library::input;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { n, q, a: [u64; n], lr: [(usize, usize); q] };
    let seg = SegmentTree::from_vec(a, MinOperation::new());
    for (l, r) in lr.into_iter() {
        writeln!(out, "{}", seg.fold(l, r))?;
    }

    Ok(())
}
