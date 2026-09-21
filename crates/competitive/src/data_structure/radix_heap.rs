macro_rules! define_radix_heap {
    ($name:ident, $key:ty, $buckets:expr) => {
        /// A min-priority queue whose removed keys are monotonically nondecreasing.
        ///
        /// Values with equal keys have no specified removal order.
        #[derive(Clone, Debug)]
        pub struct $name<T> {
            buckets: [Vec<($key, T)>; $buckets],
            last: $key,
            len: usize,
        }

        impl<T> $name<T> {
            pub fn new() -> Self {
                Self {
                    buckets: std::array::from_fn(|_| Vec::new()),
                    last: 0,
                    len: 0,
                }
            }

            pub fn len(&self) -> usize {
                self.len
            }

            pub fn is_empty(&self) -> bool {
                self.len == 0
            }

            /// Inserts a value whose key is not less than the key most recently removed.
            ///
            /// # Panics
            ///
            /// Panics if `key` is less than the key most recently removed.
            pub fn push(&mut self, key: $key, value: T) {
                assert!(key >= self.last, "key is less than the last removed key");
                self.buckets[Self::bucket_index(key, self.last)].push((key, value));
                self.len += 1;
            }

            pub fn pop(&mut self) -> Option<($key, T)> {
                if self.len == 0 {
                    return None;
                }
                if self.buckets[0].is_empty() {
                    let index = (1..self.buckets.len())
                        .find(|&index| !self.buckets[index].is_empty())
                        .unwrap();
                    self.last = self.buckets[index]
                        .iter()
                        .map(|&(key, _)| key)
                        .min()
                        .unwrap();
                    let mut values = std::mem::take(&mut self.buckets[index]);
                    while let Some((key, value)) = values.pop() {
                        let next = Self::bucket_index(key, self.last);
                        debug_assert!(next < index);
                        self.buckets[next].push((key, value));
                    }
                    self.buckets[index] = values;
                }
                self.len -= 1;
                self.buckets[0].pop()
            }

            pub fn clear(&mut self) {
                for bucket in &mut self.buckets {
                    bucket.clear();
                }
                self.last = 0;
                self.len = 0;
            }

            #[inline]
            fn bucket_index(key: $key, last: $key) -> usize {
                (<$key>::BITS - (key ^ last).leading_zeros()) as usize
            }
        }

        impl<T> Default for $name<T> {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

define_radix_heap!(RadixHeapU32, u32, 33);
define_radix_heap!(RadixHeapU64, u64, 65);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use crate::tools::testutil::{exhaustive_sequences, integer_boundary_values};
    use std::{cmp::Reverse, collections::BinaryHeap};

    #[test]
    fn test_radix_heap() {
        macro_rules! check {
            ($heap:ident, $key:ty) => {{
                let mut rng = Xorshift::default();
                let mut cases: Vec<_> = exhaustive_sequences(0..3, 0..=6).collect();
                let boundaries = integer_boundary_values!($key);
                cases.push(boundaries.iter().chain(&boundaries).copied().collect());
                for values in cases {
                    let mut actual = $heap::new();
                    let mut expected: Vec<_> = values
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(i, key)| (key, i))
                        .collect();
                    rng.shuffle(&mut expected);
                    for &(key, value) in &expected {
                        actual.push(key, value);
                    }
                    let mut cleared = actual.clone();
                    cleared.pop();
                    cleared.clear();
                    assert_eq!(cleared.len(), 0);
                    assert!(cleared.is_empty());
                    assert_eq!(cleared.pop(), None);
                    let pair = (<$key>::MIN, values.len());
                    cleared.push(pair.0, pair.1);
                    assert_eq!(cleared.pop(), Some(pair));
                    assert_eq!(cleared.pop(), None);
                    for &(key, value) in &expected {
                        cleared.push(key, value);
                    }
                    expected.sort();
                    for mut heap in [actual, cleared] {
                        let mut result = Vec::new();
                        while let Some(pair) = heap.pop() {
                            result.push(pair);
                        }
                        assert!(result.windows(2).all(|w| w[0].0 <= w[1].0));
                        result.sort();
                        assert_eq!(result, expected);
                    }
                }
                let mut actual = $heap::new();
                let mut expected = BinaryHeap::new();
                let mut rng = Xorshift::default();
                for value in 0..4096 {
                    let key = rng.rand(1_000_000) as $key;
                    actual.push(key, value);
                    expected.push(Reverse((key, value)));
                }
                for value in 4096..104_096 {
                    let (key, _) = actual.pop().unwrap();
                    let Reverse((expected_key, _)) = expected.pop().unwrap();
                    assert_eq!(key, expected_key);
                    let key = key.saturating_add(rng.rand(1_000_000) as $key);
                    actual.push(key, value);
                    expected.push(Reverse((key, value)));
                }
                while let Some((key, _)) = actual.pop() {
                    let Reverse((expected_key, _)) = expected.pop().unwrap();
                    assert_eq!(key, expected_key);
                }
                assert!(expected.is_empty());
                assert!(actual.is_empty());
            }};
        }

        check!(RadixHeapU32, u32);
        check!(RadixHeapU64, u64);
    }
}
