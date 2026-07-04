use super::{DisjointSparseTable, SemiGroup};
use std::fmt::{self, Debug, Formatter};

const DIRECT_SIZE: usize = 4;
const BLOCK_SCALE: usize = 4;

struct BlockProducts<T> {
    prefix: Vec<T>,
    suffix: Vec<T>,
    products: Vec<T>,
}

#[derive(Clone)]
pub struct StaticRangeProduct<S>
where
    S: SemiGroup,
{
    data: Vec<S::T>,
    block_shift: usize,
    prefix: Vec<S::T>,
    suffix: Vec<S::T>,
    between: Option<FixedRangeProduct<S>>,
}

impl<S> Debug for StaticRangeProduct<S>
where
    S: SemiGroup<T: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("StaticRangeProduct")
            .field("data", &self.data)
            .field("block_size", &(1usize << self.block_shift))
            .field("prefix", &self.prefix)
            .field("suffix", &self.suffix)
            .field("between", &self.between)
            .finish()
    }
}

impl<S> StaticRangeProduct<S>
where
    S: SemiGroup,
{
    pub fn new(data: Vec<S::T>) -> Self {
        let n = data.len();
        if n == 0 {
            return Self {
                data,
                block_shift: 0,
                prefix: Vec::new(),
                suffix: Vec::new(),
                between: None,
            };
        }
        let block_shift = scaled_block_shift(inverse_ackermann(n));
        let block_size = 1usize << block_shift;
        let blocks = block_products::<S>(&data, block_size);
        let between = if blocks.products.len() > 2 {
            let level = inverse_ackermann(blocks.products.len()).max(1);
            Some(FixedRangeProduct::new(blocks.products, level))
        } else {
            None
        };
        Self {
            data,
            block_shift,
            prefix: blocks.prefix,
            suffix: blocks.suffix,
            between,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub fn get(&self, index: usize) -> &S::T {
        &self.data[index]
    }

    #[inline]
    pub fn fold(&self, l: usize, r: usize) -> S::T {
        assert!(l < r);
        assert!(r <= self.data.len());
        let bl = l >> self.block_shift;
        let br = (r - 1) >> self.block_shift;
        if bl == br {
            return fold_slice::<S>(&self.data, l, r);
        }
        let mut res = self.suffix[l].clone();
        if bl + 1 < br {
            let mid = self
                .between
                .as_ref()
                .expect("middle block product is not built")
                .fold(bl + 1, br);
            res = S::operate(&res, &mid);
        }
        S::operate(&res, &self.prefix[r - 1])
    }
}

#[derive(Clone)]
enum FixedRangeProduct<S>
where
    S: SemiGroup,
{
    Direct {
        data: Vec<S::T>,
    },
    Disjoint {
        table: DisjointSparseTable<S>,
    },
    Recursive {
        data: Vec<S::T>,
        block_shift: usize,
        prefix: Vec<S::T>,
        suffix: Vec<S::T>,
        between: Box<FixedRangeProduct<S>>,
    },
}

impl<S> Debug for FixedRangeProduct<S>
where
    S: SemiGroup<T: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Direct { data } => f.debug_struct("Direct").field("data", data).finish(),
            Self::Disjoint { table } => f.debug_struct("Disjoint").field("table", table).finish(),
            Self::Recursive {
                data,
                block_shift,
                prefix,
                suffix,
                between,
            } => f
                .debug_struct("Recursive")
                .field("data", data)
                .field("block_size", &(1usize << block_shift))
                .field("prefix", prefix)
                .field("suffix", suffix)
                .field("between", between)
                .finish(),
        }
    }
}

impl<S> FixedRangeProduct<S>
where
    S: SemiGroup,
{
    fn new(data: Vec<S::T>, level: usize) -> Self {
        let n = data.len();
        if n <= DIRECT_SIZE || level == 0 {
            return Self::Direct { data };
        }
        if level == 1 {
            return Self::Disjoint {
                table: DisjointSparseTable::new(data),
            };
        }
        let block_shift = scaled_block_shift(alpha_k(level - 1, n));
        let block_size = 1usize << block_shift;
        if block_size <= 1 || block_size >= n {
            return Self::Direct { data };
        }
        let blocks = block_products::<S>(&data, block_size);
        let between = Box::new(Self::new(blocks.products, level - 1));
        Self::Recursive {
            data,
            block_shift,
            prefix: blocks.prefix,
            suffix: blocks.suffix,
            between,
        }
    }

    #[inline]
    fn fold(&self, l: usize, r: usize) -> S::T {
        match self {
            Self::Direct { data } => fold_slice::<S>(data, l, r),
            Self::Disjoint { table } => table.fold(l, r),
            Self::Recursive {
                data,
                block_shift,
                prefix,
                suffix,
                between,
            } => {
                let block_shift = *block_shift;
                let bl = l >> block_shift;
                let br = (r - 1) >> block_shift;
                if bl == br {
                    return fold_slice::<S>(data, l, r);
                }
                let mut res = suffix[l].clone();
                if bl + 1 < br {
                    let mid = between.fold(bl + 1, br);
                    res = S::operate(&res, &mid);
                }
                S::operate(&res, &prefix[r - 1])
            }
        }
    }
}

#[inline]
fn fold_slice<S>(data: &[S::T], l: usize, r: usize) -> S::T
where
    S: SemiGroup,
{
    let mut res = data[l].clone();
    for x in &data[l + 1..r] {
        res = S::operate(&res, x);
    }
    res
}

fn block_products<S>(data: &[S::T], block_size: usize) -> BlockProducts<S::T>
where
    S: SemiGroup,
{
    let n = data.len();
    let mut prefix = data.to_vec();
    let mut suffix = data.to_vec();
    let mut products = Vec::with_capacity(n.div_ceil(block_size));
    for start in (0..n).step_by(block_size) {
        let end = n.min(start + block_size);
        for i in start + 1..end {
            prefix[i] = S::operate(&prefix[i - 1], &data[i]);
        }
        for i in (start..end - 1).rev() {
            suffix[i] = S::operate(&data[i], &suffix[i + 1]);
        }
        products.push(prefix[end - 1].clone());
    }
    BlockProducts {
        prefix,
        suffix,
        products,
    }
}

fn alpha_k(k: usize, n: usize) -> usize {
    if k == 0 {
        return n.div_ceil(2);
    }
    if n <= 1 {
        return 0;
    }
    let mut x = n;
    let mut c = 0;
    while x > 1 {
        x = alpha_k(k - 1, x);
        c += 1;
    }
    c
}

fn inverse_ackermann(n: usize) -> usize {
    let mut k = 0;
    while alpha_k(k, n) > 3 {
        k += 1;
    }
    k
}

fn scaled_block_shift(alpha: usize) -> usize {
    (alpha.max(2) * BLOCK_SCALE)
        .next_power_of_two()
        .trailing_zeros() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, ConcatenateOperation, MinOperation},
        tools::Xorshift,
    };

    fn assert_all_ranges<S>(data: Vec<S::T>)
    where
        S: SemiGroup,
        S::T: Debug + PartialEq,
    {
        let table = StaticRangeProduct::<S>::new(data.clone());
        assert_eq!(table.len(), data.len());
        assert_eq!(table.is_empty(), data.is_empty());
        for (i, x) in data.iter().enumerate() {
            assert_eq!(table.get(i), x);
        }
        for l in 0..data.len() {
            let mut expected = data[l].clone();
            assert_eq!(table.fold(l, l + 1), expected);
            for r in l + 2..=data.len() {
                expected = S::operate(&expected, &data[r - 1]);
                assert_eq!(table.fold(l, r), expected);
            }
        }
    }

    #[test]
    fn test_static_range_product_randomized_exhaustive() {
        let mut rng = Xorshift::default();
        let mut sizes = vec![0];
        sizes.extend((0..96).map(|_| rng.random(0usize..=300)));
        for n in sizes {
            let data: Vec<i64> = (0..n).map(|_| rng.random(-1000..=1000)).collect();
            assert_all_ranges::<AdditiveOperation<i64>>(data.clone());
            assert_all_ranges::<MinOperation<i64>>(data);

            let n = n.min(80);
            let data: Vec<Vec<i32>> = (0..n).map(|_| vec![rng.random(-1000..=1000)]).collect();
            assert_all_ranges::<ConcatenateOperation<i32>>(data);
        }
    }
}
