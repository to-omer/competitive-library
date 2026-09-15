use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, data_structure::PotentializedUnionFind};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Unite { x: usize, y: usize, w: i64 }
        1 => Diff { x: usize, y: usize }
    }
}

#[verify::aizu_online_judge("DSL_1_B")]
pub fn dsl_1_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut uf = PotentializedUnionFind::<AdditiveOperation<_>>::new(n);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Unite { x, y, w } => {
                uf.unite_with(x, y, w);
            }
            Query::Diff { x, y } => {
                if let Some(w) = uf.difference(x, y) {
                    pp!(w);
                } else {
                    pp!("?");
                }
            }
        }
    }
}
