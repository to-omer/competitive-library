// verify-helper: PROBLEM https://judge.yosupo.jp/problem/point_add_range_sum

use competitive_library::algebra::operations::AdditiveOperation;
use competitive_library::data_structure::binary_indexed_tree::BinaryIndexedTree;
use competitive_library::{input, input_inner};
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, n, q, a: [i64; n] };
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    for i in 0..n {
        bit.update(i + 1, a[i]);
    }
    for _ in 0..q {
        input_inner! { iter, ty };
        if ty == 0 {
            input_inner! { iter, p, x: i64 };
            bit.update(p + 1, x);
        } else {
            input_inner! { iter, l, r };
            writeln!(out, "{}", bit.fold(l, r))?;
        }
    }

    Ok(())
}
