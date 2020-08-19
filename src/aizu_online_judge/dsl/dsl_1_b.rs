pub use crate::algebra::AdditiveOperation;
pub use crate::data_structure::WeightedUnionFind;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/1/DSL_1_B")]
pub fn dsl_1_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = WeightedUnionFind::new(n, AdditiveOperation::new());
    for _ in 0..q {
        scan!(scanner, ty, x, y);
        if ty == 0 {
            scan!(scanner, w: i64);
            uf.unite(x, y, w);
        } else {
            if let Some(w) = uf.get_difference(x, y) {
                writeln!(writer, "{}", w).ok();
            } else {
                writeln!(writer, "?").ok();
            }
        }
    }
}
