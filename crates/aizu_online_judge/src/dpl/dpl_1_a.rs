use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_A")]
pub fn dpl_1_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, c: [usize; iter m]);
    let mut dp = vec![usize::MAX; n + 1];
    dp[0] = 0;
    for c in c {
        for i in c..=n {
            dp[i] = (dp[i - c] + 1).min(dp[i]);
        }
    }
    pp!(dp[n]);
}
