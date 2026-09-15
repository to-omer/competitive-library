use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation, data_structure::PotentializedUnionFind,
    num::montgomery::MInt998244353,
};

competitive::define_enum_scan! {
    enum Query: u8 {
        0 => Unite { u: usize, v: usize, x: MInt998244353 }
        1 => Diff { u: usize, v: usize }
    }
}

#[verify::library_checker("unionfind_with_potential")]
pub fn unionfind_with_potential(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut uf = PotentializedUnionFind::<AdditiveOperation<MInt998244353>>::new(n);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Unite { u, v, x } => {
                if let Some(diff) = uf.difference(u, v) {
                    pp!((diff == x) as u8);
                } else {
                    uf.unite_with(u, v, x);
                    pp!("1");
                }
            }
            Query::Diff { u, v } => {
                if let Some(diff) = uf.difference(u, v) {
                    pp!(diff);
                } else {
                    pp!("-1");
                }
            }
        }
    }
}
