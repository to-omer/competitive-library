use super::BitSet;
use std::{cmp::Reverse, collections::BinaryHeap, mem::take};

#[derive(Debug, Clone)]
pub struct SubsetSumProblem {
    size: usize,
    dp: BitSet,
    pending_weights: Vec<Reverse<usize>>,
}

impl SubsetSumProblem {
    pub fn new(size: usize) -> Self {
        let mut dp = BitSet::new(if size == !0 { 0 } else { size } + 1);
        dp.set(0, true);
        Self {
            size,
            dp,
            pending_weights: vec![],
        }
    }

    pub fn insert(&mut self, weight: usize) {
        if weight == 0 || self.size < weight {
            return;
        }
        self.pending_weights.push(Reverse(weight));
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = usize>,
    {
        for weight in iter {
            self.insert(weight);
        }
    }

    pub fn contains(&mut self, sum: usize) -> bool {
        self.rebuild();
        if sum < self.dp.len() {
            self.dp.get(sum)
        } else {
            false
        }
    }

    fn rebuild(&mut self) {
        if self.pending_weights.is_empty() {
            return;
        }
        let mut heap = BinaryHeap::from(take(&mut self.pending_weights));
        let (mut current_weight, mut count) = match heap.pop() {
            Some(Reverse(w)) => (w, 1),
            None => return,
        };
        while let Some(Reverse(weight)) = heap.pop() {
            if weight == current_weight {
                count += 1;
                if count >= 3 {
                    if let Some(w) = current_weight.checked_mul(2) {
                        heap.push(Reverse(w));
                    }
                    count -= 2;
                }
                continue;
            }
            for _ in 0..count {
                if self.size == !0 {
                    self.dp.resize(self.dp.len() + current_weight);
                }
                self.dp.shl_bitor_assign(current_weight);
            }
            (current_weight, count) = (weight, 1);
        }
        for _ in 0..count {
            if self.size == !0 {
                self.dp.resize(self.dp.len() + current_weight);
            }
            self.dp.shl_bitor_assign(current_weight);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    fn naive(weights: &[usize], limit: usize) -> Vec<bool> {
        let mut dp = vec![false; limit + 1];
        dp[0] = true;
        for &w in weights {
            for s in (w..=limit).rev() {
                if dp[s - w] {
                    dp[s] = true;
                }
            }
        }
        dp
    }

    #[test]
    fn test_subset_sum_problem() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            rand!(rng, n: 0..=10usize, limit: 0..=400usize, max_weight: 1..=100usize);
            let mut ssp = SubsetSumProblem::new(limit);
            let mut weights = vec![];
            for _ in 0..n {
                rand!(rng, w: 0..=max_weight, c: 0..=10);
                for _ in 0..c {
                    weights.push(w);
                    ssp.insert(w);
                }
            }
            let sum: usize = weights.iter().sum();
            let expected = naive(&weights, sum + 2);
            for (s, &expected) in expected.iter().enumerate() {
                let expected = if s <= limit { expected } else { false };
                assert_eq!(ssp.contains(s), expected);
            }
            let mut ssp = SubsetSumProblem::new(!0);
            ssp.extend(weights.iter().cloned());
            for (s, &expected) in expected.iter().enumerate() {
                assert_eq!(ssp.contains(s), expected);
            }
        }
    }
}
