use competitive::prelude::*;

#[verify::aizu_online_judge("DSL_5_A")]
pub fn dsl_5_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, t, lr: [(usize, usize)]);
    let mut acc = vec![0; t + 1];
    for (l, r) in lr.take(n) {
        acc[l] += 1;
        acc[r] -= 1;
    }
    for i in 0..t {
        acc[i + 1] += acc[i];
    }
    pp!(acc.into_iter().max().unwrap_or_default());
}
