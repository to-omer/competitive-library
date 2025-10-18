use super::{AdditiveOperation, BinaryIndexedTree};
use std::{collections::HashMap, hash::Hash, mem::replace};

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
    output_size: usize,
}

impl<T> RangeFrequency<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(array: Vec<T>) -> Self {
        let mut queries = HashMap::<T, Vec<RangeFrequencyQuery>>::new();
        for (index, value) in array.iter().cloned().enumerate() {
            queries
                .entry(value)
                .or_default()
                .push(RangeFrequencyQuery::Add { index });
        }
        Self {
            array,
            queries,
            output_size: 0,
        }
    }

    pub fn set(&mut self, index: usize, value: T) {
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
        self.queries
            .entry(value)
            .or_default()
            .push(RangeFrequencyQuery::Query {
                left,
                right,
                output_index,
            });
        self.output_size += 1;
        output_index
    }

    pub fn execute_with_callback(mut self, mut callback: impl FnMut(usize, usize)) {
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
            for _ in 0..100 {
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
