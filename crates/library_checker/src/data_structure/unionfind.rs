use competitive::data_structure::UnionFind;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Unite { u: usize, v: usize }
        1 => Same { u: usize, v: usize }
    }
}

#[verify::library_checker("unionfind")]
pub fn unionfind(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut uf = UnionFind::new(n);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Unite { u, v } => {
                uf.unite(u, v);
            }
            Query::Same { u, v } => {
                pp!(uf.same(u, v) as usize);
            }
        }
    }
}
