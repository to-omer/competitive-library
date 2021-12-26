use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, data_structure::QueueAggregation};

#[cfg_attr(nightly, verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/3/DSL_3_A"))]
pub fn dsl_3_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, s: u64, a: [u64]);
    let mut que = QueueAggregation::<AdditiveOperation<_>>::new();
    let mut ans = std::usize::MAX;
    for a in a.take(n) {
        que.push(a);
        while que.fold_all() >= s {
            ans = ans.min(que.len());
            que.pop();
        }
    }
    writeln!(writer, "{}", if ans == std::usize::MAX { 0 } else { ans }).ok();
}
