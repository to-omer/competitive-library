use competitive::prelude::*;

#[verify::aizu_online_judge("DSL_3_C")]
pub fn dsl_3_c(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u64; n], x: [u64]);
    for x in x.take(q) {
        let mut ans = 0;
        let mut sum = 0;
        let mut l = 0;
        for (r, &b) in a.iter().enumerate() {
            sum += b;
            while sum > x {
                sum -= a[l];
                l += 1;
            }
            ans += r + 1 - l;
        }
        pp!(ans);
    }
}
