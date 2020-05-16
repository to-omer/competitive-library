pub use crate::algebra::operations::LinearOperation;
pub use crate::data_structure::sliding_winsow_aggregation::QueueAggregation;
pub use crate::math::modu32::{modulos::Modulo998244353, Modu32};
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

type M = Modu32<Modulo998244353>;

#[verify_attr::verify("https://judge.yosupo.jp/problem/queue_operate_all_composite")]
pub fn queue_operate_all_composite(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let q: usize = scanner.scan();
    let mut que = QueueAggregation::new(LinearOperation::new());
    for _ in 0..q {
        let ty: usize = scanner.scan();
        match ty {
            0 => {
                let ab: (M, M) = scanner.scan();
                que.push(ab);
            }
            1 => {
                que.pop();
            }
            _ => {
                let x: M = scanner.scan();
                let (a, b) = que.fold_all();
                writeln!(writer, "{}", a * x + b)?;
            }
        }
    }
    Ok(())
}
