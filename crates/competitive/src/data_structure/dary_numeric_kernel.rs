#![allow(unsafe_op_in_unsafe_fn)] // Tree dispatch checks the selected ISA.
#[cfg(target_arch = "x86_64")]
use super::simd;
use super::{DarySegmentGroup, DarySegmentKernel};
use std::ops::Range;

#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct DaryNumericBlock<T, const B: usize>([T; B]);

macro_rules! define_numeric_kernel {
    ($name:ident, $value:ty, $branch:expr, $unit:expr, $operation:ident,
     $sum:literal, $reduce_avx2:ident, $range_avx2:ident,
     $reduce_avx512:ident, $range_avx512:ident) => {
        #[derive(Clone, Debug)]
        pub struct $name;
        impl DarySegmentKernel for $name {
            const B: usize = $branch;
            type Key = $value;
            type Output = $value;
            type Value = $value;
            type Change = $value;
            type Block = DaryNumericBlock<$value, $branch>;
            fn encode(key: $value) -> $value {
                key
            }
            fn decode(value: $value) -> $value {
                value
            }
            fn unit() -> $value {
                $unit
            }
            fn combine(left: $value, right: $value) -> $value {
                left.$operation(right)
            }
            fn block(values: impl Iterator<Item = $value>) -> Self::Block {
                let mut block = [$unit; $branch];
                for (lane, value) in values.enumerate() {
                    block[lane] = value;
                }
                if $sum {
                    for i in 1..$branch {
                        block[i] = block[i].wrapping_add(block[i - 1]);
                    }
                }
                DaryNumericBlock(block)
            }
            fn get(block: &Self::Block, lane: usize) -> $value {
                if $sum && lane != 0 {
                    block.0[lane].wrapping_sub(block.0[lane - 1])
                } else {
                    block.0[lane]
                }
            }
            fn repair_value(value: $value, change: $value) -> $value {
                if $sum {
                    value.wrapping_add(change)
                } else {
                    change
                }
            }
            unsafe fn fold<const ISA: u8>(block: &Self::Block, range: Range<usize>) -> $value {
                if $sum {
                    let r = *block
                        .0
                        .get_unchecked(range.end.wrapping_sub(1) & ($branch - 1));
                    let l = *block
                        .0
                        .get_unchecked(range.start.wrapping_sub(1) & ($branch - 1));
                    return (r & (0 as $value).wrapping_sub((range.end != 0) as $value))
                        .wrapping_sub(
                            l & (0 as $value).wrapping_sub((range.start != 0) as $value),
                        );
                }
                #[cfg(target_arch = "x86_64")]
                if ISA == 1 {
                    return simd::$range_avx2(&*(block.0.as_ptr().cast()), range.start, range.end);
                }
                #[cfg(target_arch = "x86_64")]
                if ISA == 2 {
                    return simd::$range_avx512(
                        &*(block.0.as_ptr().cast()),
                        range.start,
                        range.end,
                    );
                }
                block.0[range]
                    .iter()
                    .copied()
                    .fold($unit, <$value>::$operation)
            }
            unsafe fn update<const ISA: u8>(
                block: &mut Self::Block,
                lane: usize,
                value: $value,
            ) -> Option<$value> {
                if $sum {
                    if value == 0 {
                        None
                    } else {
                        Self::repair::<ISA>(block, lane, value)
                    }
                } else {
                    Self::replace::<ISA>(block, lane, Self::combine(Self::get(block, lane), value))
                }
            }
            unsafe fn replace<const ISA: u8>(
                block: &mut Self::Block,
                lane: usize,
                value: $value,
            ) -> Option<$value> {
                let old = Self::get(block, lane);
                if old == value {
                    return None;
                }
                if $sum {
                    return Self::repair::<ISA>(block, lane, value.wrapping_sub(old));
                }
                block.0[lane] = value;
                #[cfg(target_arch = "x86_64")]
                if ISA == 1 {
                    return Some(simd::$reduce_avx2(&*(block.0.as_ptr().cast())));
                }
                #[cfg(target_arch = "x86_64")]
                if ISA == 2 {
                    return Some(simd::$reduce_avx512(&*(block.0.as_ptr().cast())));
                }
                Some(block.0.iter().copied().fold($unit, <$value>::$operation))
            }
            unsafe fn repair<const ISA: u8>(
                block: &mut Self::Block,
                lane: usize,
                change: $value,
            ) -> Option<$value> {
                if $sum {
                    #[cfg(target_arch = "x86_64")]
                    if ISA != 0 {
                        if std::mem::size_of::<$value>() == 4 {
                            let values = &mut *block.0.as_mut_ptr().cast::<[u32; 16]>();
                            if ISA == 1 {
                                simd::add_suffix_u32x16_avx2(values, lane, change as u32);
                            } else {
                                simd::add_suffix_u32x16_avx512(values, lane, change as u32);
                            }
                        } else {
                            let values = &mut *block.0.as_mut_ptr().cast::<[u64; 16]>();
                            if ISA == 1 {
                                simd::add_suffix_u64x16_avx2(values, lane, change as u64);
                            } else {
                                simd::add_suffix_u64x16_avx512(values, lane, change as u64);
                            }
                        }
                        return Some(change);
                    }
                    for x in &mut block.0[lane..] {
                        *x = x.wrapping_add(change);
                    }
                    Some(change)
                } else {
                    Self::replace::<ISA>(block, lane, change)
                }
            }
        }
    };
}
define_numeric_kernel!(
    DaryMinKernelI32,
    i32,
    16,
    i32::MAX,
    min,
    false,
    minimum_i32x16_avx2,
    minimum_range_i32x16_avx2,
    minimum_i32x16_avx512,
    minimum_range_i32x16_avx512
);
define_numeric_kernel!(
    DaryMaxKernelI32,
    i32,
    16,
    i32::MIN,
    max,
    false,
    maximum_i32x16_avx2,
    maximum_range_i32x16_avx2,
    maximum_i32x16_avx512,
    maximum_range_i32x16_avx512
);
define_numeric_kernel!(
    DaryMinKernelI64,
    i64,
    8,
    i64::MAX,
    min,
    false,
    minimum_i64x8_avx2,
    minimum_range_i64x8_avx2,
    minimum_i64x8_avx512,
    minimum_range_i64x8_avx512
);
define_numeric_kernel!(
    DaryMaxKernelI64,
    i64,
    8,
    i64::MIN,
    max,
    false,
    maximum_i64x8_avx2,
    maximum_range_i64x8_avx2,
    maximum_i64x8_avx512,
    maximum_range_i64x8_avx512
);
define_numeric_kernel!(
    DaryAddKernelI32,
    i32,
    16,
    0,
    wrapping_add,
    true,
    sum_i32x16_avx2,
    sum_range_i32x16_avx2,
    sum_i32x16_avx512,
    sum_range_i32x16_avx512
);
define_numeric_kernel!(
    DaryAddKernelI64,
    i64,
    16,
    0,
    wrapping_add,
    true,
    sum_i64x8_avx2,
    sum_range_i64x8_avx2,
    sum_i64x8_avx512,
    sum_range_i64x8_avx512
);

impl DarySegmentGroup for DaryAddKernelI32 {
    fn inverse(value: i32) -> i32 {
        value.wrapping_neg()
    }
}
impl DarySegmentGroup for DaryAddKernelI64 {
    fn inverse(value: i64) -> i64 {
        value.wrapping_neg()
    }
}
