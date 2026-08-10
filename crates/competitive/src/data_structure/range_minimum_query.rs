const BLOCK_SIZE: usize = 64;

#[derive(Clone, Debug)]
pub struct RangeMinimumQuery<T> {
    data: Vec<T>,
    suffix: Vec<T>,
    prefix: Vec<T>,
    table: Vec<T>,
    blocks: usize,
}

impl<T> RangeMinimumQuery<T>
where
    T: Ord + Copy,
{
    pub fn new(data: Vec<T>) -> Self {
        let n = data.len();
        let blocks = n.div_ceil(BLOCK_SIZE);
        if blocks == 0 {
            return Self {
                data,
                suffix: vec![],
                prefix: vec![],
                table: vec![],
                blocks,
            };
        }
        let levels = usize::BITS as usize - blocks.leading_zeros() as usize;
        let mut table = vec![data[0]; levels * blocks];
        let mut prefix = Vec::with_capacity(n);
        let mut minimum = data[0];
        for (i, &value) in data.iter().enumerate() {
            minimum = if i % BLOCK_SIZE == 0 {
                value
            } else {
                minimum.min(value)
            };
            prefix.push(minimum);
            if i % BLOCK_SIZE == BLOCK_SIZE - 1 || i + 1 == n {
                table[i / BLOCK_SIZE] = minimum;
            }
        }
        let mut suffix = data.clone();
        for block in suffix.chunks_mut(BLOCK_SIZE) {
            minimum = *block.last().unwrap();
            for value in block.iter_mut().rev() {
                minimum = minimum.min(*value);
                *value = minimum;
            }
        }
        for level in 1..levels {
            let current = level * blocks;
            let previous = current - blocks;
            let half = 1 << (level - 1);
            for i in 0..blocks - (1 << level) + 1 {
                table[current + i] = if table[previous + i] < table[previous + i + half] {
                    table[previous + i]
                } else {
                    table[previous + i + half]
                };
            }
        }

        Self {
            data,
            suffix,
            prefix,
            table,
            blocks,
        }
    }

    #[inline]
    pub fn fold(&self, l: usize, r: usize) -> T {
        let r = r - 1;
        let left_block = l / BLOCK_SIZE;
        let right_block = r / BLOCK_SIZE;
        if left_block + 1 < right_block {
            let middle_blocks = right_block - left_block - 1;
            let level = middle_blocks.ilog2() as usize;
            let offset = level * self.blocks;
            self.suffix[l]
                .min(self.prefix[r])
                .min(self.table[offset + left_block + 1])
                .min(self.table[offset + right_block - (1 << level)])
        } else if left_block < right_block {
            self.suffix[l].min(self.prefix[r])
        } else {
            *self.data[l..=r].iter().min().unwrap()
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
    fn test_range_minimum_query() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..200, arr: [-1000i64..=1000; n]);
            let rmq = RangeMinimumQuery::new(arr.clone());
            for _ in 0..200 {
                rand!(rng, (l, r): Nes(n));
                let expected = arr[l..r].iter().min().cloned().unwrap();
                assert_eq!(rmq.fold(l, r), expected);
            }
        }
    }
}
