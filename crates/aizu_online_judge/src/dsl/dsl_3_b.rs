use competitive::prelude::*;

#[verify::aizu_online_judge("DSL_3_B")]
pub fn dsl_3_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, k, a: [Usize1; n]);
    let mut counter = vec![0; 100_001];
    let mut j = 0;
    let mut ans = usize::MAX;
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
    pp!(if ans == usize::MAX { 0 } else { ans });
}
