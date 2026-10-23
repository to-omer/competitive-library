use super::{Group, Monoid};
use std::fmt::{self, Debug, Formatter};

pub struct BinaryIndexedTree2D<M>
where
    M: Monoid,
{
    h: usize,
    w: usize,
    bit: Vec<M::T>,
}

impl<M> Clone for BinaryIndexedTree2D<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            h: self.h,
            w: self.w,
            bit: self.bit.clone(),
        }
    }
}

impl<M> Debug for BinaryIndexedTree2D<M>
where
    M: Monoid<T: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryIndexedTree2D")
            .field("h", &self.h)
            .field("w", &self.w)
            .field("bit", &self.bit)
            .finish()
    }
}

impl<M> BinaryIndexedTree2D<M>
where
    M: Monoid,
{
    #[inline]
    pub fn new(h: usize, w: usize) -> Self {
        let bit = vec![M::unit(); (h + 1) * (w + 1)];
        Self { h, w, bit }
    }
    #[inline]
    /// fold [0, i) x [0, j)
    pub fn accumulate0(&self, i: usize, j: usize) -> M::T {
        assert!(i <= self.h && j <= self.w);
        let mut res = M::unit();
        let mut a = i;
        let stride = self.w + 1;
        while a > 0 {
            let mut b = j;
            while b > 0 {
                // SAFETY: the method validates both prefix endpoints, and Fenwick ancestors
                // remain within the allocated `(h + 1) * (w + 1)` table.
                M::operate_assign(&mut res, unsafe { self.bit.get_unchecked(a * stride + b) });
                b -= b & (!b + 1);
            }
            a -= a & (!a + 1);
        }
        res
    }
    #[inline]
    /// fold [0, i] x [0, j]
    pub fn accumulate(&self, i: usize, j: usize) -> M::T {
        self.accumulate0(i + 1, j + 1)
    }
    #[inline]
    pub fn update(&mut self, i: usize, j: usize, x: M::T) {
        assert!(i < self.h && j < self.w);
        let mut a = i + 1;
        let stride = self.w + 1;
        while a <= self.h {
            let mut b = j + 1;
            while b <= self.w {
                // SAFETY: the method validates the leaf, and Fenwick ancestors stay within the
                // allocated table.
                M::operate_assign(unsafe { self.bit.get_unchecked_mut(a * stride + b) }, &x);
                b += b & (!b + 1);
            }
            a += a & (!a + 1);
        }
    }
}

impl<G> BinaryIndexedTree2D<G>
where
    G: Group,
{
    #[inline]
    /// 0-indexed [i1, i2) x [j1, j2)
    pub fn fold(&self, i1: usize, j1: usize, i2: usize, j2: usize) -> G::T {
        let mut res = self.accumulate0(i1, j1);
        G::rinv_operate_assign(&mut res, &self.accumulate0(i1, j2));
        G::rinv_operate_assign(&mut res, &self.accumulate0(i2, j1));
        G::operate_assign(&mut res, &self.accumulate0(i2, j2));
        res
    }
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> G::T {
        self.fold(i, j, i + 1, j + 1)
    }
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, x: G::T) {
        let y = G::inverse(&self.get(i, j));
        let z = G::operate(&y, &x);
        self.update(i, j, z);
    }
}

#[cfg(kani)]
mod verification {
    use super::*;
    use crate::algebra::{AdditiveOperation, BitXorOperation};
    use std::num::Wrapping;

    #[kani::proof]
    #[kani::unwind(4)]
    fn proof_rectangle_update() {
        let mut values: [[u64; 3]; 2] = kani::any();
        let row: usize = kani::any();
        let column: usize = kani::any();
        let change: u64 = kani::any();
        let replace: bool = kani::any();
        let top: usize = kani::any();
        let bottom: usize = kani::any();
        let left: usize = kani::any();
        let right: usize = kani::any();
        kani::assume(row < 2 && column < 3);
        kani::assume(top <= bottom && bottom <= 2 && left <= right && right <= 3);
        let mut bit = BinaryIndexedTree2D::<BitXorOperation<u64>>::new(2, 3);
        for (i, values) in values.iter().enumerate() {
            for (j, &value) in values.iter().enumerate() {
                bit.update(i, j, value);
            }
        }
        if replace {
            bit.set(row, column, change);
            values[row][column] = change;
        } else {
            bit.update(row, column, change);
            values[row][column] ^= change;
        }
        let mut expected = 0;
        for values in &values[top..bottom] {
            for &value in &values[left..right] {
                expected ^= value;
            }
        }
        kani::assert(
            bit.get(row, column) == values[row][column]
                && bit.fold(top, left, bottom, right) == expected,
            "For all 2-by-3 u64 grids, point XOR updates or assignments and valid rectangles, 2D Fenwick queries equal the logical grid XOR; unchecked accesses stay within the allocated table",
        );
        kani::cover!(top == bottom || left == right);
        kani::cover!(top == 0 && bottom == 2 && left == 0 && right == 3 && column == 2);

        // A single occupied cell distinguishes inverse operations that XOR cannot check.
        let initial = Wrapping(kani::any());
        let change = Wrapping(kani::any());
        let mut bit = BinaryIndexedTree2D::<AdditiveOperation<Wrapping<u8>>>::new(2, 3);
        bit.update(row, column, initial);
        let value = if replace {
            bit.set(row, column, change);
            change
        } else {
            bit.update(row, column, change);
            initial + change
        };
        let contains = top <= row && row < bottom && left <= column && column < right;
        kani::assert(
            bit.get(row, column) == value
                && bit.fold(top, left, bottom, right) == if contains { value } else { Wrapping(0) },
            "For every single-cell position in a 2-by-3 grid and all u8 initial and replacement or increment values, addition modulo 256 preserves the cell value and returns it exactly for rectangles containing the cell",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, MaxOperation},
        tools::Xorshift,
    };

    const A: u64 = 1_000_000_000;
    const B: i64 = 1_000_000_000;

    #[test]
    fn test_binary_indexed_tree_2d() {
        let mut rng = Xorshift::default();
        for _ in 0..16 {
            let h = rng.rand(80) as usize + 1;
            let w = rng.rand(80) as usize + 1;
            let q = rng.rand(4_000) as usize + 1_000;
            let mut bit = BinaryIndexedTree2D::<AdditiveOperation<_>>::new(h, w);
            let mut arr = vec![vec![0; w]; h];
            for (i, j, v) in rng.random_iter((..h, ..w, ..A)).take(q) {
                bit.update(i, j, v);
                arr[i][j] += v;
            }
            for arr in arr.iter_mut() {
                for j in 0..w - 1 {
                    arr[j + 1] += arr[j];
                }
            }
            for i in 0..h - 1 {
                let [a, b] = arr.get_disjoint_mut([i + 1, i]).unwrap();
                for (a, b) in a.iter_mut().zip(b) {
                    *a += *b;
                }
            }
            for (i, arr) in arr.iter().enumerate() {
                for (j, a) in arr.iter().cloned().enumerate() {
                    assert_eq!(bit.accumulate(i, j), a);
                }
            }

            let mut bit = BinaryIndexedTree2D::<MaxOperation<_>>::new(h, w);
            let mut arr = vec![vec![0; w]; h];
            for (i, j, v) in rng.random_iter((..h, ..w, ..A)).take(q) {
                bit.update(i, j, v);
                arr[i][j] = std::cmp::max(arr[i][j], v);
            }
            for arr in arr.iter_mut() {
                for j in 0..w - 1 {
                    arr[j + 1] = std::cmp::max(arr[j + 1], arr[j]);
                }
            }
            for i in 0..h - 1 {
                let [a, b] = arr.get_disjoint_mut([i + 1, i]).unwrap();
                for (a, b) in a.iter_mut().zip(b) {
                    *a = std::cmp::max(*a, *b);
                }
            }
            for (i, arr) in arr.iter().enumerate() {
                for (j, a) in arr.iter().cloned().enumerate() {
                    assert_eq!(bit.accumulate(i, j), a);
                }
            }
        }
    }

    #[test]
    fn test_group_binary_indexed_tree2d() {
        let mut rng = Xorshift::default();
        for _ in 0..32 {
            let h = rng.rand(32) as usize;
            let w = rng.rand(32) as usize;
            let mut bit = BinaryIndexedTree2D::<AdditiveOperation<i64>>::new(h, w);
            let mut values = vec![vec![0; w]; h];
            for _ in 0..500 {
                if h != 0 && w != 0 {
                    let i = rng.rand(h as u64) as usize;
                    let j = rng.rand(w as u64) as usize;
                    let value = rng.rand(2 * B as u64) as i64 - B;
                    if rng.rand(2) == 0 {
                        bit.update(i, j, value);
                        values[i][j] += value;
                    } else {
                        bit.set(i, j, value);
                        values[i][j] = value;
                    }
                }
                let i1 = rng.rand(h as u64 + 1) as usize;
                let i2 = i1 + rng.rand((h - i1) as u64 + 1) as usize;
                let j1 = rng.rand(w as u64 + 1) as usize;
                let j2 = j1 + rng.rand((w - j1) as u64 + 1) as usize;
                assert_eq!(
                    bit.fold(i1, j1, i2, j2),
                    values[i1..i2].iter().flat_map(|row| &row[j1..j2]).sum()
                );
            }
        }
    }
}
