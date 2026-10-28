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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::testutil::exhaustive_sequences;

    #[test]
    fn test_longest_increasing_subsequence() {
        for values in exhaustive_sequences(0..=4, 0..=5) {
            let mut sequence = LongestIncreasingSubsequence::new();
            let mut ending = vec![0; values.len()];
            let mut longest = 0;
            assert_eq!(sequence.longest_length(), 0);
            for (i, &value) in values.iter().enumerate() {
                ending[i] = 1;
                for j in 0..i {
                    if values[j] < value {
                        ending[i] = ending[i].max(ending[j] + 1);
                    }
                }
                longest = longest.max(ending[i]);
                sequence.insert(value);
                assert_eq!(sequence.longest_length(), longest);
                assert!(sequence.dp.windows(2).all(|pair| pair[0] < pair[1]));
            }
        }
    }
}
