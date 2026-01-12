#[doc(no_inline)]
pub use competitive::data_structure::UnionFind;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Unite { x: usize, y: usize }
        1 => Same { x: usize, y: usize }
    }
}

#[verify::aizu_online_judge("DSL_1_A")]
pub fn dsl_1_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = UnionFind::new(n);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Unite { x, y } => {
                uf.unite(x, y);
            }
            Query::Same { x, y } => {
                writeln!(writer, "{}", (uf.same(x, y) as usize)).ok();
            }
        }
    }
}
