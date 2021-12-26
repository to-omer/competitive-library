#[doc(no_inline)]
pub use competitive::data_structure::UnionFind;
use competitive::prelude::*;

#[cfg_attr(nightly, verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/1/DSL_1_A"))]
pub fn dsl_1_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = UnionFind::new(n);
    for _ in 0..q {
        scan!(scanner, ty, x, y);
        if ty == 0 {
            uf.unite(x, y);
        } else {
            writeln!(writer, "{}", (uf.same(x, y) as usize)).ok();
        }
    }
}
