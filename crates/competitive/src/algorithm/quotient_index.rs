use std::{
    iter::{ExactSizeIterator, FusedIterator},
    ops::RangeInclusive,
};
#[cfg(verus_keep_ghost)]
use verification::*;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

/// sorted({ floor(n/k) | k in \[1, n\] })
#[cfg_attr(verus_keep_ghost, verus_verify)]
pub struct FloorQuotientIndex {
    num: usize,
    sqrt: usize,
    pivot: usize,
}

impl FloorQuotientIndex {
    pub fn new(num: usize) -> Self {
        assert!(num > 0, "num must be positive");
        let sqrt = num.isqrt();
        let pivot = num / sqrt;
        Self { num, sqrt, pivot }
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = usize> + FusedIterator + '_ {
        let len = self.len();
        (0..len).map(move |index| unsafe { self.index_to_value_unchecked(index) })
    }

    pub fn key_values(
        &self,
    ) -> impl ExactSizeIterator<Item = (RangeInclusive<usize>, usize)> + FusedIterator + '_ {
        let len = self.len();
        (0..len).map(move |index| unsafe {
            (
                self.index_to_key_unchecked(index),
                self.index_to_value_unchecked(index),
            )
        })
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        2 * self.sqrt - (self.pivot == self.sqrt) as usize
    }

    pub fn contains_key(&self, key: usize) -> bool {
        (1..=self.num).contains(&key)
    }

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self@ > 0,
        ensures result == floor_quotient(self@ as int, value as int),
    ))]
    pub fn contains_value(&self, value: usize) -> bool {
        #[cfg(verus_keep_ghost)]
        proof! {
            floor_membership(self.num, value);
            #[verifier::proof_note("For every positive usize numerator and every usize value, contains_value recognizes exactly the values n/k for keys 1 through n, including usize::MAX")]
            assert(floor_quotient(self.num as int, value as int) ==
                (value > 0 && value <= self.num && (value == usize::MAX
                    || (self.num as int) / (value as int) > (self.num as int) / (value + 1))));
        }
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
        if index < self.sqrt {
            self.num / (index + 2) + 1..=self.num / (index + 1)
        } else {
            let key = self.len() - index;
            key..=key
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
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self@ > 0, floor_quotient(self@ as int, value as int),
        ensures
            1 <= result@.start <= result@.end <= self@,
            forall|k: int| 1 <= k <= self@ ==>
                ((result@.start <= k <= result@.end) == (#[trigger] (self@ as int / k) == value)),
    ))]
    pub unsafe fn value_to_key_unchecked(&self, value: usize) -> RangeInclusive<usize> {
        #[cfg(verus_keep_ghost)]
        proof! { floor_membership(self.num, value); }
        if value == usize::MAX {
            #[cfg(verus_keep_ghost)]
            proof! {
                assert forall|k: int| 1 <= k <= self.num implies
                    ((k == 1) == (#[trigger] (self.num as int / k) == value)) by {
                    if k > 1 { vstd::arithmetic::div_mod::lemma_div_decreases(self.num as int, k); }
                }
                #[verifier::proof_note("The maximum usize quotient occurs only for numerator usize::MAX and key 1")]
                assert(self.num == usize::MAX && forall|k: int| 1 <= k <= self.num ==>
                    ((k == 1) == (#[trigger] (self.num as int / k) == value)));
            }
            return 1..=1;
        }
        let start = (self.num / (value + 1)) + 1;
        let end = self.num / value;
        #[cfg(verus_keep_ghost)]
        proof! {
            vstd::arithmetic::div_mod::lemma_div_nonincreasing(self.num as int, value as int);
            assert forall|k: int| 1 <= k <= self.num implies
                ((start <= k <= end) == (#[trigger] (self.num as int / k) == value)) by {
                quotient_interval(self.num as int, value as int, k);
            }
            #[verifier::proof_note("For every present floor quotient below usize::MAX, value_to_key_unchecked returns a nonempty interval containing exactly all keys producing that quotient")]
            assert(1 <= start <= end <= self.num && forall|k: int| 1 <= k <= self.num ==>
                ((start <= k <= end) == (#[trigger] (self.num as int / k) == value)));
        }
        start..=end
    }

    /// # Safety
    /// `key` must satisfy `1 <= key && key <= self.num`.
    pub unsafe fn key_to_index_unchecked(&self, key: usize) -> usize {
        let len = self.len();
        if key <= len - self.sqrt {
            len - key
        } else {
            self.num / key - 1
        }
    }

    /// # Safety
    /// `key` must satisfy `1 <= key && key <= self.num`.
    pub unsafe fn key_to_value_unchecked(&self, key: usize) -> usize {
        self.num / key
    }
}

/// sorted({ ceil(n/k) | k in \[1, n\] })
pub struct CeilQuotientIndex {
    num: usize,
    sqrt: usize,
    pivot: usize,
}

impl CeilQuotientIndex {
    pub fn new(num: usize) -> Self {
        assert!(num > 0, "num must be positive");
        let sqrt = num.isqrt();
        let pivot = num.div_ceil(sqrt);
        Self { num, sqrt, pivot }
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = usize> + FusedIterator + '_ {
        let len = self.len();
        (0..len).map(move |index| unsafe { self.index_to_value_unchecked(index) })
    }

    pub fn key_values(
        &self,
    ) -> impl ExactSizeIterator<Item = (RangeInclusive<usize>, usize)> + FusedIterator + '_ {
        let len = self.len();
        (0..len).map(move |index| unsafe {
            (
                self.index_to_key_unchecked(index),
                self.index_to_value_unchecked(index),
            )
        })
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.sqrt + self.pivot - 1
    }

    pub fn contains_key(&self, key: usize) -> bool {
        (1..=self.num).contains(&key)
    }

    pub fn contains_value(&self, value: usize) -> bool {
        if value == 0 || value > self.num {
            return false;
        }
        if value == 1 {
            return true;
        }
        let start = self.num.div_ceil(value);
        let end = (self.num - 1) / (value - 1);
        start <= end
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
        if index < self.pivot {
            if index == 0 {
                return self.num..=self.num;
            }
            let start = self.num.div_ceil(index + 1);
            let end = (self.num - 1) / index;
            start..=end
        } else {
            let key = self.len() - index;
            key..=key
        }
    }

    /// # Safety
    /// `index` must satisfy `index < self.len()`.
    pub unsafe fn index_to_value_unchecked(&self, index: usize) -> usize {
        if index < self.pivot {
            index + 1
        } else {
            self.num.div_ceil(self.len() - index)
        }
    }

    /// # Safety
    /// `value` must satisfy `self.contains_value(value)`.
    pub unsafe fn value_to_index_unchecked(&self, value: usize) -> usize {
        if value <= self.pivot {
            value - 1
        } else {
            self.len() - (self.num - 1) / (value - 1)
        }
    }

    /// # Safety
    /// `value` must satisfy `self.contains_value(value)`.
    pub unsafe fn value_to_key_unchecked(&self, value: usize) -> RangeInclusive<usize> {
        if value == 1 {
            return self.num..=self.num;
        }
        let start = self.num.div_ceil(value);
        let end = (self.num - 1) / (value - 1);
        start..=end
    }

    /// # Safety
    /// `key` must satisfy `1 <= key && key <= self.num`.
    pub unsafe fn key_to_index_unchecked(&self, key: usize) -> usize {
        if key < self.sqrt {
            self.len() - key
        } else {
            self.num.div_ceil(key) - 1
        }
    }

    /// # Safety
    /// `key` must satisfy `1 <= key && key <= self.num`.
    pub unsafe fn key_to_value_unchecked(&self, key: usize) -> usize {
        self.num.div_ceil(key)
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    impl View for FloorQuotientIndex {
        type V = usize;
        closed spec fn view(&self) -> usize { self.num }
    }
}

#[cfg(verus_keep_ghost)]
mod verification {
    use vstd::{arithmetic::div_mod::*, prelude::*};

    verus! {
        pub open spec fn floor_quotient(n: int, value: int) -> bool {
            exists|k: int| 1 <= k <= n && #[trigger] (n / k) == value
        }

        pub proof fn quotient_interval(n: int, v: int, k: int)
            requires n > 0, v > 0, k > 0,
            ensures (n / k == v) == (n / (v + 1) < k <= n / v),
        {
            lemma_fundamental_div_mod(n, k);
            lemma_mod_bound(n, k);
            lemma_fundamental_div_mod(n, v);
            lemma_mod_bound(n, v);
            lemma_fundamental_div_mod(n, v + 1);
            lemma_mod_bound(n, v + 1);
            #[verifier::proof_note("For positive integers n, v and k, n/k equals v exactly when n/(v+1) < k <= n/v")]
            assert((n / k == v) == (n / (v + 1) < k <= n / v)) by(nonlinear_arith)
                requires n > 0, v > 0, k > 0,
                    n == k * (n / k) + n % k, 0 <= n % k < k,
                    n == v * (n / v) + n % v, 0 <= n % v < v,
                    n == (v + 1) * (n / (v + 1)) + n % (v + 1), 0 <= n % (v + 1) < v + 1;
        }

        pub proof fn floor_membership(n: usize, value: usize)
            requires n > 0,
            ensures floor_quotient(n as int, value as int) ==
                (value > 0 && value <= n && (value == usize::MAX
                    || (n as int) / (value as int) > (n as int) / (value + 1))),
        {
            let v = value as int;
            if value == 0 || value > n {
                assert forall|k: int| 1 <= k <= n implies #[trigger] (n as int / k) != v by {
                    lemma_div_non_zero(n as int, k);
                    lemma_div_nonincreasing(n as int, k);
                }
            } else if value == usize::MAX {
                assert(n == usize::MAX);
                assert(n as int / 1 == v);
            } else {
                let k = n as int / v;
                if (n as int) / v > (n as int) / (v + 1) {
                    lemma_div_non_zero(n as int, v);
                    lemma_div_nonincreasing(n as int, v);
                    quotient_interval(n as int, v, k);
                    assert(n as int / k == v);
                } else {
                    assert forall|k: int| 1 <= k <= n implies #[trigger] (n as int / k) != v by {
                        quotient_interval(n as int, v, k);
                    }
                }
            }
            #[verifier::proof_note("A positive floor quotient is present exactly when its key interval contains an integer, with zero, out-of-range values and usize::MAX handled explicitly")]
            assert(floor_quotient(n as int, value as int) ==
                (value > 0 && value <= n && (value == usize::MAX
                    || (n as int) / (value as int) > (n as int) / (value + 1))));
        }
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
            let qi = FloorQuotientIndex::new(n);
            let mut expected_values: Vec<_> = (1..=n).rev().map(|key| n / key).collect();
            expected_values.dedup();
            assert_eq!(qi.len(), expected_values.len());
            let values: Vec<_> = qi.values().collect();
            assert_eq!(values, expected_values);
            let expected_key_values: Vec<_> = (0..qi.len())
                .map(|index| (qi.index_to_key(index), qi.index_to_value(index)))
                .collect();
            let key_values: Vec<_> = qi.key_values().collect();
            assert_eq!(key_values, expected_key_values);
            let mut count = 0;
            let mut is_value = vec![false; n + 1];
            for (index, &value) in values.iter().enumerate() {
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
            let qi = FloorQuotientIndex::new(n);
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

    #[test]
    fn test_ceil_quotient_index() {
        let mut rng = Xorshift::default();
        for n in (1..=2000).chain(rng.random_iter(1..=200_000).take(100)) {
            let qi = CeilQuotientIndex::new(n);
            let mut expected_values: Vec<_> = (1..=n).rev().map(|key| n.div_ceil(key)).collect();
            expected_values.dedup();
            assert_eq!(qi.len(), expected_values.len());
            let values: Vec<_> = qi.values().collect();
            assert_eq!(values, expected_values);
            let expected_key_values: Vec<_> = (0..qi.len())
                .map(|index| (qi.index_to_key(index), qi.index_to_value(index)))
                .collect();
            let key_values: Vec<_> = qi.key_values().collect();
            assert_eq!(key_values, expected_key_values);
            let mut count = 0;
            let mut is_value = vec![false; n + 1];
            for (index, &value) in values.iter().enumerate() {
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
            let qi = CeilQuotientIndex::new(n);
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
