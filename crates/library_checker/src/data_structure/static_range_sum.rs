use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, data_structure::Accumulate};

#[verify::library_checker("static_range_sum")]
pub fn static_range_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; iter n]);
    let acc: Accumulate<AdditiveOperation<i64>> = a.collect();
    sc!(lr: [(usize, usize); iter q]);
    for (l, r) in lr {
        pp!(acc.fold(l..r));
    }
}
