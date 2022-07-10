use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, data_structure::WeightedUnionFind};

#[verify::aizu_online_judge("DSL_1_B")]
pub fn dsl_1_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = WeightedUnionFind::<AdditiveOperation<_>>::new(n);
    for _ in 0..q {
        scan!(scanner, ty, x, y);
        if ty == 0 {
            scan!(scanner, w: i64);
            uf.unite(x, y, w);
        } else if let Some(w) = uf.get_difference(x, y) {
            writeln!(writer, "{}", w).ok();
        } else {
            writeln!(writer, "?").ok();
        }
    }
}
