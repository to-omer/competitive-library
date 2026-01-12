#[doc(no_inline)]
pub use competitive::data_structure::UnionFind;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Unite { u: usize, v: usize }
        1 => Same { u: usize, v: usize }
    }
}

#[verify::library_checker("unionfind")]
pub fn unionfind(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = UnionFind::new(n);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Unite { u, v } => {
                uf.unite(u, v);
            }
            Query::Same { u, v } => {
                writeln!(writer, "{}", uf.same(u, v) as usize).ok();
            }
        }
    }
}
