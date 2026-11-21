use super::Monoid;

/// A monoid with a lane-wise implementation that can be shared by SIMD trees.
///
/// Implement this trait to use the operation directly in a d-ary segment tree.
/// Commutative operations can also be used in a SIMD binary indexed tree.
pub trait SimdMonoid: Monoid<T: Copy> {
    /// Replaces each left operand with its ordered product with the right operand.
    ///
    /// # Safety
    /// The slices have equal lengths. ISA 0 requests portable execution, ISA 1
    /// guarantees x86-64 AVX2, and ISA 2 guarantees x86-64 AVX-512F/DQ/CD/BW/VL.
    /// Implementations support unaligned slices of any length, including tails,
    /// and check any additional CPU requirements themselves.
    unsafe fn operate_lanes<const ISA: u8>(lhs: &mut [Self::T], rhs: &[Self::T]);

    /// Appends the same right operand to every lane from `start` onward.
    ///
    /// # Safety
    /// The ISA and alignment contracts are the same as for [`Self::operate_lanes`].
    /// `start` must be at most `values.len()`.
    #[inline]
    unsafe fn broadcast_suffix<const ISA: u8>(values: &mut [Self::T], start: usize, rhs: Self::T) {
        let operands = [rhs; LANES];
        for chunk in values[start..].chunks_mut(LANES) {
            unsafe { Self::operate_lanes::<ISA>(chunk, &operands[..chunk.len()]) };
        }
    }

    /// Folds the slice in order, returning the unit for an empty slice.
    ///
    /// # Safety
    /// The ISA contract is the same as for [`Self::operate_lanes`].
    /// The slice has at most 16 values.
    #[inline]
    unsafe fn reduce_lanes<const ISA: u8>(values: &[Self::T]) -> Self::T {
        if ISA == 0 || values.len() <= 1 {
            return Self::fold(values.iter().copied());
        }

        let mut left = [Self::unit(); LANES / 2];
        let mut right = left;
        let (input, rest) = values.as_chunks::<2>();
        let mut pairs = input.len();
        for (i, pair) in input.iter().enumerate() {
            left[i] = pair[0];
            right[i] = pair[1];
        }
        unsafe { Self::operate_lanes::<ISA>(&mut left[..pairs], &right[..pairs]) };
        // Odd operands are suffixes of the preceding pairs.
        let mut suffix = rest.first().copied().unwrap_or_else(Self::unit);
        while pairs > 1 {
            if pairs & 1 != 0 {
                suffix = Self::operate(&left[pairs - 1], &suffix);
            }
            pairs /= 2;
            for i in 0..pairs {
                right[i] = left[2 * i + 1];
                left[i] = left[2 * i];
            }
            unsafe { Self::operate_lanes::<ISA>(&mut left[..pairs], &right[..pairs]) };
        }
        Self::operate(&left[0], &suffix)
    }
}

pub const LANES: usize = 16;

#[cfg(test)]
mod tests {
    use super::SimdMonoid;
    use crate::{
        algebra::{
            Associative, BitXorOperation, Commutative, Invertible, Magma, MaxOperation,
            MinOperation, Unital,
        },
        data_structure::{DaryPrefixGroup, DarySegmentTree, SimdAddI64, SimdBinaryIndexedTree},
        tools::Xorshift,
    };
    use std::{
        num::Wrapping,
        sync::atomic::{AtomicUsize, Ordering},
    };

    struct Xor;

    static PACKED_CALLS: AtomicUsize = AtomicUsize::new(0);

    impl Magma for Xor {
        type T = u64;

        fn operate(left: &u64, right: &u64) -> u64 {
            left ^ right
        }
    }
    impl Associative for Xor {}
    impl Commutative for Xor {}
    impl Unital for Xor {
        fn unit() -> u64 {
            0
        }
    }
    impl Invertible for Xor {
        fn inverse(x: &u64) -> u64 {
            *x
        }
    }
    impl SimdMonoid for Xor {
        unsafe fn operate_lanes<const ISA: u8>(lhs: &mut [u64], rhs: &[u64]) {
            unsafe { BitXorOperation::<u64>::operate_lanes::<ISA>(lhs, rhs) };
            if ISA != 0 {
                PACKED_CALLS.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    #[test]
    fn test_shared_point_operations() {
        let packed = crate::tools::simd_backend() != crate::tools::SimdBackend::Scalar;
        let mut rng = Xorshift::default();
        for n in [0, 1, 15, 16, 17, 255, 256, 257, 1025] {
            let mut keys: Vec<_> = (0..n).map(|_| rng.rand64()).collect();
            let mut seg = DarySegmentTree::<Xor>::from_keys(keys.iter().copied());
            let mut bit = SimdBinaryIndexedTree::<Xor>::from_keys(keys.iter().copied());
            let mut extrema: Vec<_> = keys.iter().map(|&key| key as i32).collect();
            let mut maximum =
                SimdBinaryIndexedTree::<MaxOperation<i32>>::from_keys(extrema.iter().copied());
            let mut minimum =
                DarySegmentTree::<MinOperation<i32>>::from_keys(extrema.iter().copied());
            let mut sums: Vec<_> = keys.iter().map(|&x| x as i64).collect();
            let mut prefix = DarySegmentTree::<DaryPrefixGroup<SimdAddI64>>::from_keys(
                sums.iter().copied().map(Wrapping),
            );
            for step in 0..256 {
                let left = rng.rand((n + 1) as u64) as usize;
                let right = rng.rand((n + 1) as u64) as usize;
                let (l, r) = if step % 8 == 0 {
                    (0, n)
                } else {
                    (left.min(right), left.max(right))
                };
                let xor = keys[l..r].iter().fold(0, |acc, x| acc ^ x);
                let sum = sums[l..r].iter().fold(0_i64, |acc, &x| acc.wrapping_add(x));
                assert_eq!(seg.fold(l..r), xor);
                assert_eq!(seg.fold_by_prefix(l..r), xor);
                assert_eq!(bit.fold(l, r), xor);
                assert_eq!(
                    maximum.accumulate0(r),
                    extrema[..r].iter().copied().max().unwrap_or(i32::MIN)
                );
                assert_eq!(
                    minimum.fold(l..r),
                    extrema[l..r].iter().copied().min().unwrap_or(i32::MAX)
                );
                assert_eq!(prefix.fold(l..r), Wrapping(sum));
                assert_eq!(prefix.fold_by_prefix(l..r), Wrapping(sum));
                if n == 0 {
                    continue;
                }
                let i = if step % 8 <= 1 {
                    n - 1
                } else {
                    rng.rand(n as u64) as usize
                };
                let x = rng.rand64();
                if step % 2 == 0 {
                    keys[i] ^= x;
                    sums[i] = sums[i].wrapping_add(x as i64);
                    seg.update(i, x);
                    bit.update(i, x);
                    prefix.update(i, Wrapping(x as i64));
                } else {
                    keys[i] = x;
                    sums[i] = x as i64;
                    seg.set(i, x);
                    bit.set(i, x);
                    prefix.set(i, Wrapping(x as i64));
                }
                maximum.update(i, x as i32);
                extrema[i] = extrema[i].max(x as i32);
                minimum.set(i, extrema[i]);
                assert_eq!(bit.get(i), keys[i]);
                assert_eq!(seg.get(i), keys[i]);
                assert_eq!(prefix.get(i), Wrapping(sums[i]));
            }
        }

        let mut maximum = SimdBinaryIndexedTree::<MaxOperation<i32>>::from_keys(0..257);
        maximum.update(256, i32::MAX);
        assert_eq!(maximum.accumulate0(256), 255);
        assert_eq!(maximum.accumulate0(257), i32::MAX);

        if packed {
            // Check each public structure separately. Scalar results alone would not detect a bypass.
            let mut seg = DarySegmentTree::<Xor>::from_keys((0..17).map(|i| i as u64));
            let before = PACKED_CALLS.load(Ordering::Relaxed);
            seg.set(0, 100);
            assert!(PACKED_CALLS.load(Ordering::Relaxed) > before);
            let mut bit = SimdBinaryIndexedTree::<Xor>::from_keys((0..17).map(|i| i as u64));
            let before = PACKED_CALLS.load(Ordering::Relaxed);
            bit.update(0, 100);
            assert!(PACKED_CALLS.load(Ordering::Relaxed) > before);
        }
    }
}
