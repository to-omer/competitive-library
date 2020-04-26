// verify-helper: PROBLEM https://judge.yosupo.jp/problem/queue_operate_all_composite

use competitive_library::algebra::operations::LinearOperation;
use competitive_library::data_structure::sliding_winsow_aggregation::QueueAggregation;
use competitive_library::math::modu32::{modulos::Modulo998244353, Modu32};
use competitive_library::{input, input_inner};
use std::io::Write;

type M = Modu32<Modulo998244353>;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, q };
    let mut que = QueueAggregation::new(LinearOperation::new());
    for _ in 0..q {
        input_inner! { iter, ty };
        match ty {
            0 => {
                input_inner! { iter, ab: (M, M) };
                que.push(ab);
            }
            1 => {
                que.pop();
            }
            _ => {
                input_inner! { iter, x: M };
                let (a, b) = que.fold_all();
                writeln!(out, "{}", a * x + b)?;
            }
        }
    }

    Ok(())
}
