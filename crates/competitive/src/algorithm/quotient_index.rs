use std::ops::RangeInclusive;

/// sorted({ floor(n/k) | k in \[1, n\] })
pub struct QuotientIndex {
    num: usize,
    sqrt: usize,
    pivot: usize,
}

impl QuotientIndex {
    pub fn new(num: usize) -> Self {
        assert!(num > 0, "num must be positive");
        let sqrt = num.isqrt();
        let pivot = num / sqrt;
        Self { num, sqrt, pivot }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        2 * self.sqrt - (self.pivot == self.sqrt) as usize
    }

    pub fn contains_key(&self, key: usize) -> bool {
        (1..=self.num).contains(&key)
    }

    pub fn contains_value(&self, value: usize) -> bool {
        if value == 0 || value > self.num {
            return false;
        }
        if value == usize::MAX {
            return self.num == usize::MAX;
        }
        self.num / value > self.num / (value + 1)
    }

    pub fn index_to_key(&self, index: usize) -> RangeInclusive<usize> {
        assert!(index < self.len(), "index out of bounds");
        unsafe { self.index_to_key_unchecked(index) }
    }

    pub fn index_to_value(&self, index: usize) -> usize {
        assert!(index < self.len(), "index out of bounds");
        unsafe { self.index_to_value_unchecked(index) }
    }

    pub fn value_to_index(&self, value: usize) -> usize {
        assert!(self.contains_value(value), "value is not present");
        unsafe { self.value_to_index_unchecked(value) }
    }

    pub fn value_to_key(&self, value: usize) -> RangeInclusive<usize> {
        assert!(self.contains_value(value), "value is not present");
        unsafe { self.value_to_key_unchecked(value) }
    }

    pub fn key_to_index(&self, key: usize) -> usize {
        assert!(self.contains_key(key), "key out of bounds");
        let value = self.key_to_value(key);
        unsafe { self.value_to_index_unchecked(value) }
    }

    pub fn key_to_value(&self, key: usize) -> usize {
        assert!(self.contains_key(key), "key out of bounds");
        unsafe { self.key_to_value_unchecked(key) }
    }

    /// # Safety
    /// `index` must satisfy `index < self.len()`.
    pub unsafe fn index_to_key_unchecked(&self, index: usize) -> RangeInclusive<usize> {
        unsafe {
            let value = self.index_to_value_unchecked(index);
            self.value_to_key_unchecked(value)
        }
    }

    /// # Safety
    /// `index` must satisfy `index < self.len()`.
    pub unsafe fn index_to_value_unchecked(&self, index: usize) -> usize {
        if index < self.sqrt {
            index + 1
        } else {
            self.num / (self.len() - index)
        }
    }

    /// # Safety
    /// `value` must satisfy `self.contains_value(value)`.
    pub unsafe fn value_to_index_unchecked(&self, value: usize) -> usize {
        if value <= self.sqrt {
            value - 1
        } else {
            self.len() - self.num / value
        }
    }

    /// # Safety
    /// `value` must satisfy `self.contains_value(value)`.
    pub unsafe fn value_to_key_unchecked(&self, value: usize) -> RangeInclusive<usize> {
        if value == usize::MAX {
            return 1..=1;
        }
        let start = (self.num / (value + 1)) + 1;
        let end = self.num / value;
        start..=end
    }

    /// # Safety
    /// `key` must satisfy `1 <= key && key <= self.num`.
    pub unsafe fn key_to_index_unchecked(&self, key: usize) -> usize {
        unsafe {
            let value = self.key_to_value_unchecked(key);
            self.value_to_index_unchecked(value)
        }
    }

    /// # Safety
    /// `key` must satisfy `1 <= key && key <= self.num`.
    pub unsafe fn key_to_value_unchecked(&self, key: usize) -> usize {
        self.num / key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_quotient_index() {
        let mut rng = Xorshift::default();
        for n in (1..=2000).chain(rng.random_iter(1..=200_000).take(100)) {
            let qi = QuotientIndex::new(n);
            let mut a: Vec<_> = (1..=n).rev().map(|key| n / key).collect();
            a.dedup();
            assert_eq!(qi.len(), a.len());
            let mut count = 0;
            let mut is_value = vec![false; n + 1];
            for (index, &value) in a.iter().enumerate() {
                assert_eq!(qi.index_to_value(index), value);
                assert_eq!(qi.value_to_index(value), index);
                assert_eq!(qi.index_to_key(index), qi.value_to_key(value));
                for key in qi.index_to_key(index) {
                    assert_eq!(qi.key_to_value(key), value);
                    assert_eq!(qi.key_to_index(key), index);
                    count += 1;
                }
                is_value[value] = true;
            }
            assert_eq!(count, n);
            for (value, &present) in is_value.iter().enumerate() {
                assert_eq!(qi.contains_value(value), present);
            }
            assert!(!qi.contains_key(!0));
        }

        for n in [usize::MAX]
            .into_iter()
            .chain(rng.random_iter(1..=!0).take(100))
        {
            let qi = QuotientIndex::new(n);
            let len = qi.len();
            for index in (0..len.min(1000)).chain(len.saturating_sub(1000)..len) {
                let value = qi.index_to_value(index);
                assert_eq!(qi.index_to_value(index), value);
                assert_eq!(qi.value_to_index(value), index);
                assert_eq!(qi.index_to_key(index), qi.value_to_key(value));
                for key in qi
                    .index_to_key(index)
                    .take(1)
                    .chain(qi.index_to_key(index).rev().take(1))
                {
                    assert_eq!(qi.key_to_value(key), value);
                    assert_eq!(qi.key_to_index(key), index);
                }
            }
        }
    }
}
