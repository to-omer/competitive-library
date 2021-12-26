#[codesnip::entry("levenshtein_distance")]
pub fn levenshtein_distance<T: PartialEq>(x: &[T], y: &[T]) -> usize {
    let n = x.len();
    let m = y.len();
    let mut dp = vec![vec![0; m + 1]; n + 1];
    for (i, dp) in dp.iter_mut().enumerate() {
        dp[0] = i;
    }
    for j in 1..=m {
        dp[0][j] = j;
    }
    for (i, x) in x.iter().enumerate() {
        for (j, y) in y.iter().enumerate() {
            dp[i + 1][j + 1] =
                (dp[i][j + 1].min(dp[i + 1][j]) + 1).min(dp[i][j] + (x != y) as usize);
        }
    }
    dp[n][m]
}
