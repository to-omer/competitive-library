pub use crate::data_structure::union_find::UnionFind;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/unionfind")]
pub fn unionfind(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let n: usize = scanner.scan();
    let q: usize = scanner.scan();
    let mut uf = UnionFind::new(n);
    for _ in 0..q {
        let (ty, u, v): (usize, usize, usize) = scanner.scan();
        if ty == 0 {
            uf.unite(u, v);
        } else {
            writeln!(writer, "{}", uf.same(u, v) as usize)?;
        }
    }
    Ok(())
}
