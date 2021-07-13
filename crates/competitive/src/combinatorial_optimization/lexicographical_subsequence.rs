#[codesnip::entry("LexicographicalSubsequence")]
#[derive(Debug, Clone)]
pub struct LexicographicalSubsequence {
    dp: Vec<usize>,
    index: Vec<Vec<usize>>,
}
#[codesnip::entry("LexicographicalSubsequence")]
impl LexicographicalSubsequence {
    pub fn new(sequence: &[usize]) -> Self {
        let n = sequence.len();
        let w = sequence.iter().max().map(|w| w + 1).unwrap_or_default();
        let mut dp = vec![0usize; n + 2];
        dp[n] = 1;
        let mut next = vec![n; w];
        let mut index = vec![vec![]; w];
        for (i, c) in sequence.iter().cloned().enumerate().rev() {
            next[c] = i;
            index[c].push(i);
            dp[i] = next.iter().fold(1, |acc, &j| acc.saturating_add(dp[j + 1]));
        }
        Self { dp, index }
    }
    /// empty sequence is included
    pub fn kth_sequence(&self, mut k: usize) -> Option<Vec<usize>> {
        if self.dp[0] <= k {
            return None;
        }
        let mut seq = Vec::new();
        let mut pos: Vec<_> = self.index.iter().map(Vec::len).collect();
        let mut cur = 0usize;
        while k > 0 {
            k -= 1;
            for (c, (idx, pos)) in self.index.iter().zip(pos.iter_mut()).enumerate() {
                while *pos > 0 && idx[*pos - 1] < cur {
                    *pos -= 1;
                }
                if *pos == 0 {
                    continue;
                }
                let x = idx[*pos - 1];
                if self.dp[x + 1] <= k {
                    k -= self.dp[x + 1];
                } else {
                    cur = x + 1;
                    seq.push(c);
                    break;
                }
            }
        }
        Some(seq)
    }
}
