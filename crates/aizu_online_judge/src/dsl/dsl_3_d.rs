use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::MinOperation, data_structure::QueueAggregation};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/3/DSL_3_D")]
pub fn dsl_3_d(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, l, a: [u64]);
    let mut que = QueueAggregation::<MinOperation<_>>::new();
    let mut ans = Vec::with_capacity(n - l + 1);
    for a in a.take(n) {
        que.push(a);
        if que.len() == l {
            ans.push(que.fold_all());
            que.pop();
        }
    }
    iter_print!(writer, @it ans);
}
