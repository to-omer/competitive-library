use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, data_structure::Accumulate};

#[verify::library_checker("static_range_sum")]
pub fn static_range_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; n], lr: [(usize, usize)]);
    let acc: Accumulate<AdditiveOperation<i64>> = a.into_iter().collect();
    for (l, r) in lr.take(q) {
        pp!(acc.fold(l..r));
    }
}
