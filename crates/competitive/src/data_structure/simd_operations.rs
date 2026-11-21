#[cfg(target_arch = "x86_64")]
use super::simd;
use super::{AdditiveOperation, BitXorOperation, Magma, MaxOperation, MinOperation, SimdMonoid};

#[codesnip::entry("_SimdLaneOps", include("algebra"))]
pub mod simd_lane_ops {
    use super::Magma;

    #[inline]
    pub unsafe fn operate<M, const ISA: u8>(lhs: &mut [M::T], rhs: &[M::T])
    where
        M: Magma,
    {
        #[cfg(target_arch = "x86_64")]
        match ISA {
            1 => return unsafe { operate_avx2::<M>(lhs, rhs) },
            2 => return unsafe { operate_avx512::<M>(lhs, rhs) },
            _ => {}
        }
        operate_scalar::<M>(lhs, rhs);
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn operate_avx2<M>(lhs: &mut [M::T], rhs: &[M::T])
    where
        M: Magma,
    {
        operate_scalar::<M>(lhs, rhs);
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f")]
    unsafe fn operate_avx512<M>(lhs: &mut [M::T], rhs: &[M::T])
    where
        M: Magma,
    {
        operate_scalar::<M>(lhs, rhs);
    }

    #[inline(always)] // Vectorize the loop with the caller's target features.
    fn operate_scalar<M>(lhs: &mut [M::T], rhs: &[M::T])
    where
        M: Magma,
    {
        for (left, right) in lhs.iter_mut().zip(rhs) {
            *left = M::operate(left, right);
        }
    }
}

#[codesnip::entry("SimdAddI64")]
pub use self::simd_add_i64::SimdAddI64;

#[codesnip::entry("SimdAddI64", include("SimdMonoid", "AdditiveOperation", "_simd"))]
mod simd_add_i64 {
    #[cfg(target_arch = "x86_64")]
    use super::simd;
    use super::{AdditiveOperation, Magma, SimdMonoid};
    use std::num::Wrapping;

    /// Wrapping 64-bit addition shared by SIMD trees.
    pub type SimdAddI64 = AdditiveOperation<Wrapping<i64>>;

    impl SimdMonoid for SimdAddI64 {
        #[inline]
        unsafe fn operate_lanes<const ISA: u8>(lhs: &mut [Wrapping<i64>], rhs: &[Wrapping<i64>]) {
            #[cfg(target_arch = "x86_64")]
            let (lhs, rhs) = unsafe {
                match ISA {
                    1 => {
                        let len = lhs.len() / 4 * 4;
                        let (head, rest) = lhs.split_at_mut(len);
                        add_avx2(head, &rhs[..len]);
                        (rest, &rhs[len..])
                    }
                    2 => {
                        let len = lhs.len() / 8 * 8;
                        let (head, rest) = lhs.split_at_mut(len);
                        add_avx512(head, &rhs[..len]);
                        (rest, &rhs[len..])
                    }
                    _ => (lhs, rhs),
                }
            };
            for (x, y) in lhs.iter_mut().zip(rhs) {
                *x = Self::operate(x, y);
            }
        }

        #[inline]
        unsafe fn broadcast_suffix<const ISA: u8>(
            values: &mut [Wrapping<i64>],
            start: usize,
            rhs: Wrapping<i64>,
        ) {
            if let Ok(block) = <&mut [Wrapping<i64>; 16]>::try_from(&mut *values) {
                #[cfg(target_arch = "x86_64")]
                if ISA != 0 {
                    // SAFETY: Wrapping<i64> and u64 have the same layout and all
                    // bit patterns are valid. The kernels allow start == 16.
                    let block = unsafe { &mut *block.as_mut_ptr().cast::<[u64; 16]>() };
                    if ISA == 1 {
                        unsafe { simd::add_suffix_u64x16_avx2(block, start, rhs.0 as u64) };
                    } else {
                        unsafe { simd::add_suffix_u64x16_avx512(block, start, rhs.0 as u64) };
                    }
                    return;
                }

                for (lane, value) in block.iter_mut().enumerate() {
                    let mask = -(i64::from(lane >= start));
                    *value += Wrapping(rhs.0 & mask);
                }
                return;
            }
            for value in &mut values[start..] {
                *value += rhs;
            }
        }

        #[inline]
        unsafe fn reduce_lanes<const ISA: u8>(values: &[Wrapping<i64>]) -> Wrapping<i64> {
            #[cfg(target_arch = "x86_64")]
            let (sum, values) = if ISA != 0 {
                let (chunks, rest) = values.as_chunks::<8>();
                let mut sum = 0_i64;
                for chunk in chunks {
                    // SAFETY: Wrapping<i64> is transparent and the kernels use unaligned loads.
                    let chunk = unsafe { &*chunk.as_ptr().cast::<[i64; 8]>() };
                    let partial = if ISA == 1 {
                        unsafe { simd::sum_i64x8_avx2(chunk) }
                    } else {
                        unsafe { simd::sum_i64x8_avx512(chunk) }
                    };
                    sum = sum.wrapping_add(partial);
                }
                (Wrapping(sum), rest)
            } else {
                (Wrapping(0), values)
            };
            #[cfg(not(target_arch = "x86_64"))]
            let sum = Wrapping(0);
            values.iter().copied().fold(sum, |acc, x| acc + x)
        }
    }

    #[cfg(target_arch = "x86_64")]
    macro_rules! define_add {
        ($name:ident, $feature:literal, $lanes:literal, $load:ident, $store:ident, $add:ident) => {
            #[target_feature(enable = $feature)]
            unsafe fn $name(lhs: &mut [Wrapping<i64>], rhs: &[Wrapping<i64>]) {
                use std::arch::x86_64::*;
                for (left, right) in lhs.chunks_exact_mut($lanes).zip(rhs.chunks_exact($lanes)) {
                    // SAFETY: each slice has one full vector, and loads/stores allow unaligned addresses.
                    unsafe {
                        let p = left.as_mut_ptr().cast();
                        $store(p, $add($load(p), $load(right.as_ptr().cast())));
                    }
                }
            }
        };
    }

    #[cfg(target_arch = "x86_64")]
    define_add!(
        add_avx2,
        "avx2",
        4,
        _mm256_loadu_si256,
        _mm256_storeu_si256,
        _mm256_add_epi64
    );
    #[cfg(target_arch = "x86_64")]
    define_add!(
        add_avx512,
        "avx512f",
        8,
        _mm512_loadu_si512,
        _mm512_storeu_si512,
        _mm512_add_epi64
    );

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_shared_add_lanes() {
            fn check<const ISA: u8>() {
                #[repr(align(64))]
                struct Aligned([Wrapping<i64>; 34]);

                let input = std::array::from_fn(|i| Wrapping(i64::MAX - i as i64));
                let rhs = Wrapping(31);
                for len in 0..=32 {
                    let mut values = Aligned(input);
                    let operands = Aligned(std::array::from_fn(|i| Wrapping(31 + i as i64)));
                    unsafe {
                        SimdAddI64::operate_lanes::<ISA>(
                            &mut values.0[1..1 + len],
                            &operands.0[2..2 + len],
                        )
                    };
                    let mut expected = input;
                    for (value, operand) in
                        expected[1..1 + len].iter_mut().zip(&operands.0[2..2 + len])
                    {
                        value.0 = value.0.wrapping_add(operand.0);
                    }
                    assert_eq!(values.0, expected, "ISA {ISA}, len {len}");

                    for start in 0..=len {
                        let mut values = Aligned(input);
                        // Offset by one operand to avoid SIMD alignment.
                        unsafe {
                            SimdAddI64::broadcast_suffix::<ISA>(
                                &mut values.0[1..1 + len],
                                start,
                                rhs,
                            )
                        };
                        let mut expected = input;
                        for value in &mut expected[1 + start..1 + len] {
                            value.0 = value.0.wrapping_add(rhs.0);
                        }
                        assert_eq!(values.0, expected, "ISA {ISA}, len {len}, start {start}");
                    }

                    if len <= 16 {
                        let values = Aligned(input);
                        let slice = &values.0[1..1 + len];
                        let expected = slice.iter().fold(0_i64, |sum, x| sum.wrapping_add(x.0));
                        assert_eq!(
                            unsafe { SimdAddI64::reduce_lanes::<ISA>(slice) }.0,
                            expected
                        );
                    }
                }
            }

            check::<0>();
            #[cfg(target_arch = "x86_64")]
            {
                if is_x86_feature_detected!("avx2") {
                    check::<1>();
                }
                if crate::tools::avx512_supported() {
                    check::<2>();
                }
            }
        }
    }
}

#[codesnip::entry(
    "_SimdXorU64",
    when("SimdMonoid", "BitXorOperation"),
    include("_SimdLaneOps")
)]
mod simd_xor_u64 {
    use super::{BitXorOperation, SimdMonoid, simd_lane_ops};

    impl SimdMonoid for BitXorOperation<u64> {
        #[inline]
        unsafe fn operate_lanes<const ISA: u8>(lhs: &mut [u64], rhs: &[u64]) {
            unsafe { simd_lane_ops::operate::<Self, ISA>(lhs, rhs) };
        }
    }
}

#[codesnip::entry(
    "_SimdMinI32",
    when("SimdMonoid", "MinOperation"),
    include("_SimdLaneOps")
)]
mod simd_min_i32 {
    use super::{MinOperation, SimdMonoid, simd_lane_ops};

    impl SimdMonoid for MinOperation<i32> {
        #[inline]
        unsafe fn operate_lanes<const ISA: u8>(lhs: &mut [i32], rhs: &[i32]) {
            unsafe { simd_lane_ops::operate::<Self, ISA>(lhs, rhs) };
        }
    }
}

#[codesnip::entry(
    "_SimdMaxI32",
    when("SimdMonoid", "MaxOperation"),
    include("_SimdLaneOps")
)]
mod simd_max_i32 {
    use super::{MaxOperation, SimdMonoid, simd_lane_ops};

    impl SimdMonoid for MaxOperation<i32> {
        #[inline]
        unsafe fn operate_lanes<const ISA: u8>(lhs: &mut [i32], rhs: &[i32]) {
            unsafe { simd_lane_ops::operate::<Self, ISA>(lhs, rhs) };
        }
    }
}
