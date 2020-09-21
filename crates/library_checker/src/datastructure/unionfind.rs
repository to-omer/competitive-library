#[doc(no_inline)]
pub use competitive::data_structure::UnionFind;
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/unionfind")]
pub fn unionfind(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = UnionFind::new(n);
    for _ in 0..q {
        scan!(scanner, ty, u, v);
        if ty == 0 {
            uf.unite(u, v);
        } else {
            writeln!(writer, "{}", uf.same(u, v) as usize).ok();
        }
    }
}
