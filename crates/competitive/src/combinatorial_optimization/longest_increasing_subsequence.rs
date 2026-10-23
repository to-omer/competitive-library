#[derive(Debug, Clone)]
pub struct LongestIncreasingSubsequence<T> {
    pub dp: Vec<T>,
}

impl<T> Default for LongestIncreasingSubsequence<T> {
    fn default() -> Self {
        Self { dp: Vec::new() }
    }
}

impl<T> LongestIncreasingSubsequence<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn longest_length(&self) -> usize {
        self.dp.len()
    }
}

impl<T> LongestIncreasingSubsequence<T>
where
    T: Ord,
{
    pub fn insert(&mut self, x: T) {
        let i = self.dp.binary_search(&x).unwrap_or_else(|x| x);
        if let Some(t) = self.dp.get_mut(i) {
            *t = x;
        } else {
            self.dp.push(x);
        }
    }
}

impl<T> Extend<T> for LongestIncreasingSubsequence<T>
where
    T: Ord,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for x in iter.into_iter() {
            self.insert(x);
        }
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_strictly_increasing_subsequence() {
        let values: [u8; 5] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= values.len());
        let mut sequence = LongestIncreasingSubsequence::new();
        let mut ending = [0; 5];
        let mut longest = 0;
        kani::assert(
            sequence.longest_length() == 0,
            "The empty input has longest strictly increasing subsequence length zero",
        );
        for i in 0..len {
            ending[i] = 1;
            for j in 0..i {
                if values[j] < values[i] {
                    ending[i] = ending[i].max(ending[j] + 1);
                }
            }
            longest = longest.max(ending[i]);
            sequence.insert(values[i]);
        }
        kani::assert(
            sequence.longest_length() == longest,
            "For any sequence of zero to five arbitrary u8 values, the reported length equals independent quadratic subsequence DP; equal values cannot extend a subsequence",
        );
        for j in 1..sequence.dp.len() {
            kani::assert(
                sequence.dp[j - 1] < sequence.dp[j],
                "The stored minimum tails are strictly increasing for every input of up to five u8 values",
            );
        }
        kani::cover!(len == 0);
        kani::cover!(len == 5 && longest == 5);
        kani::cover!(len == 5 && values[0] == u8::MAX && longest == 1);
    }
}
