use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/3/DSL_3_B")]
pub fn dsl_3_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, k, a: [Usize1; n]);
    let mut counter = vec![0; 100_001];
    let mut j = 0;
    let mut ans = std::usize::MAX;
    let mut cnt = 0;
    for i in 0..n {
        while j < n && cnt < k {
            cnt += (a[j] < k && counter[a[j]] == 0) as usize;
            counter[a[j]] += 1;
            j += 1;
        }
        if cnt == k {
            ans = ans.min(j - i);
        }
        counter[a[i]] -= 1;
        cnt -= (a[i] < k && counter[a[i]] == 0) as usize;
    }
    writeln!(writer, "{}", if ans == std::usize::MAX { 0 } else { ans }).ok();
}
