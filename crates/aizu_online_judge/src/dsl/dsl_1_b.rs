use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, data_structure::PotentializedUnionFind};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Unite { x: usize, y: usize, w: i64 }
        1 => Diff { x: usize, y: usize }
    }
}

#[verify::aizu_online_judge("DSL_1_B")]
pub fn dsl_1_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = PotentializedUnionFind::<AdditiveOperation<_>>::new(n);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Unite { x, y, w } => {
                uf.unite_with(x, y, w);
            }
            Query::Diff { x, y } => {
                if let Some(w) = uf.difference(x, y) {
                    writeln!(writer, "{}", w).ok();
                } else {
                    writeln!(writer, "?").ok();
                }
            }
        }
    }
}
