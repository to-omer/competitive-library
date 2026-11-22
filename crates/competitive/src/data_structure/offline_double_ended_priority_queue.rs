use super::{RadixSortKey, SliceSortExt};

/// Double-ended priority queue with initial elements and reserved insertion handles.
#[derive(Clone, Debug)]
pub struct OfflineDoubleEndedPriorityQueue<T> {
    initial: Vec<T>,
    remaining: std::ops::Range<usize>,
    keys: Vec<T>,
    ranks: Vec<usize>,
    handles: Vec<usize>,
    reserved_len: usize,
    min_rank: usize,
    max_rank: usize,
    levels: Vec<Vec<u64>>,
}

#[derive(Clone, Debug)]
pub struct OfflineDoubleEndedPriorityQueueBuilder<T> {
    initial: Vec<T>,
    reserved: Vec<T>,
}

impl<T> OfflineDoubleEndedPriorityQueueBuilder<T> {
    /// Adds an element to the initial contents.
    pub fn insert(&mut self, key: T) {
        self.initial.push(key);
    }

    /// Returns the handle for a future insertion.
    pub fn reserve(&mut self, key: T) -> usize {
        let index = self.reserved.len();
        self.reserved.push(key);
        index
    }

    pub fn build(mut self) -> OfflineDoubleEndedPriorityQueue<T>
    where
        T: Ord,
    {
        self.initial.sort_unstable();
        let mut handles: Vec<_> = (0..self.reserved.len()).collect();
        handles.sort_unstable_by(|&a, &b| self.reserved[a].cmp(&self.reserved[b]));
        OfflineDoubleEndedPriorityQueue::from_sorted(self.initial, self.reserved, handles)
    }

    /// The radix order must agree with the key's ordering.
    pub fn build_radix(mut self) -> OfflineDoubleEndedPriorityQueue<T>
    where
        T: Ord + RadixSortKey,
    {
        self.initial.radix_sort_by_key(|&x| x);
        let mut handles: Vec<_> = (0..self.reserved.len()).collect();
        handles.radix_sort_by_key(|&handle| self.reserved[handle]);
        OfflineDoubleEndedPriorityQueue::from_sorted(self.initial, self.reserved, handles)
    }
}

impl<T> OfflineDoubleEndedPriorityQueue<T> {
    pub fn builder() -> OfflineDoubleEndedPriorityQueueBuilder<T> {
        OfflineDoubleEndedPriorityQueueBuilder {
            initial: Vec::new(),
            reserved: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.reserved_len + self.remaining.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        for level in &mut self.levels {
            level.fill(0);
        }
        self.remaining = 0..0;
        self.reserved_len = 0;
        self.min_rank = usize::MAX;
        self.max_rank = 0;
    }

    pub fn peek_min(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.peek::<false>()
    }

    pub fn peek_max(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.peek::<true>()
    }

    /// Inserts the element reserved by this queue's builder. Panics if the handle is already present.
    /// A popped or cleared handle may be pushed again.
    pub fn push(&mut self, handle: usize) {
        let mut i = self.ranks[handle];
        assert_eq!(
            self.levels[0][i / 64] & (1 << (i % 64)),
            0,
            "handle already present"
        );
        self.reserved_len += 1;
        self.min_rank = self.min_rank.min(i);
        self.max_rank = self.max_rank.max(i);
        for level in &mut self.levels {
            let word = &mut level[i / 64];
            let empty = *word == 0;
            *word |= 1 << (i % 64);
            if !empty {
                break;
            }
            i /= 64;
        }
    }

    pub fn pop_min(&mut self) -> Option<&T>
    where
        T: Ord,
    {
        self.pop::<false>()
    }

    pub fn pop_max(&mut self) -> Option<&T>
    where
        T: Ord,
    {
        self.pop::<true>()
    }

    fn from_sorted(initial: Vec<T>, keys: Vec<T>, handles: Vec<usize>) -> Self {
        let mut ranks = vec![0; keys.len()];
        for (rank, &handle) in handles.iter().enumerate() {
            ranks[handle] = rank;
        }
        let mut levels = Vec::new();
        let mut n = keys.len();
        loop {
            n = n.div_ceil(64);
            levels.push(vec![0; n]);
            if n <= 1 {
                break;
            }
        }
        Self {
            remaining: 0..initial.len(),
            initial,
            reserved_len: 0,
            min_rank: usize::MAX,
            max_rank: 0,
            keys,
            ranks,
            handles,
            levels,
        }
    }

    fn peek<const MAX: bool>(&self) -> Option<&T>
    where
        T: Ord,
    {
        let (initial, i) = self.endpoint::<MAX>()?;
        Some(if initial {
            &self.initial[i]
        } else {
            &self.keys[self.handles[i]]
        })
    }

    fn endpoint<const MAX: bool>(&self) -> Option<(bool, usize)>
    where
        T: Ord,
    {
        let i = (self.reserved_len != 0).then_some(if MAX { self.max_rank } else { self.min_rank });
        let mut range = self.remaining.clone();
        let j = if MAX { range.next_back() } else { range.next() };
        if let Some(j) = j
            && i.is_none_or(|i| {
                if MAX {
                    self.initial[j] >= self.keys[self.handles[i]]
                } else {
                    self.initial[j] <= self.keys[self.handles[i]]
                }
            })
        {
            return Some((true, j));
        }
        i.map(|i| (false, i))
    }

    fn pop<const MAX: bool>(&mut self) -> Option<&T>
    where
        T: Ord,
    {
        let (initial, mut i) = self.endpoint::<MAX>()?;
        if initial {
            if MAX {
                self.remaining.end -= 1;
            } else {
                self.remaining.start += 1;
            }
            return Some(&self.initial[i]);
        }
        let value = &self.keys[self.handles[i]];
        self.reserved_len -= 1;
        for depth in 0..self.levels.len() {
            let word = &mut self.levels[depth][i / 64];
            *word &= !(1 << (i % 64));
            i /= 64;
            if *word != 0 {
                // The next extreme belongs to the first ancestor that is still occupied.
                for level in self.levels[..=depth].iter().rev() {
                    let word = level[i];
                    let bit = if MAX {
                        63 - word.leading_zeros()
                    } else {
                        word.trailing_zeros()
                    };
                    i = i * 64 + bit as usize;
                }
                if MAX {
                    self.max_rank = i;
                } else {
                    self.min_rank = i;
                }
                return Some(value);
            }
        }
        self.min_rank = usize::MAX;
        self.max_rank = 0;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::OfflineDoubleEndedPriorityQueue;
    use crate::tools::Xorshift;
    use std::collections::BTreeMap;

    #[test]
    fn test_offline_double_ended_priority_queue() {
        let mut rng = Xorshift::default();
        for trial in 0..70 {
            let mode = trial % 6;
            let n = if trial < 64 {
                trial * 2
            } else if mode == 0 {
                64_usize.pow(3) + 1
            } else {
                160_000 + rng.rand(32) as usize
            };
            let initial = if trial < 64 {
                trial
            } else if mode == 0 {
                0
            } else {
                n / 2
            };
            let values: Vec<i64> = (0..n as i64)
                .map(|i| match (i % 101, mode) {
                    (_, 0) => i,
                    (_, 1) => -i,
                    (_, 2) => 7,
                    (0, _) => i64::MIN,
                    (1, _) => i64::MAX,
                    _ => rng.rand64() as i64,
                })
                .collect();
            let mut builder = OfflineDoubleEndedPriorityQueue::builder();
            let mut expected = BTreeMap::new();
            for &value in &values[..initial] {
                builder.insert(value);
                *expected.entry(value).or_insert(0) += 1;
            }
            let handles: Vec<_> = values[initial..]
                .iter()
                .map(|&v| builder.reserve(v))
                .collect();
            let mut heap = if mode % 2 == 0 {
                builder.build()
            } else {
                builder.build_radix()
            };
            let mut next = initial;
            for step in 0.. {
                assert_eq!(heap.peek_min(), expected.first_key_value().map(|(k, _)| k));
                assert_eq!(heap.peek_max(), expected.last_key_value().map(|(k, _)| k));
                if next < values.len() && (expected.is_empty() || rng.rand(5) < 3) {
                    heap.push(handles[next - initial]);
                    *expected.entry(values[next]).or_insert(0) += 1;
                    next += 1;
                } else {
                    let min = match mode {
                        0 => true,
                        1 => false,
                        2 => step % 2 == 0,
                        _ => rng.rand(2) == 0,
                    };
                    let actual = if min { heap.pop_min() } else { heap.pop_max() }.copied();
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
            assert!(heap.is_empty());
            for (&handle, value) in handles.iter().zip(&values[initial..]).rev() {
                heap.push(handle);
                assert_eq!(heap.pop_min(), Some(value));
            }
        }
    }

    #[test]
    fn test_clear() {
        let mut rng = Xorshift::default();
        for n in (0..128).chain([4095, 4096, 4097]) {
            let mut builder = OfflineDoubleEndedPriorityQueue::builder();
            let mut reserved = Vec::new();
            for _ in 0..n {
                builder.insert(rng.rand(128).to_string());
                let value = rng.rand(128).to_string();
                reserved.push((builder.reserve(value.clone()), value));
            }
            let mut heap = builder.build();
            for (handle, _) in &reserved {
                heap.push(*handle);
            }
            assert_eq!(heap.len(), 2 * n);
            heap.clear();
            assert!(heap.is_empty());
            assert_eq!(heap.peek_max(), None);
            for (handle, value) in reserved.into_iter().rev() {
                heap.push(handle);
                assert_eq!(heap.pop_max(), Some(&value));
            }
        }
    }
}
