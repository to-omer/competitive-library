use super::{AdditiveOperation, BinaryIndexedTree, FibHashMap};
use std::{collections::hash_map::Entry, hash::Hash, mem::replace};

#[derive(Debug, Clone, Copy)]
enum RangeFrequencyQuery {
    Add {
        index: u32,
    },
    Remove {
        index: u32,
    },
    Query {
        left: u32,
        right: u32,
        output_index: u32,
    },
}

#[derive(Debug, Clone)]
pub struct RangeFrequency<T>
where
    T: Clone + Eq + Hash,
{
    array: Vec<u32>,
    values: FibHashMap<T, u32>,
    events: Vec<(u32, RangeFrequencyQuery)>,
    queried: Vec<u8>,
    static_queries: Option<Vec<(u32, u32, u32, u32)>>,
    zero_queries: Vec<u32>,
    output_size: usize,
}

impl<T> RangeFrequency<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(array: Vec<T>) -> Self {
        let mut result = Self {
            array: Vec::with_capacity(array.len()),
            values: FibHashMap::with_capacity_and_hasher(array.len(), Default::default()),
            events: Vec::new(),
            queried: Vec::new(),
            static_queries: Some(Vec::new()),
            zero_queries: Vec::new(),
            output_size: 0,
        };
        for value in array {
            let value = result.value_id(value);
            result.array.push(value);
        }
        result
    }

    fn value_id(&mut self, value: T) -> u32 {
        match self.values.entry(value) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = self.queried.len() as u32;
                entry.insert(id);
                self.queried.push(0);
                id
            }
        }
    }

    pub fn set(&mut self, index: usize, value: T) {
        if let Some(queries) = self.static_queries.take() {
            self.events.reserve(self.array.len() + queries.len() + 2);
            for (index, &value) in self.array.iter().enumerate() {
                self.events.push((
                    value,
                    RangeFrequencyQuery::Add {
                        index: index as u32,
                    },
                ));
            }
            for (left, right, value, output_index) in queries {
                self.events.push((
                    value,
                    RangeFrequencyQuery::Query {
                        left,
                        right,
                        output_index,
                    },
                ));
            }
        }
        let value = self.value_id(value);
        let old_value = replace(&mut self.array[index], value);
        self.events.push((
            old_value,
            RangeFrequencyQuery::Remove {
                index: index as u32,
            },
        ));
        self.events.push((
            value,
            RangeFrequencyQuery::Add {
                index: index as u32,
            },
        ));
    }

    pub fn query(&mut self, left: usize, right: usize, value: T) -> usize {
        let output_index = self.output_size;
        if let Some(&value) = self.values.get(&value) {
            self.queried[value as usize] = 1;
            if let Some(queries) = &mut self.static_queries {
                queries.push((left as u32, right as u32, value, output_index as u32));
            } else {
                self.events.push((
                    value,
                    RangeFrequencyQuery::Query {
                        left: left as u32,
                        right: right as u32,
                        output_index: output_index as u32,
                    },
                ));
            }
        } else {
            self.zero_queries.push(output_index as u32);
        }
        self.output_size += 1;
        output_index
    }

    pub fn execute_with_callback(mut self, mut callback: impl FnMut(usize, usize)) {
        for output_index in self.zero_queries {
            callback(output_index as usize, 0);
        }
        if let Some(mut queries) = self.static_queries.take() {
            let n = self.array.len();
            if queries.is_empty() {
                return;
            }
            let mut offsets = vec![0; n + 2];
            for &(left, right, _, _) in &queries {
                if left < right {
                    offsets[left as usize + 1] += 1;
                    offsets[right as usize + 1] += 1;
                }
            }
            for i in 0..=n {
                offsets[i + 1] += offsets[i];
            }
            let mut next = offsets.clone();
            let mut events = vec![0u32; 2 * queries.len()];
            for (i, &(left, right, _, output_index)) in queries.iter().enumerate() {
                if left >= right {
                    callback(output_index as usize, 0);
                    continue;
                }
                for (side, position) in [left, right].into_iter().enumerate() {
                    events[next[position as usize]] = (2 * i + side) as u32;
                    next[position as usize] += 1;
                }
            }
            let mut count = vec![0u32; self.values.len()];
            for position in 0..=n {
                for &endpoint in &events[offsets[position]..offsets[position + 1]] {
                    let query = (endpoint >> 1) as usize;
                    let frequency = count[queries[query].2 as usize];
                    if endpoint & 1 == 0 {
                        queries[query].0 = frequency;
                    } else {
                        callback(
                            queries[query].3 as usize,
                            (frequency - queries[query].0) as usize,
                        );
                    }
                }
                if position < n {
                    count[self.array[position] as usize] += 1;
                }
            }
            return;
        }
        let mut processor = RangeFrequencyProcessor::new(self.array.len());
        for (index, value) in self.array.into_iter().enumerate() {
            self.events.push((
                value,
                RangeFrequencyQuery::Remove {
                    index: index as u32,
                },
            ));
        }
        let mut offsets = vec![0; self.queried.len() + 1];
        for &(value, _) in &self.events {
            offsets[value as usize + 1] += self.queried[value as usize] as usize;
        }
        for i in 0..self.queried.len() {
            offsets[i + 1] += offsets[i];
        }
        let mut next = offsets.clone();
        let mut events = vec![RangeFrequencyQuery::Add { index: 0 }; *offsets.last().unwrap()];
        for (value, event) in self.events {
            let value = value as usize;
            if self.queried[value] != 0 {
                events[next[value]] = event;
                next[value] += 1;
            }
        }
        for range in offsets.windows(2) {
            for &query in &events[range[0]..range[1]] {
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
                        callback(output_index as usize, processor.query(left, right));
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

    fn add(&mut self, index: u32) {
        let index = index as usize;
        let (block, bit) = (index / 64, index % 64);
        assert!(self.data[block] & (1 << bit) == 0);
        self.data[block] |= 1 << bit;
        self.bit.update(block, 1);
    }

    fn remove(&mut self, index: u32) {
        let index = index as usize;
        let (i, j) = (index / 64, index % 64);
        assert!(self.data[i] & (1 << j) != 0);
        self.data[i] &= !(1 << j);
        self.bit.update(i, -1);
    }

    fn query(&self, left: u32, right: u32) -> usize {
        if left >= right {
            return 0;
        }
        let (left, right) = (left as usize, right as usize - 1);
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
            rand!(rng, n: 1..200, mut a: [0..20; n]);
            let mut rf = RangeFrequency::new(a.clone());
            let mut expected = vec![];
            let dynamic = rng.gen_bool(0.5);
            for _ in 0..100 {
                if dynamic && rng.gen_bool(0.5) {
                    rand!(rng, i: 0..n, v: 0..20);
                    rf.set(i, v);
                    a[i] = v;
                }
                let (l, r) = rng.random(Nes(n));
                for v in 0..20 {
                    expected.push(a[l..r].iter().filter(|&&x| x == v).count());
                    rf.query(l, r, v);
                }
            }
            let result = rf.execute();
            assert_eq!(result, expected);
        }
    }
}
