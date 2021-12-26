use competitive::prelude::*;

#[cfg_attr(nightly, verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/3/DSL_3_C"))]
pub fn dsl_3_c(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], x: [u64]);
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
        writeln!(writer, "{}", ans).ok();
    }
}
