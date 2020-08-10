pub use crate::algebra::AdditiveOperation;
pub use crate::data_structure::QueueAggregation;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/3/DSL_3_A")]
pub fn dsl_3_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, s: u64, a: [u64; n]);
    let mut que = QueueAggregation::new(AdditiveOperation::new());
    let mut ans = std::usize::MAX;
    for a in a {
        que.push(a);
        while que.fold_all() >= s {
            ans = ans.min(que.len());
            que.pop();
        }
    }
    writeln!(writer, "{}", if ans == std::usize::MAX { 0 } else { ans }).ok();
}
