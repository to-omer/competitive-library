use std::{
    iter::{ExactSizeIterator, FusedIterator},
    ops::RangeInclusive,
};
#[cfg(verus_keep_ghost)]
use verification::*;
#[cfg(verus_keep_ghost)]
use vstd::{prelude::*, std_specs::iter::IteratorSpec};

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

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(),
        ensures
            result.obeys_prophetic_iter_laws(), result.decrease() is Some,
            result.remaining().len() <= self.index_len(),
            result.will_return_none() ==> result.remaining().len() == self.index_len(),
            forall|i: int| 0 <= i < result.remaining().len()
                ==> #[trigger] result.remaining()[i] == self.value_at(i),
    ))]
    pub fn values(&self) -> impl ExactSizeIterator<Item = usize> + FusedIterator + '_ {
        let len = self.len();
        (0..len).map(
            #[cfg_attr(verus_keep_ghost, verus_spec(result: usize =>
                requires index < self.index_len(),
                ensures result == self.value_at(index as int),
            ))]
            move |index| unsafe { self.index_to_value_unchecked(index) },
        )
    }

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(),
        ensures
            result.obeys_prophetic_iter_laws(), result.decrease() is Some,
            result.remaining().len() <= self.index_len(),
            result.will_return_none() ==> result.remaining().len() == self.index_len(),
            forall|i: int| 0 <= i < result.remaining().len() ==> {
                let entry = #[trigger] result.remaining()[i];
                entry.1 == self.value_at(i)
                    && 1 <= entry.0@.start <= entry.0@.end <= self@
                    && forall|key: int| 1 <= key <= self@ ==>
                        ((entry.0@.start <= key <= entry.0@.end)
                            == (#[trigger] (self@ as int / key) == entry.1))
            },
    ))]
    pub fn key_values(
        &self,
    ) -> impl ExactSizeIterator<Item = (RangeInclusive<usize>, usize)> + FusedIterator + '_ {
        let len = self.len();
        (0..len).map(
            #[cfg_attr(verus_keep_ghost, verus_spec(result: (RangeInclusive<usize>, usize) =>
                requires index < self.index_len(),
                ensures result.1 == self.value_at(index as int),
                    1 <= result.0@.start <= result.0@.end <= self@,
                    forall|key: int| 1 <= key <= self@ ==>
                        ((result.0@.start <= key <= result.0@.end)
                            == (#[trigger] (self@ as int / key) == result.1)),
            ))]
            move |index| unsafe {
                (
                    self.index_to_key_unchecked(index),
                    self.index_to_value_unchecked(index),
                )
            },
        )
    }

    #[allow(clippy::len_without_is_empty)]
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(),
        ensures result == self.index_len(), result > 0,
    ))]
    pub fn len(&self) -> usize {
        #[cfg(verus_keep_ghost)]
        proof! { self.index_bounds(); }
        2 * self.sqrt - (self.pivot == self.sqrt) as usize
    }

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures result == (1 <= key <= self@),
    ))]
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

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(), index < self.index_len(),
        ensures
            1 <= result@.start <= result@.end <= self@,
            forall|key: int| 1 <= key <= self@ ==>
                ((result@.start <= key <= result@.end)
                    == (#[trigger] (self@ as int / key) == self.value_at(index as int))),
    ))]
    pub fn index_to_key(&self, index: usize) -> RangeInclusive<usize> {
        assert!(index < self.len(), "index out of bounds");
        unsafe { self.index_to_key_unchecked(index) }
    }

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(), index < self.index_len(),
        ensures result == self.value_at(index as int), floor_quotient(self@ as int, result as int),
    ))]
    pub fn index_to_value(&self, index: usize) -> usize {
        assert!(index < self.len(), "index out of bounds");
        unsafe { self.index_to_value_unchecked(index) }
    }

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(), floor_quotient(self@ as int, value as int),
        ensures result < self.index_len(), self.value_at(result as int) == value,
    ))]
    pub fn value_to_index(&self, value: usize) -> usize {
        assert!(self.contains_value(value), "value is not present");
        unsafe { self.value_to_index_unchecked(value) }
    }

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self@ > 0, floor_quotient(self@ as int, value as int),
        ensures
            1 <= result@.start <= result@.end <= self@,
            forall|k: int| 1 <= k <= self@ ==>
                ((result@.start <= k <= result@.end) == (#[trigger] (self@ as int / k) == value)),
    ))]
    pub fn value_to_key(&self, value: usize) -> RangeInclusive<usize> {
        assert!(self.contains_value(value), "value is not present");
        unsafe { self.value_to_key_unchecked(value) }
    }

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(), 1 <= key <= self@,
        ensures result < self.index_len(), self.value_at(result as int) == self@ as int / key as int,
    ))]
    pub fn key_to_index(&self, key: usize) -> usize {
        assert!(self.contains_key(key), "key out of bounds");
        let value = self.key_to_value(key);
        unsafe { self.value_to_index_unchecked(value) }
    }

    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self@ > 0, 1 <= key <= self@,
        ensures result == self@ as int / (key as int), floor_quotient(self@ as int, result as int),
    ))]
    pub fn key_to_value(&self, key: usize) -> usize {
        assert!(self.contains_key(key), "key out of bounds");
        unsafe { self.key_to_value_unchecked(key) }
    }

    /// # Safety
    /// `index` must satisfy `index < self.len()`.
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(), index < self.index_len(),
        ensures
            1 <= result@.start <= result@.end <= self@,
            forall|key: int| 1 <= key <= self@ ==>
                ((result@.start <= key <= result@.end)
                    == (#[trigger] (self@ as int / key) == self.value_at(index as int))),
    ))]
    pub unsafe fn index_to_key_unchecked(&self, index: usize) -> RangeInclusive<usize> {
        #[cfg(verus_keep_ghost)]
        proof! { self.index_bounds(); self.index_value_properties(index as int); }
        if index < self.sqrt {
            #[cfg(verus_keep_ghost)]
            proof! {
                let value = index + 1;
                floor_membership(self.num, value as usize);
                assert forall|key: int| 1 <= key <= self.num implies
                    ((self.num as int / (value + 1) < key <= self.num as int / value)
                        == (#[trigger] (self.num as int / key) == self.value_at(index as int))) by {
                    quotient_interval(self.num as int, value, key);
                }
            }
            self.num / (index + 2) + 1..=self.num / (index + 1)
        } else {
            let key = self.len() - index;
            #[cfg(verus_keep_ghost)]
            proof! {
                assert forall|k: int| 1 <= k <= self.num implies
                    ((k == key) == (#[trigger] (self.num as int / k) == self.value_at(index as int))) by {
                    if self.num as int / k == self.value_at(index as int) {
                        quotient_at_least(self.num as int, k, self.sqrt + 1);
                        small_floor_quotient(self.num as int, self.sqrt as int, k);
                        floor_reciprocal(self.num as int, k);
                    }
                }
            }
            key..=key
        }
    }

    /// # Safety
    /// `index` must satisfy `index < self.len()`.
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(), index < self.index_len(),
        ensures result == self.value_at(index as int), floor_quotient(self@ as int, result as int),
    ))]
    pub unsafe fn index_to_value_unchecked(&self, index: usize) -> usize {
        #[cfg(verus_keep_ghost)]
        proof! { self.index_value_properties(index as int); }
        if index < self.sqrt {
            index + 1
        } else {
            self.num / (self.len() - index)
        }
    }

    /// # Safety
    /// `value` must satisfy `self.contains_value(value)`.
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(), floor_quotient(self@ as int, value as int),
        ensures result < self.index_len(), self.value_at(result as int) == value,
    ))]
    pub unsafe fn value_to_index_unchecked(&self, value: usize) -> usize {
        #[cfg(verus_keep_ghost)]
        proof! {
            self.index_bounds();
            floor_reciprocal(self.num as int, value as int);
        }
        if value <= self.sqrt {
            value - 1
        } else {
            #[cfg(verus_keep_ghost)]
            proof! {
                vstd::arithmetic::div_mod::lemma_div_is_ordered_by_denominator(
                    self.num as int, self.sqrt + 1, value as int);
            }
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
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.index_invariant(), 1 <= key <= self@,
        ensures result < self.index_len(), self.value_at(result as int) == self@ as int / key as int,
    ))]
    pub unsafe fn key_to_index_unchecked(&self, key: usize) -> usize {
        #[cfg(verus_keep_ghost)]
        proof! {
            self.index_bounds();
            quotient_at_least(self.num as int, key as int, self.sqrt + 1);
            assert(floor_quotient(self.num as int, self.num as int / key as int));
            floor_reciprocal(self.num as int, self.num as int / key as int);
        }
        let len = self.len();
        if key <= len - self.sqrt {
            len - key
        } else {
            self.num / key - 1
        }
    }

    /// # Safety
    /// `key` must satisfy `1 <= key && key <= self.num`.
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self@ > 0, 1 <= key <= self@,
        ensures result == self@ as int / (key as int), floor_quotient(self@ as int, result as int),
    ))]
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
    impl FloorQuotientIndex {
        pub closed spec fn index_invariant(&self) -> bool {
            self.num > 0 && self.sqrt > 0
            && self.sqrt as int * self.sqrt as int <= self.num
            && self.num < (self.sqrt + 1) * (self.sqrt + 1)
            && self.pivot == self.num as int / self.sqrt as int
        }

        pub closed spec fn index_len(&self) -> int {
            2 * self.sqrt - if self.pivot == self.sqrt { 1int } else { 0int }
        }

        pub closed spec fn value_at(&self, index: int) -> int
            recommends self.index_invariant(), 0 <= index < self.index_len(),
        {
            if index < self.sqrt { index + 1 } else { self.num as int / (self.index_len() - index) }
        }

        proof fn index_bounds(&self)
            requires self.index_invariant(),
            ensures
                2 * self.sqrt <= usize::MAX,
                0 < self.index_len() <= usize::MAX,
                self.sqrt <= self.index_len() <= 2 * self.sqrt,
                self.index_len() - self.sqrt == self.num as int / (self.sqrt + 1),
        {
            floor_split(self.num as int, self.sqrt as int);
            assert(2 * self.sqrt <= usize::MAX) by(nonlinear_arith)
                requires self.sqrt > 0, self.sqrt as int * self.sqrt as int <= self.num,
                    self.num <= usize::MAX, usize::MAX >= 2;
        }

        proof fn index_value_properties(&self, index: int)
            requires self.index_invariant(), 0 <= index < self.index_len(),
            ensures
                1 <= self.value_at(index) <= self.num,
                floor_quotient(self.num as int, self.value_at(index)),
                index >= self.sqrt ==> self.sqrt < self.value_at(index)
                    && self.num as int / self.value_at(index) == self.index_len() - index,
        {
            self.index_bounds();
            let n = self.num as int;
            let s = self.sqrt as int;
            if index < s {
                small_floor_quotient(n, s, index + 1);
                floor_reciprocal(n, index + 1);
            } else {
                let key = self.index_len() - index;
                small_floor_quotient(n, s, key);
                floor_reciprocal(n, key);
                quotient_at_least(n, key, s + 1);
                assert(1 <= key <= n && n / key == self.value_at(index));
            }
        }

        proof fn index_order(&self, i: int, j: int)
            requires self.index_invariant(), 0 <= i < j < self.index_len(),
            ensures self.value_at(i) < self.value_at(j),
        {
            self.index_value_properties(i);
            self.index_value_properties(j);
            if i >= self.sqrt {
                vstd::arithmetic::div_mod::lemma_div_is_ordered_by_denominator(
                    self.num as int, self.index_len() - j, self.index_len() - i);
            }
        }
    }

    impl View for FloorQuotientIndex {
        type V = usize;
        closed spec fn view(&self) -> usize { self.num }
    }

}

#[cfg(any(verus_keep_ghost, kani))]
mod verification {
    #[cfg(kani)]
    use super::CeilQuotientIndex;
    use super::FloorQuotientIndex;
    #[cfg(verus_keep_ghost)]
    use super::RangeInclusive;
    #[cfg(verus_keep_ghost)]
    use vstd::{arithmetic::div_mod::*, prelude::*};

    #[cfg(verus_keep_ghost)]
    verus! {
        pub fn collect_quotients(num: usize, sqrt: usize)
            requires sqrt > 0, sqrt as int * sqrt as int <= num < (sqrt + 1) * (sqrt + 1),
        {
            assert(num > 0) by(nonlinear_arith)
                requires sqrt > 0, sqrt as int * sqrt as int <= num;
            let index = FloorQuotientIndex { num, sqrt, pivot: num / sqrt };
            let len = index.len();
            let values: Vec<usize> = index.values().collect();
            let key_values: Vec<(RangeInclusive<usize>, usize)> = index.key_values().collect();
            assert(values.len() == len && key_values.len() == len);
            assert forall|i: int, j: int| 0 <= i < j < values.len() implies
                #[trigger] values@[i] < #[trigger] values@[j] by {
                index.index_order(i, j);
            }
            proof {
                index.index_bounds();
                assert forall|value: usize| floor_quotient(num as int, value as int) implies
                    #[trigger] values@.contains(value) by {
                    floor_reciprocal(num as int, value as int);
                    let position = if value <= sqrt { value - 1 } else { len - num as int / value as int };
                    if value > sqrt {
                        lemma_div_is_ordered_by_denominator(num as int, sqrt + 1, value as int);
                    }
                    assert(0 <= position < len && index.value_at(position) == value);
                    assert(values@[position] == value);
                }
                assert forall|value: usize| #[trigger] values@.contains(value) implies
                    floor_quotient(num as int, value as int) by {
                    let position = choose|i: int| 0 <= i < values.len() && #[trigger] values@[i] == value;
                    index.index_value_properties(position);
                }
            }
            #[verifier::proof_note("Collecting values for any valid floor quotient index returns exactly all distinct floor(n/k) values for 1 <= k <= n, in strictly increasing order")]
            assert((forall|value: usize| #[trigger] values@.contains(value)
                == floor_quotient(num as int, value as int))
                && forall|i: int, j: int| 0 <= i < j < values.len() ==>
                    #[trigger] values@[i] < #[trigger] values@[j]);
            #[verifier::proof_note("Collecting key_values returns one nonempty exact key interval per quotient, in the same order and with the same length as values")]
            assert(key_values.len() == values.len()
                && forall|i: int| 0 <= i < key_values.len() ==> {
                    let entry = #[trigger] key_values@[i];
                    entry.1 == values@[i]
                        && 1 <= entry.0@.start <= entry.0@.end <= num
                        && forall|key: int| 1 <= key <= num ==>
                            ((entry.0@.start <= key <= entry.0@.end)
                                == (#[trigger] (num as int / key) == entry.1))
                });
        }

        pub fn index_round_trip(num: usize, sqrt: usize, position: usize, key: usize)
            requires sqrt > 0, sqrt as int * sqrt as int <= num < (sqrt + 1) * (sqrt + 1),
                1 <= key <= num,
        {
            let index = FloorQuotientIndex { num, sqrt, pivot: num / sqrt };
            let len = index.len();
            if position < len {
                let value = index.index_to_value(position);
                let reverse = index.value_to_index(value);
                proof {
                    if reverse < position { index.index_order(reverse as int, position as int); }
                    if position < reverse { index.index_order(position as int, reverse as int); }
                }
                let keys = index.index_to_key(position);
                let value_keys = index.value_to_key(value);
                assert(keys@.start == value_keys@.start && keys@.end == value_keys@.end) by {
                    assert(num as int / keys@.start as int == value);
                    assert(num as int / keys@.end as int == value);
                    assert(num as int / value_keys@.start as int == value);
                    assert(num as int / value_keys@.end as int == value);
                }
                #[verifier::proof_note("For every positive usize numerator, its integer square root and every valid index, index-to-value and value-to-index round trip exactly, and index-to-key returns the same complete interval as value-to-key")]
                assert(reverse == position && keys@.start == value_keys@.start
                    && keys@.end == value_keys@.end);
            }
            let position = index.key_to_index(key);
            let unchecked_position = unsafe { index.key_to_index_unchecked(key) };
            let value = index.index_to_value(position);
            let expected = index.key_to_value(key);
            proof {
                if position < unchecked_position { index.index_order(position as int, unchecked_position as int); }
                if unchecked_position < position { index.index_order(unchecked_position as int, position as int); }
            }
            #[verifier::proof_note("For every valid key, both checked and unchecked key-to-index produce the same index, and mapping that index to a value yields exactly the key's floor quotient")]
            assert(position == unchecked_position && value == expected);
            assert forall|i: int, j: int| 0 <= i < j < len implies
                #[trigger] index.value_at(i) < #[trigger] index.value_at(j) by {
                index.index_order(i, j);
            }
            #[verifier::proof_note("The indexed quotient values are strictly increasing, so no two valid indices represent the same floor quotient")]
            assert(forall|i: int, j: int| 0 <= i < j < len ==>
                #[trigger] index.value_at(i) < #[trigger] index.value_at(j));
        }

        pub fn key_round_trip(index: &FloorQuotientIndex, key: usize)
            requires index@ > 0, 1 <= key <= index@,
        {
            let present_key = index.contains_key(key);
            let value = index.key_to_value(key);
            let present_value = index.contains_value(value);
            let keys = index.value_to_key(value);
            #[verifier::proof_note("For every positive usize numerator and valid key, the safe quotient APIs recognize the key and its quotient and return exactly the interval of keys producing that quotient, including the original key")]
            assert(present_key && present_value && keys@.start <= key <= keys@.end
                && forall|k: int| 1 <= k <= index@ ==>
                    ((keys@.start <= k <= keys@.end) == (#[trigger] (index@ as int / k) == value)));
        }

        pub open spec fn floor_quotient(n: int, value: int) -> bool {
            exists|k: int| 1 <= k <= n && #[trigger] (n / k) == value
        }

        pub proof fn small_floor_quotient(n: int, s: int, value: int)
            requires 0 < s, s * s <= n, 1 <= value <= s,
            ensures floor_quotient(n, value),
        {
            assert(s * value <= s * s) by(nonlinear_arith)
                requires 0 < s, 1 <= value <= s;
            lemma_div_is_ordered(s * value, n, value);
            lemma_div_by_multiple(s, value);
            lemma_div_nonincreasing(n, value);
            lemma_fundamental_div_mod(n, value);
            lemma_mod_bound(n, value);
            let key = n / value;
            lemma_fundamental_div_mod_converse(n, key, value, n % value);
            assert(1 <= key <= n && n / key == value);
        }

        pub proof fn quotient_at_least(n: int, key: int, value: int)
            requires n >= 0, key > 0, value > 0,
            ensures (value <= n / key) == (key <= n / value),
        {
            lemma_fundamental_div_mod(n, key);
            lemma_mod_bound(n, key);
            lemma_fundamental_div_mod(n, value);
            lemma_mod_bound(n, value);
            assert((value <= n / key) == (key * value <= n)) by(nonlinear_arith)
                requires key > 0, n == key * (n / key) + n % key, 0 <= n % key < key;
            assert((key <= n / value) == (key * value <= n)) by(nonlinear_arith)
                requires value > 0, n == value * (n / value) + n % value, 0 <= n % value < value;
        }

        pub proof fn floor_reciprocal(n: int, value: int)
            requires n > 0, floor_quotient(n, value),
            ensures 1 <= value <= n, 1 <= n / value <= n, n / (n / value) == value,
        {
            let key = choose|key: int| 1 <= key <= n && #[trigger] (n / key) == value;
            lemma_div_is_ordered(key, n, key);
            lemma_div_nonincreasing(n, key);
            quotient_interval(n, value, key);
            lemma_div_nonincreasing(n, value);
            quotient_interval(n, value, n / value);
        }

        pub proof fn floor_split(n: int, s: int)
            requires s > 0, s * s <= n < (s + 1) * (s + 1),
            ensures s <= n / s,
                n / (s + 1) == s - if n / s == s { 1int } else { 0int },
        {
            lemma_fundamental_div_mod(n, s);
            lemma_mod_bound(n, s);
            lemma_fundamental_div_mod(n, s + 1);
            lemma_mod_bound(n, s + 1);
            assert(s <= n / s) by(nonlinear_arith)
                requires s > 0, s * s <= n, n == s * (n / s) + n % s, 0 <= n % s < s;
            assert(s - 1 <= n / (s + 1) <= s) by(nonlinear_arith)
                requires s > 0, s * s <= n < (s + 1) * (s + 1),
                    n == (s + 1) * (n / (s + 1)) + n % (s + 1), 0 <= n % (s + 1) < s + 1;
            if n / s == s {
                assert(n / (s + 1) == s - 1) by(nonlinear_arith)
                    requires s > 0, n / s == s, s - 1 <= n / (s + 1) <= s,
                        n == s * (n / s) + n % s, 0 <= n % s < s,
                        n == (s + 1) * (n / (s + 1)) + n % (s + 1), 0 <= n % (s + 1);
            } else {
                assert(n / (s + 1) == s) by(nonlinear_arith)
                    requires s > 0, s < n / s, s - 1 <= n / (s + 1) <= s,
                        n == s * (n / s) + n % s, 0 <= n % s,
                        n == (s + 1) * (n / (s + 1)) + n % (s + 1), n % (s + 1) < s + 1;
            }
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

    #[cfg(all(kani, target_pointer_width = "64"))]
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_quotient_constructors_u64_max() {
        let distance: u64 = kani::any();
        let root = u32::MAX as usize;
        kani::assume(distance <= 2 * root as u64);
        let num = usize::MAX - distance as usize;
        let floor = FloorQuotientIndex::new(num);
        let ceil = CeilQuotientIndex::new(num);
        let floor_pivot = if distance == 0 {
            root + 2
        } else if distance <= root as u64 {
            root + 1
        } else {
            root
        };
        let ceil_pivot = if distance < root as u64 {
            root + 2
        } else if distance < 2 * root as u64 {
            root + 1
        } else {
            root
        };
        kani::assert(
            floor.num == num && floor.sqrt == root && floor.pivot == floor_pivot,
            "On 64-bit targets, for every numerator from (2^32-1)^2 through usize::MAX, the actual floor constructor stores square root 2^32-1 and its exact floor quotient; all three pivot values and both square-interval endpoints are covered",
        );
        kani::assert(
            ceil.num == num && ceil.sqrt == root && ceil.pivot == ceil_pivot,
            "On 64-bit targets, for every numerator from (2^32-1)^2 through usize::MAX, the actual ceiling constructor stores square root 2^32-1 and its exact ceiling quotient; all three pivot values and both square-interval endpoints are covered",
        );
        kani::cover!(distance == 0);
        kani::cover!(distance == 1);
        kani::cover!(distance == root as u64);
        kani::cover!(distance == root as u64 + 1);
        kani::cover!(distance == 2 * root as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_small_constructors() {
        for num in 1..=u16::MAX as usize {
            let floor = FloorQuotientIndex::new(num);
            let ceil = CeilQuotientIndex::new(num);
            assert_eq!(floor.sqrt, num.isqrt());
            assert_eq!(floor.num, num);
            assert_eq!(floor.pivot, num / floor.sqrt);
            assert_eq!(ceil.num, num);
            assert_eq!(ceil.sqrt, floor.sqrt);
            assert_eq!(ceil.pivot, num.div_ceil(floor.sqrt));
        }
    }

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
