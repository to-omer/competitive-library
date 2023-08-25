use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_A")]
pub fn dpl_1_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, c: [usize; m]);
    let mut dp = vec![usize::MAX; n + 1];
    dp[0] = 0;
    for c in c {
        for i in c..=n {
            dp[i] = (dp[i - c] + 1).min(dp[i]);
        }
    }
    writeln!(writer, "{}", dp[n]).ok();
}
