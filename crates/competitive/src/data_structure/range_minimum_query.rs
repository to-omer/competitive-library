const BLOCK_SIZE: usize = 64;
const BLOCK_SHIFT: usize = BLOCK_SIZE.trailing_zeros() as usize;
const BLOCK_MASK: usize = BLOCK_SIZE - 1;

#[derive(Clone, Debug)]
pub struct RangeMinimumQuery<T> {
    data: Vec<T>,
    mask: Vec<u64>,
    sparse: Vec<usize>,
    sparse_offset: Vec<usize>,
}

impl<T> RangeMinimumQuery<T>
where
    T: Ord,
{
    pub fn new(data: Vec<T>) -> Self {
        let n = data.len();
        let blocks = (n + BLOCK_MASK) >> BLOCK_SHIFT;
        let mut mask = vec![0u64; n];
        let mut block_min = vec![0usize; blocks];

        let mut stack: Vec<usize> = Vec::with_capacity(BLOCK_SIZE);
        for (block, min_slot) in block_min.iter_mut().enumerate() {
            stack.clear();
            let start = block << BLOCK_SHIFT;
            let end = (start + BLOCK_SIZE).min(n);
            for i in start..end {
                while let Some(&top) = stack.last() {
                    if data[top] < data[i] {
                        break;
                    }
                    stack.pop();
                }
                let prev_mask = stack.last().map_or(0, |&top| mask[top]);
                mask[i] = prev_mask | 1u64 << (i - start);
                stack.push(i);
            }
            *min_slot = (start..end).min_by_key(|&i| &data[i]).unwrap();
        }

        let mut total = 0usize;
        let mut k = 0usize;
        while (1usize << k) <= blocks {
            total += blocks - (1usize << k) + 1;
            k += 1;
        }
        let mut sparse = Vec::with_capacity(total);
        let mut sparse_offset = Vec::with_capacity(k);
        sparse_offset.push(0);
        sparse.extend_from_slice(&block_min);
        let mut k = 1usize;
        let mut prev_offset = 0usize;
        while (1usize << k) <= blocks {
            let len = blocks - (1usize << k) + 1;
            let offset = sparse.len();
            sparse_offset.push(offset);
            let jump = 1usize << (k - 1);
            for i in 0..len {
                let left = sparse[prev_offset + i];
                let right = sparse[prev_offset + i + jump];
                sparse.push(if data[left] < data[right] {
                    left
                } else {
                    right
                });
            }
            prev_offset = offset;
            k += 1;
        }

        Self {
            data,
            mask,
            sparse,
            sparse_offset,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: usize) -> &T {
        &self.data[index]
    }

    fn min_index(&self, a: usize, b: usize) -> usize {
        if self.data[a] < self.data[b] { a } else { b }
    }

    fn argmin_in_block(&self, l: usize, r: usize) -> usize {
        debug_assert!(l <= r);
        debug_assert!(l >> BLOCK_SHIFT == r >> BLOCK_SHIFT);
        let start = l & !BLOCK_MASK;
        let shift = l & BLOCK_MASK;
        let masked = self.mask[r] & (!0u64 << shift);
        let offset = masked.trailing_zeros() as usize;
        start + offset
    }

    fn argmin_blocks(&self, l_block: usize, r_block: usize) -> usize {
        debug_assert!(l_block < r_block);
        let len = r_block - l_block;
        let k = len.ilog2() as usize;
        let offset = self.sparse_offset[k];
        let left = self.sparse[offset + l_block];
        let right = self.sparse[offset + r_block - (1usize << k)];
        self.min_index(left, right)
    }

    pub fn argmin(&self, l: usize, r: usize) -> usize {
        assert!(l < r);
        assert!(r <= self.data.len());
        let r = r - 1;
        let bl = l >> BLOCK_SHIFT;
        let br = r >> BLOCK_SHIFT;
        if bl == br {
            return self.argmin_in_block(l, r);
        }
        let left_end = ((bl + 1) << BLOCK_SHIFT) - 1;
        let right_start = br << BLOCK_SHIFT;
        let mut best = self.min_index(
            self.argmin_in_block(l, left_end),
            self.argmin_in_block(right_start, r),
        );
        if bl + 1 < br {
            let mid = self.argmin_blocks(bl + 1, br);
            best = self.min_index(best, mid);
        }
        best
    }
}

impl<T> RangeMinimumQuery<T>
where
    T: Ord + Clone,
{
    pub fn fold(&self, l: usize, r: usize) -> T {
        let idx = self.argmin(l, r);
        self.data[idx].clone()
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
                let idx = rmq.argmin(l, r);
                assert!(l <= idx && idx < r);
                assert_eq!(rmq.get(idx), &expected);
            }
        }
    }
}
