use competitive::math::floor_sum;
use competitive::prelude::*;

#[verify::library_checker("sum_of_floor_of_linear")]
pub fn sum_of_floor_of_linear(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t, query: [(u64, u64, u64, u64)]);
    for (n, m, a, b) in query.take(t) {
        pp!(floor_sum(n, a, b, m));
    }
}
