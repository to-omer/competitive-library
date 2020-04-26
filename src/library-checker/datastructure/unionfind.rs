// verify-helper: PROBLEM https://judge.yosupo.jp/problem/unionfind

use competitive_library::data_structure::union_find::UnionFind;
use competitive_library::{input, input_inner};
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, n, q };
    let mut uf = UnionFind::new(n);
    for _ in 0..q {
        input_inner! { iter, x, u, v };
        if x == 0 {
            uf.unite(u, v);
        } else {
            writeln!(out, "{}", uf.same(u, v) as usize)?;
        }
    }

    Ok(())
}
