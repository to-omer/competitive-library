#[doc(no_inline)]
pub use competitive::data_structure::Static2DTree;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_C")]
pub fn dsl_2_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, xy: [(i64, i64)]);
    let tree = Static2DTree::new(xy.take(n).enumerate().map(|(i, (x, y))| (x, y, i)));
    scan!(scanner, q, query: [(i64, i64, i64, i64)]);
    for (sx, tx, sy, ty) in query.take(q) {
        let mut v = tree.range(sx..tx + 1, sy..ty + 1);
        v.sort();
        for v in v {
            writeln!(writer, "{}", v).ok();
        }
        writeln!(writer).ok();
    }
}
