use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/3/DSL_3_C")]
pub fn dsl_3_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], x: [u64; q]);
    for x in x {
        let mut ans = 0;
        let mut sum = 0;
        let mut p = 0;
        for (i, &b) in a.iter().enumerate() {
            sum += b;
            while sum > x {
                sum -= a[p];
                p += 1;
            }
            ans += i - p + 1;
        }
        writeln!(writer, "{}", ans).ok();
    }
}
