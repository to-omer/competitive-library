pub use crate::algebra::MinOperation;
pub use crate::data_structure::QueueAggregation;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/3/DSL_3_D")]
pub fn dsl_3_d(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, l, a: [u64; n]);
    let mut que = QueueAggregation::new(MinOperation::new());
    for (i, a) in a.into_iter().enumerate() {
        que.push(a);
        if que.len() == l {
            write!(
                writer,
                "{}{}",
                que.fold_all(),
                if i == n - 1 { "" } else { " " }
            )
            .ok();
            que.pop();
        }
    }
    writeln!(writer).ok();
}
