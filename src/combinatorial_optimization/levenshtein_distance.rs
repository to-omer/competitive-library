#[cargo_snippet::snippet("levenshtein_distance")]
pub fn levenshtein_distance<T: PartialEq>(x: &[T], y: &[T]) -> usize {
    let n = x.len();
    let m = y.len();
    let mut dp = vec![vec![0; m + 1]; n + 1];
    for i in 1..=n {
        dp[i][0] = i;
    }
    for j in 1..=m {
        dp[0][j] = j;
    }
    for i in 0..n {
        for j in 0..m {
            dp[i + 1][j + 1] =
                (dp[i][j + 1].min(dp[i + 1][j]) + 1).min(dp[i][j] + (x[i] != y[j]) as usize);
        }
    }
    dp[n][m]
}
