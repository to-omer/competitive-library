pub use crate::algebra::LinearOperation;
pub use crate::data_structure::QueueAggregation;
pub use crate::num::{modulus::MInt998244353, MInt};
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/queue_operate_all_composite")]
pub fn queue_operate_all_composite(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q);
    let mut que = QueueAggregation::new(LinearOperation::new());
    for _ in 0..q {
        scan!(scanner, ty);
        match ty {
            0 => {
                scan!(scanner, ab: (MInt998244353, MInt998244353));
                que.push(ab);
            }
            1 => {
                que.pop();
            }
            _ => {
                scan!(scanner, x: MInt998244353);
                let (a, b) = que.fold_all();
                writeln!(writer, "{}", a * x + b).ok();
            }
        }
    }
}
