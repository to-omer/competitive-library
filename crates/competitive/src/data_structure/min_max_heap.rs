/// Double-ended priority queue with linear-time heap construction.
#[derive(Clone, Debug)]
pub struct MinMaxHeap<T> {
    data: Vec<T>,
}

impl<T> Default for MinMaxHeap<T> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<T> FromIterator<T> for MinMaxHeap<T>
where
    T: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let data: Vec<_> = iter.into_iter().collect();
        Self::from(data)
    }
}

impl<T> Extend<T> for MinMaxHeap<T>
where
    T: Ord,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

impl<T> From<Vec<T>> for MinMaxHeap<T>
where
    T: Ord,
{
    fn from(data: Vec<T>) -> Self {
        let mut heap = Self { data };
        for i in (0..heap.len() / 2).rev() {
            if Self::is_min_level(i) {
                heap.sift_down::<false>(i);
            } else {
                heap.sift_down::<true>(i);
            }
        }
        heap
    }
}

impl<T> MinMaxHeap<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn peek_min(&self) -> Option<&T> {
        self.data.first()
    }
}

impl<T> MinMaxHeap<T>
where
    T: Ord,
{
    pub fn peek_max(&self) -> Option<&T> {
        self.data.get(self.max_index())
    }

    pub fn push(&mut self, value: T) {
        let i = self.len();
        self.data.push(value);
        if i == 0 {
            return;
        }
        let parent = (i - 1) / 2;
        if Self::is_min_level(i) {
            if self.data[i] > self.data[parent] {
                self.data.swap(i, parent);
                self.sift_up::<true>(parent);
            } else {
                self.sift_up::<false>(i);
            }
        } else if self.data[i] < self.data[parent] {
            self.data.swap(i, parent);
            self.sift_up::<false>(parent);
        } else {
            self.sift_up::<true>(i);
        }
    }

    pub fn pop_min(&mut self) -> Option<T> {
        self.pop::<false>()
    }

    pub fn pop_max(&mut self) -> Option<T> {
        self.pop::<true>()
    }

    /// Unconditionally replaces the least value, or inserts into an empty heap.
    pub fn replace_min(&mut self, value: T) -> Option<T> {
        self.replace::<false>(value)
    }

    /// Unconditionally replaces the greatest value, or inserts into an empty heap.
    pub fn replace_max(&mut self, value: T) -> Option<T> {
        self.replace::<true>(value)
    }

    /// Returns the elements in ascending order.
    pub fn into_sorted_vec(mut self) -> Vec<T> {
        self.data.sort_unstable();
        self.data
    }

    fn replace<const MAX: bool>(&mut self, value: T) -> Option<T> {
        if self.data.is_empty() {
            self.data.push(value);
            return None;
        }
        let i = if MAX { self.max_index() } else { 0 };
        let old = std::mem::replace(&mut self.data[i], value);
        if MAX && self.data[0] > self.data[i] {
            self.data.swap(0, i);
        }
        self.sift_down::<MAX>(i);
        Some(old)
    }

    fn pop<const MAX: bool>(&mut self) -> Option<T> {
        if self.data.len() <= 1 {
            return self.data.pop();
        }
        let i = if MAX { self.max_index() } else { 0 };
        let value = self.data.swap_remove(i);
        self.sift_down::<MAX>(i);
        Some(value)
    }

    fn max_index(&self) -> usize {
        match self.len() {
            0 | 1 => 0,
            2 => 1,
            _ => 1 + usize::from(self.data[2] > self.data[1]),
        }
    }

    fn is_min_level(i: usize) -> bool {
        (i + 1).ilog2().is_multiple_of(2)
    }

    fn sift_up<const MAX: bool>(&mut self, mut i: usize) {
        while i >= 3 {
            let grandparent = (i - 3) / 4;
            if !Self::precedes::<MAX>(&self.data[i], &self.data[grandparent]) {
                break;
            }
            self.data.swap(i, grandparent);
            i = grandparent;
        }
    }

    fn precedes<const MAX: bool>(a: &T, b: &T) -> bool {
        if MAX { a > b } else { a < b }
    }

    fn sift_down<const MAX: bool>(&mut self, mut i: usize) {
        let n = self.len();
        while i < n / 2 {
            let child = 2 * i + 1;
            let grandchild = 4 * i + 3;
            let better = |a, b| Self::precedes::<MAX>(&self.data[a], &self.data[b]);
            let best = if grandchild + 3 < n {
                let [a, b, c, d] = self.data[grandchild..].first_chunk().unwrap();
                let left = Self::precedes::<MAX>(b, a);
                let right = Self::precedes::<MAX>(d, c);
                let left_index = grandchild + usize::from(left);
                let right_index = grandchild + 2 + usize::from(right);
                if Self::precedes::<MAX>(if left { b } else { a }, if right { d } else { c }) {
                    left_index
                } else {
                    right_index
                }
            } else {
                let mut best = child;
                for j in (child + 1..(child + 2).min(n)).chain(grandchild..(grandchild + 4).min(n))
                {
                    if better(j, best) {
                        best = j;
                    }
                }
                best
            };
            if !better(best, i) {
                break;
            }
            self.data.swap(i, best);
            if best < grandchild {
                break;
            }
            let parent = (best - 1) / 2;
            if Self::precedes::<MAX>(&self.data[parent], &self.data[best]) {
                self.data.swap(parent, best);
            }
            i = best;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MinMaxHeap;
    use crate::tools::Xorshift;
    use std::collections::BTreeMap;

    #[test]
    fn test_min_max_heap() {
        let mut rng = Xorshift::default();
        for trial in 0..70 {
            let mode = trial % 6;
            let initial = if trial < 64 { trial } else { 105_000 + mode };
            let operations = if trial < 64 {
                rng.rand(128) as usize
            } else {
                105_000 + rng.rand(16) as usize
            };
            let values: Vec<i32> = (0..(initial + operations) as i32)
                .map(|i| match (i % 101, mode) {
                    (_, 0) => i,
                    (_, 1) => -i,
                    (_, 2) => 7,
                    (0, _) => i32::MIN,
                    (1, _) => i32::MAX,
                    _ => rng.rand64() as i32,
                })
                .collect();
            let mut heap: MinMaxHeap<_> = values[..initial].iter().copied().collect();
            let mut expected = BTreeMap::new();
            for &value in &values[..initial] {
                *expected.entry(value).or_insert(0) += 1;
            }
            let mut next = initial;
            for step in 0.. {
                assert_eq!(heap.peek_min(), expected.first_key_value().map(|(k, _)| k));
                assert_eq!(heap.peek_max(), expected.last_key_value().map(|(k, _)| k));
                if next < values.len() && (expected.is_empty() || rng.rand(5) < 3) {
                    heap.push(values[next]);
                    *expected.entry(values[next]).or_insert(0) += 1;
                    next += 1;
                } else {
                    let min = match mode {
                        0 => true,
                        1 => false,
                        2 => step % 2 == 0,
                        _ => rng.rand(2) == 0,
                    };
                    let actual = if min { heap.pop_min() } else { heap.pop_max() };
                    let entry = if min {
                        expected.first_entry()
                    } else {
                        expected.last_entry()
                    };
                    let Some(mut entry) = entry else {
                        assert_eq!(actual, None);
                        break;
                    };
                    assert_eq!(actual, Some(*entry.key()));
                    *entry.get_mut() -= 1;
                    if *entry.get() == 0 {
                        entry.remove();
                    }
                }
            }
        }
    }

    #[test]
    fn test_replace() {
        let mut rng = Xorshift::default();
        for (trial, n) in (0..64).chain([0, 0, 255, 4096]).enumerate() {
            let values: Vec<_> = (0..n).map(|_| rng.rand(128).to_string()).collect();
            let mut heap = match trial % 3 {
                0 => MinMaxHeap::new(),
                1 => MinMaxHeap::default(),
                _ => MinMaxHeap::with_capacity(n),
            };
            heap.extend(values.iter().cloned());
            let mut expected = values;
            expected.sort();
            for step in 0..256 {
                let value = rng.rand(128).to_string();
                if (step + trial) % 2 == 0 {
                    let old = (!expected.is_empty()).then(|| expected.remove(0));
                    assert_eq!(heap.replace_min(value.clone()), old);
                } else {
                    let old = expected.pop();
                    assert_eq!(heap.replace_max(value.clone()), old);
                }
                let i = expected.partition_point(|x| x < &value);
                expected.insert(i, value);
                assert_eq!(heap.len(), expected.len());
            }
            assert_eq!(heap.clone().into_sorted_vec(), expected);
            heap.clear();
            assert!(heap.is_empty());
        }
    }
}
