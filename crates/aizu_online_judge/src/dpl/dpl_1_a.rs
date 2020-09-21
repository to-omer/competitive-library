use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/1/DPL_1_A")]
pub fn dpl_1_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, c: [usize; m]);
    let mut dp = vec![std::usize::MAX; n + 1];
    dp[0] = 0;
    for c in c {
        for i in c..=n {
            dp[i] = (dp[i - c] + 1).min(dp[i]);
        }
    }
    writeln!(writer, "{}", dp[n]).ok();
}
