use super::{AdditiveOperation, BinaryIndexedTree, FibHashMap};
use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
    mem::replace,
};

#[derive(Debug, Clone)]
enum RangeFrequencyQuery {
    Add {
        index: usize,
    },
    Remove {
        index: usize,
    },
    Query {
        left: usize,
        right: usize,
        output_index: usize,
    },
}

#[derive(Debug, Clone)]
pub struct RangeFrequency<T>
where
    T: Clone + Eq + Hash,
{
    array: Vec<T>,
    queries: HashMap<T, Vec<RangeFrequencyQuery>>,
    static_queries: Option<Vec<(usize, usize, T)>>,
    output_size: usize,
}

impl<T> RangeFrequency<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(array: Vec<T>) -> Self {
        Self {
            array,
            queries: HashMap::new(),
            static_queries: Some(Vec::new()),
            output_size: 0,
        }
    }

    pub fn set(&mut self, index: usize, value: T) {
        if let Some(queries) = self.static_queries.take() {
            for (index, value) in self.array.iter().cloned().enumerate() {
                self.queries
                    .entry(value)
                    .or_default()
                    .push(RangeFrequencyQuery::Add { index });
            }
            for (output_index, (left, right, value)) in queries.into_iter().enumerate() {
                self.queries
                    .entry(value)
                    .or_default()
                    .push(RangeFrequencyQuery::Query {
                        left,
                        right,
                        output_index,
                    });
            }
        }
        let old_value = replace(&mut self.array[index], value);
        self.queries
            .entry(old_value)
            .or_default()
            .push(RangeFrequencyQuery::Remove { index });
        self.queries
            .entry(self.array[index].clone())
            .or_default()
            .push(RangeFrequencyQuery::Add { index });
    }

    pub fn query(&mut self, left: usize, right: usize, value: T) -> usize {
        let output_index = self.output_size;
        if let Some(queries) = &mut self.static_queries {
            queries.push((left, right, value));
        } else {
            self.queries
                .entry(value)
                .or_default()
                .push(RangeFrequencyQuery::Query {
                    left,
                    right,
                    output_index,
                });
        }
        self.output_size += 1;
        output_index
    }

    pub fn execute_with_callback(mut self, mut callback: impl FnMut(usize, usize)) {
        if let Some(mut queries) = self.static_queries.take() {
            let n = self.array.len();
            if queries.is_empty() {
                return;
            }
            let mut offsets = vec![0; n + 2];
            for &(left, right, _) in &queries {
                if left < right {
                    offsets[left + 1] += 1;
                    offsets[right + 1] += 1;
                }
            }
            for i in 0..=n {
                offsets[i + 1] += offsets[i];
            }
            let mut next = offsets.clone();
            let mut events = vec![0; 2 * queries.len()];
            for (i, &(left, right, _)) in queries.iter().enumerate() {
                if left >= right {
                    callback(i, 0);
                    continue;
                }
                for (side, position) in [left, right].into_iter().enumerate() {
                    events[next[position]] = 2 * i + side;
                    next[position] += 1;
                }
            }
            let mut index = FibHashMap::with_capacity_and_hasher(n, Default::default());
            let mut count = Vec::with_capacity(n);
            let mut array = Vec::with_capacity(n);
            for value in self.array {
                let id = match index.entry(value) {
                    Entry::Occupied(entry) => *entry.get(),
                    Entry::Vacant(entry) => {
                        let id = count.len();
                        entry.insert(id);
                        count.push(0usize);
                        id
                    }
                };
                array.push(id);
            }
            for query in &mut queries {
                query.0 = index.get(&query.2).copied().unwrap_or(!0);
            }
            for position in 0..=n {
                for &endpoint in &events[offsets[position]..offsets[position + 1]] {
                    let query = endpoint >> 1;
                    let id = queries[query].0;
                    let frequency = if id == !0 { 0 } else { count[id] };
                    if endpoint & 1 == 0 {
                        queries[query].1 = frequency;
                    } else {
                        callback(query, frequency - queries[query].1);
                    }
                }
                if position < n {
                    count[array[position]] += 1;
                }
            }
            return;
        }
        let mut processor = RangeFrequencyProcessor::new(self.array.len());
        for (index, value) in self.array.into_iter().enumerate() {
            self.queries
                .entry(value)
                .or_default()
                .push(RangeFrequencyQuery::Remove { index });
        }
        for (_, queries) in self.queries {
            for query in queries {
                match query {
                    RangeFrequencyQuery::Add { index } => {
                        processor.add(index);
                    }
                    RangeFrequencyQuery::Remove { index } => {
                        processor.remove(index);
                    }
                    RangeFrequencyQuery::Query {
                        left,
                        right,
                        output_index,
                    } => {
                        callback(output_index, processor.query(left, right));
                    }
                }
            }
        }
    }

    pub fn execute(self) -> Vec<usize> {
        let mut results = vec![0; self.output_size];
        self.execute_with_callback(|i, v| results[i] = v);
        results
    }
}

#[derive(Debug, Clone)]
struct RangeFrequencyProcessor {
    bit: BinaryIndexedTree<AdditiveOperation<i32>>,
    data: Vec<u64>,
}

impl RangeFrequencyProcessor {
    fn new(size: usize) -> Self {
        Self {
            bit: BinaryIndexedTree::new(size.div_ceil(64)),
            data: vec![0; size.div_ceil(64)],
        }
    }

    fn add(&mut self, index: usize) {
        let (block, bit) = (index / 64, index % 64);
        assert!(self.data[block] & (1 << bit) == 0);
        self.data[block] |= 1 << bit;
        self.bit.update(block, 1);
    }

    fn remove(&mut self, index: usize) {
        let (i, j) = (index / 64, index % 64);
        assert!(self.data[i] & (1 << j) != 0);
        self.data[i] &= !(1 << j);
        self.bit.update(i, -1);
    }

    fn query(&self, left: usize, right: usize) -> usize {
        if left >= right {
            return 0;
        }
        let right = right - 1;
        let (li, lj) = (left / 64, left % 64);
        let (ri, rj) = (right / 64, right % 64);
        let rj_r = 63 - rj;
        if li == ri {
            (self.data[li] << rj_r >> (lj + rj_r)).count_ones() as usize
        } else {
            let mut ans = self.bit.fold(li + 1, ri) as usize;
            ans += (self.data[li] >> lj).count_ones() as usize;
            ans += (self.data[ri] << rj_r).count_ones() as usize;
            ans
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        rand,
        tools::{NotEmptySegment as Nes, Xorshift},
    };

    #[test]
    fn test_range_frequency() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..200, mut a: [0..10; n]);
            let mut rf = RangeFrequency::new(a.clone());
            for _ in 0..if rng.gen_bool(0.5) { 0 } else { 100 } {
                rand!(rng, i: 0..n, v: 0..10);
                rf.set(i, v);
                a[i] = v;
            }
            let mut expected = vec![];
            for _ in 0..100 {
                let (l, r) = rng.random(Nes(n));
                for v in 0..10 {
                    expected.push(a[l..r].iter().filter(|&&x| x == v).count());
                    rf.query(l, r, v);
                }
            }
            let result = rf.execute();
            assert_eq!(result, expected);
        }
    }
}
