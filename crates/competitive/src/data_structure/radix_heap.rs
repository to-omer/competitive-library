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
    use std::{cmp::Reverse, collections::BinaryHeap};

    #[test]
    fn test_radix_heap() {
        let mut edges = RadixHeapU64::new();
        for (value, key) in [0, 0, 1, 63, 64, u32::MAX as u64, u64::MAX]
            .into_iter()
            .enumerate()
        {
            edges.push(key, value);
        }
        let mut keys = Vec::new();
        while let Some((key, _)) = edges.pop() {
            keys.push(key);
        }
        assert_eq!(keys, [0, 0, 1, 63, 64, u32::MAX as u64, u64::MAX]);

        macro_rules! check {
            ($heap:ident, $key:ty) => {{
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
                actual.clear();
                actual.push(0, 0);
                assert_eq!(actual.pop(), Some((0, 0)));
            }};
        }

        check!(RadixHeapU32, u32);
        check!(RadixHeapU64, u64);
    }
}
