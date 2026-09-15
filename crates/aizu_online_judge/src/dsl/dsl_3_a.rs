use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, data_structure::QueueAggregation};

#[verify::aizu_online_judge("DSL_3_A")]
pub fn dsl_3_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, s: u64, a: [u64]);
    let mut que = QueueAggregation::<AdditiveOperation<_>>::new();
    let mut ans = usize::MAX;
    for a in a.take(n) {
        que.push(a);
        while que.fold_all() >= s {
            ans = ans.min(que.len());
            que.pop();
        }
    }
    pp!(if ans == usize::MAX { 0 } else { ans });
}
