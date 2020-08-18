pub use crate::data_structure::Static2DTree;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_C")]
pub fn dsl_2_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, xy: [(i64, i64); n], q);
    let tree = Static2DTree::new(xy.into_iter().enumerate().map(|(i, (x, y))| (x, y, i)));
    for (sx, tx, sy, ty) in scanner.iter::<(i64, i64, i64, i64)>().take(q) {
        let mut v = tree.range(sx..tx + 1, sy..ty + 1);
        v.sort();
        for v in v {
            writeln!(writer, "{}", v).ok();
        }
        writeln!(writer).ok();
    }
}
