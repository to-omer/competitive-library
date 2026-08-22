use super::{AbelianGroup, BinaryIndexedTree, RadixSortKey, SliceSortExt};
use std::ops::Range;

/// Offline range folds over entries whose keys are at most a query bound.
pub struct RangeFoldWithUpperBound<K, M>
where
    M: AbelianGroup,
{
    keys: Vec<K>,
    weights: Vec<M::T>,
    queries: Vec<(Range<usize>, K)>,
}

impl<K, M> RangeFoldWithUpperBound<K, M>
where
    K: RadixSortKey + Ord,
    M: AbelianGroup,
{
    pub fn new(values: impl IntoIterator<Item = (K, M::T)>) -> Self {
        let (keys, weights) = values.into_iter().unzip();
        Self {
            keys,
            weights,
            queries: Vec::new(),
        }
    }

    pub fn query(&mut self, range: Range<usize>, upper_bound: K) -> usize {
        let index = self.queries.len();
        self.queries.push((range, upper_bound));
        index
    }

    pub fn execute(self) -> Vec<M::T> {
        let mut values: Vec<_> = self
            .keys
            .iter()
            .copied()
            .enumerate()
            .map(|(index, key)| (key, index))
            .collect();
        values.radix_sort_by_key(|&(key, _)| key);
        let mut order: Vec<_> = self
            .queries
            .iter()
            .enumerate()
            .map(|(index, (_, upper_bound))| (*upper_bound, index))
            .collect();
        order.radix_sort_by_key(|&(upper_bound, _)| upper_bound);

        let mut bit: BinaryIndexedTree<M> = BinaryIndexedTree::new(values.len());
        let mut answers = vec![M::unit(); self.queries.len()];
        let mut inserted = 0;
        for (upper_bound, index) in order {
            while inserted < values.len() && values[inserted].0 <= upper_bound {
                let position = values[inserted].1;
                bit.update(position, self.weights[position].clone());
                inserted += 1;
            }
            let range = &self.queries[index].0;
            answers[index] = bit.fold(range.start, range.end);
        }
        answers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algebra::AdditiveOperation, rand, tools::Xorshift};

    #[test]
    fn test_range_fold_with_upper_bound() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..200, keys: [-10..10; n], weights: [-100..100; n]);
            let mut fold: RangeFoldWithUpperBound<_, AdditiveOperation<i64>> =
                RangeFoldWithUpperBound::new(keys.iter().copied().zip(weights.iter().copied()));
            let mut expected = Vec::new();
            for _ in 0..100 {
                rand!(rng, l: 0..=n, r: l..=n, upper_bound: -10..=10);
                expected.push(
                    keys[l..r]
                        .iter()
                        .zip(&weights[l..r])
                        .filter(|&(&key, _)| key <= upper_bound)
                        .map(|(_, &weight)| weight)
                        .sum(),
                );
                fold.query(l..r, upper_bound);
            }
            assert_eq!(fold.execute(), expected);
        }
    }
}
