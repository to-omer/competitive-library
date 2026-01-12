use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
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
pub fn unionfind_with_potential(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = PotentializedUnionFind::<AdditiveOperation<MInt998244353>>::new(n);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Unite { u, v, x } => {
                if let Some(diff) = uf.difference(u, v) {
                    writeln!(writer, "{}", (diff == x) as u8).ok();
                } else {
                    uf.unite_with(u, v, x);
                    writeln!(writer, "1").ok();
                }
            }
            Query::Diff { u, v } => {
                if let Some(diff) = uf.difference(u, v) {
                    writeln!(writer, "{}", diff).ok();
                } else {
                    writeln!(writer, "-1").ok();
                }
            }
        }
    }
}
