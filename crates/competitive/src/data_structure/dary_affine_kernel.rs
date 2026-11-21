use super::{
    DaryDualKernel, DaryLazyKernel, DarySegmentGroup, DarySegmentKernel, MIntBase, MIntConvert,
    mint_basic, montgomery,
};
#[cfg(target_arch = "x86_64")]
use super::{SimdBackend, montgomery_simd, simd_backend};

#[codesnip::entry("_DaryAffineArithmetic", include("MInt", "montgomery"))]
pub mod dary_affine_arithmetic {
    pub use super::{
        mint_basic::MInt998244353 as M,
        montgomery::{Modulo998244353 as Mod, MontgomeryReduction32},
    };
    pub use std::ops::Range;

    pub const B: usize = 16;
    pub const ONE: u32 = Mod::N1;
    pub const REV: [usize; 16] = [0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15];
    #[cfg(target_arch = "x86_64")]
    pub const PREFIX_MASK: [u16; 17] = {
        let mut a = [0; 17];
        let mut i = 0;
        while i < 16 {
            a[i + 1] = a[i] | (1 << REV[i]);
            i += 1;
        }
        a
    };
    pub type Affine = (u32, u32);

    // Internal residues are below 2p. Since 4p < 2^32, addition and Montgomery
    // multiplication stay in that interval without a final subtraction of p.
    #[inline]
    pub fn mod_add(x: u32, y: u32) -> u32 {
        let z = x + y;
        if z >= 2 * Mod::MOD {
            z - 2 * Mod::MOD
        } else {
            z
        }
    }
    #[inline]
    pub fn mod_sub(x: u32, y: u32) -> u32 {
        if x < y { x + 2 * Mod::MOD - y } else { x - y }
    }
    #[inline]
    pub fn mod_mul(x: u32, y: u32) -> u32 {
        let z = x as u64 * y as u64;
        ((z + (z as u32).wrapping_mul(Mod::R) as u64 * Mod::MOD as u64) >> 32) as u32
    }

    #[inline]
    pub fn compose(left: Affine, right: Affine) -> Affine {
        (
            mod_mul(left.0, right.0),
            mod_add(mod_mul(left.1, right.0), right.1),
        )
    }
}

#[codesnip::entry("_DarySumLeaf")]
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct SumLeaf([u32; dary_affine_arithmetic::B]);

#[codesnip::entry("_DarySumLeaf", include("_DaryAffineArithmetic"))]
pub mod dary_sum_leaf {
    use super::{SumLeaf, dary_affine_arithmetic::*};
    pub fn leaf(keys: impl Iterator<Item = u32>) -> SumLeaf {
        let mut node = SumLeaf([0; B]);
        for (lane, value) in keys.enumerate() {
            node.0[lane] = value;
        }
        node
    }
}

#[codesnip::entry("_DarySumVector", include("_DarySumLeaf", "montgomery_simd"))]
#[allow(unsafe_op_in_unsafe_fn)] // The generic tree checks the selected ISA.
#[cfg(target_arch = "x86_64")]
pub mod dary_sum_vector {
    use super::{SumLeaf, dary_affine_arithmetic::*, montgomery_simd::*};
    use std::arch::x86_64::*;

    pub unsafe fn edit_affine<const ISA: u8>(
        values: &mut [u32; B],
        multipliers: Option<&mut [u32; B]>,
        len: usize,
        range: Range<usize>,
        action: Affine,
        carry: Affine,
    ) {
        if ISA == 1 {
            avx2::edit_affine(values, multipliers, len, range, action, carry);
        } else {
            avx512::edit_affine(values, multipliers, len, range, action, carry);
        }
    }
    #[inline]
    pub unsafe fn sum<const ISA: u8>(values: &[u32; B], range: Range<usize>) -> u32 {
        if ISA == 1 {
            avx2::sum(values, range)
        } else {
            avx512::sum(values, range)
        }
    }

    pub unsafe fn edit_sum_leaf<const ISA: u8>(
        node: &mut SumLeaf,
        len: usize,
        range: Range<usize>,
        action: Affine,
        carry: Affine,
    ) -> u32 {
        if ISA == 1 {
            avx2::edit_sum(&mut node.0, None, len, 1, range, action, carry)
        } else {
            avx512::edit_sum(&mut node.0, None, len, 1, range, action, carry)
        }
    }

    macro_rules! define_vector {
        ($feature:literal, $lanes:expr, $steps:expr, $set:ident, $load:ident, $store:ident, $mul:ident, $add:ident, $sub:ident, $extract:expr) => {
            #[target_feature(enable=$feature)]
            pub unsafe fn edit_affine(
                values: &mut [u32; B],
                mut multipliers: Option<&mut [u32; B]>,
                len: usize,
                range: Range<usize>,
                action: Affine,
                carry: Affine,
            ) {
                let modulus = $set(Mod::MOD as i32);

                let combined = compose(carry, action);
                let carry_r = $set(carry.0.wrapping_mul(Mod::R) as i32);
                let combined_r = $set(combined.0.wrapping_mul(Mod::R) as i32);
                for base in (0..B).step_by($lanes) {
                    let selected = mask(range.start, range.end, base);
                    let valid = mask(0, len, base);
                    let a = blend(
                        valid,
                        $set(ONE as i32),
                        blend(selected, $set(carry.0 as i32), $set(combined.0 as i32)),
                    );
                    let a_r = blend(
                        valid,
                        $set(ONE.wrapping_mul(Mod::R) as i32),
                        blend(selected, carry_r, combined_r),
                    );
                    let b = blend(
                        valid,
                        $set(0),
                        blend(selected, $set(carry.1 as i32), $set(combined.1 as i32)),
                    );
                    let p = values.as_mut_ptr().add(base).cast();
                    $store(p, $add(mul_prepared($load(p), a, a_r, modulus), b, modulus));
                    if let Some(mul) = &mut multipliers {
                        let p = mul.as_mut_ptr().add(base).cast();
                        $store(p, mul_prepared($load(p), a, a_r, modulus));
                    }
                }
            }

            #[target_feature(enable = $feature)]
            pub unsafe fn sum(values: &[u32; B], range: Range<usize>) -> u32 {
                let modulus = $set(Mod::MOD as i32);
                let mut result = $set(0);
                for base in (0..B).step_by($lanes) {
                    let values = $load(values.as_ptr().add(base).cast());
                    result = $add(
                        result,
                        blend(mask(range.start, range.end, base), $set(0), values),
                        modulus,
                    );
                }
                reduce(result)
            }

            #[target_feature(enable = $feature)]
            pub unsafe fn edit_sum(
                values: &mut [u32; B],
                mut tags: Option<(&mut [u32; B], &mut [u32; B])>,
                len: usize,
                width: usize,
                full: Range<usize>,
                action: Affine,
                carry: Affine,
            ) -> u32 {
                let modulus = $set(Mod::MOD as i32);

                let combined = compose(carry, action);
                let ca_r = $set(carry.0.wrapping_mul(Mod::R) as i32);
                let fa_r = $set(combined.0.wrapping_mul(Mod::R) as i32);
                let ca = $set(carry.0 as i32);
                let cb = $set(carry.1 as i32);
                let fa = $set(combined.0 as i32);
                let fb = $set(combined.1 as i32);
                let count = ((len - 1) >> width.trailing_zeros()) + 1;
                let scaled = |value: u32, count: usize| {
                    ((value as u64 * (count as u64 % Mod::MOD as u64)) % Mod::MOD as u64) as i32
                };
                let old_offset = $set(scaled(carry.1, width));
                let new_offset = $set(scaled(combined.1, width));
                let tail = len - (count - 1) * width;
                let (old_tail, new_tail) = if tail == width {
                    (old_offset, new_offset)
                } else {
                    ($set(scaled(carry.1, tail)), $set(scaled(combined.1, tail)))
                };
                let mut selected_sum = $set(0);
                for base in (0..B).step_by($lanes) {
                    let selected = mask(full.start, full.end, base);
                    let a = blend(selected, ca, fa);
                    let a_r = blend(selected, ca_r, fa_r);
                    let b = blend(selected, cb, fb);
                    let offset = blend(selected, old_offset, new_offset);
                    let offset = if tail == width {
                        offset
                    } else {
                        blend(
                            mask(count - 1, count, base),
                            offset,
                            blend(selected, old_tail, new_tail),
                        )
                    };
                    let offset = if count == B {
                        offset
                    } else {
                        blend(mask(0, count, base), $set(0), offset)
                    };
                    let old = $load(values.as_ptr().add(base).cast());
                    selected_sum = $add(selected_sum, blend(selected, $set(0), old), modulus);
                    let new = $add(mul_prepared(old, a, a_r, modulus), offset, modulus);
                    $store(values.as_mut_ptr().add(base).cast(), new);
                    if let Some((mul, add)) = &mut tags {
                        let old_mul = $load(mul.as_ptr().add(base).cast());
                        let old_add = $load(add.as_ptr().add(base).cast());
                        $store(
                            mul.as_mut_ptr().add(base).cast(),
                            mul_prepared(old_mul, a, a_r, modulus),
                        );
                        $store(
                            add.as_mut_ptr().add(base).cast(),
                            $add(mul_prepared(old_add, a, a_r, modulus), b, modulus),
                        );
                    }
                }
                let previous = reduce(selected_sum);
                let covered = if full.is_empty() {
                    0
                } else {
                    if full.end == count {
                        len - full.start * width
                    } else {
                        (full.end - full.start) * width
                    }
                };
                mod_add(
                    mod_mul(previous, mod_sub(combined.0, carry.0)),
                    scaled(mod_sub(combined.1, carry.1), covered) as u32,
                )
            }
        };
    }

    pub mod avx2 {
        use super::*;
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn reduce(value: __m256i) -> u32 {
            let even = _mm256_and_si256(value, _mm256_set1_epi64x(u32::MAX as i64));
            let pairs = _mm256_add_epi64(even, _mm256_srli_epi64::<32>(value));
            let halves = _mm_add_epi64(
                _mm256_castsi256_si128(pairs),
                _mm256_extracti128_si256::<1>(pairs),
            );
            let sum = (_mm_cvtsi128_si64(halves) as u64) + (_mm_extract_epi64::<1>(halves) as u64);
            (sum % Mod::MOD as u64) as u32
        }

        #[target_feature(enable = "avx2")]
        #[inline]
        unsafe fn mul_prepared(a: __m256i, b: __m256i, _br: __m256i, p: __m256i) -> __m256i {
            montgomery_mul_256(a, b, _mm256_set1_epi32(Mod::R as i32), p)
        }

        #[target_feature(enable = "avx2")]
        #[inline]
        unsafe fn mask(start: usize, end: usize, base: usize) -> __m256i {
            let lanes = _mm256_add_epi32(
                _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7),
                _mm256_set1_epi32(base as i32),
            );
            _mm256_and_si256(
                _mm256_cmpgt_epi32(lanes, _mm256_set1_epi32(start as i32 - 1)),
                _mm256_cmpgt_epi32(_mm256_set1_epi32(end as i32), lanes),
            )
        }
        #[target_feature(enable = "avx2")]
        #[inline]
        unsafe fn blend(mask: __m256i, left: __m256i, right: __m256i) -> __m256i {
            _mm256_blendv_epi8(left, right, mask)
        }
        #[target_feature(enable = "avx2")]
        #[inline]
        unsafe fn add_redundant(a: __m256i, b: __m256i, p: __m256i) -> __m256i {
            add_mod_256(a, b, _mm256_add_epi32(p, p))
        }
        define_vector!(
            "avx2",
            8,
            [1, 2, 4],
            _mm256_set1_epi32,
            _mm256_loadu_si256,
            _mm256_storeu_si256,
            montgomery_mul_256,
            add_redundant,
            sub_mod_256,
            |x| _mm256_extract_epi32::<0>(x)
        );
    }

    pub mod avx512 {
        use super::*;
        #[inline]
        #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
        unsafe fn reduce(value: __m512i) -> u32 {
            let even = _mm512_and_si512(value, _mm512_set1_epi64(u32::MAX as i64));
            let pairs = _mm512_add_epi64(even, _mm512_srli_epi64::<32>(value));
            ((_mm512_reduce_add_epi64(pairs) as u64) % Mod::MOD as u64) as u32
        }

        #[target_feature(enable = "avx512f")]
        #[inline]
        unsafe fn mul_prepared(a: __m512i, b: __m512i, br: __m512i, p: __m512i) -> __m512i {
            let a13 = _mm512_srli_epi64::<32>(a);
            let b13 = _mm512_srli_epi64::<32>(b);
            let br13 = _mm512_srli_epi64::<32>(br);
            let t02 = _mm512_mul_epu32(a, b);
            let t13 = _mm512_mul_epu32(a13, b13);
            let m02 = mul_epu32_512(a, br);
            let m13 = mul_epu32_512(a13, br13);
            let u02 = _mm512_add_epi64(t02, _mm512_mul_epu32(m02, p));
            let u13 = _mm512_add_epi64(t13, _mm512_mul_epu32(m13, p));
            _mm512_or_si512(_mm512_srli_epi64::<32>(u02), u13)
        }

        #[target_feature(enable = "avx512f")]
        #[inline]
        unsafe fn mask(start: usize, end: usize, _base: usize) -> __mmask16 {
            (((1u32 << end) - 1) ^ ((1u32 << start) - 1)) as u16
        }
        #[target_feature(enable = "avx512f")]
        #[inline]
        unsafe fn blend(mask: __mmask16, left: __m512i, right: __m512i) -> __m512i {
            _mm512_mask_blend_epi32(mask, left, right)
        }
        #[target_feature(enable = "avx512f")]
        #[inline]
        unsafe fn add_redundant(a: __m512i, b: __m512i, p: __m512i) -> __m512i {
            add_mod_512(a, b, _mm512_add_epi32(p, p))
        }
        define_vector!(
            "avx512f,avx512dq,avx512cd,avx512bw,avx512vl",
            16,
            [1, 2, 4, 8],
            _mm512_set1_epi32,
            _mm512_loadu_si512,
            _mm512_storeu_si512,
            montgomery_mul_512,
            add_redundant,
            sub_mod_512,
            |x| _mm_cvtsi128_si32(_mm512_castsi512_si128(x))
        );
    }
}

#[codesnip::entry("_DaryCompositeLeaf")]
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct CompositeLeaf {
    mul: [u32; dary_affine_arithmetic::B],
    add: [u32; dary_affine_arithmetic::B],
}

#[codesnip::entry(
    "_DaryCompositeLeaf",
    include("_DaryAffineArithmetic", "montgomery_simd")
)]
#[allow(unsafe_op_in_unsafe_fn)] // The generic tree checks the selected ISA.
pub mod dary_composite_leaf {
    #[cfg(target_arch = "x86_64")]
    use super::bitrev_vector;
    use super::{CompositeLeaf, dary_affine_arithmetic::*};
    pub fn leaf(keys: impl Iterator<Item = Affine>) -> CompositeLeaf {
        let mut node = CompositeLeaf {
            mul: [ONE; B],
            add: [0; B],
        };
        for (lane, (mul, add)) in keys.enumerate() {
            node.mul[REV[lane]] = mul;
            node.add[REV[lane]] = add;
        }
        node
    }
    pub unsafe fn refold<const ISA: u8>(node: &CompositeLeaf) -> Affine {
        #[cfg(target_arch = "x86_64")]
        if ISA != 0 {
            return bitrev_vector::fold::<ISA, true>(node, 0..B);
        }
        let pair =
            |a: usize, b: usize| compose((node.mul[a], node.add[a]), (node.mul[b], node.add[b]));
        let quarter = |i: usize| {
            let [a, b, c, d] = REV.as_chunks::<4>().0[i];
            compose(pair(a, b), pair(c, d))
        };
        compose(
            compose(quarter(0), quarter(1)),
            compose(quarter(2), quarter(3)),
        )
    }

    pub unsafe fn fold<const ISA: u8>(node: &CompositeLeaf, range: Range<usize>) -> Affine {
        // Short fragments avoid the permutation cost of a vector reduction.
        #[cfg(target_arch = "x86_64")]
        if ISA != 0 && range.len() > 3 {
            return bitrev_vector::fold::<ISA, false>(node, range);
        }
        if range.is_empty() {
            return (ONE, 0);
        }
        let first = REV[range.start];
        (range.start + 1..range.end).fold((node.mul[first], node.add[first]), |value, lane| {
            compose(value, (node.mul[REV[lane]], node.add[REV[lane]]))
        })
    }
}

#[codesnip::entry("_DaryCompositeLeaf")]
#[allow(unsafe_op_in_unsafe_fn)] // The generic tree checks the selected ISA.
#[cfg(target_arch = "x86_64")]
pub mod bitrev_vector {
    use super::montgomery_simd::*;
    use super::{CompositeLeaf, dary_affine_arithmetic::*};
    use std::arch::x86_64::*;
    pub unsafe fn fold<const ISA: u8, const FULL: bool>(
        node: &CompositeLeaf,
        range: Range<usize>,
    ) -> Affine {
        if ISA == 1 || is_x86_feature_detected!("avx2") {
            avx2::<FULL>(node, range)
        } else {
            avx512::<FULL>(node, range)
        }
    }
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn even(x: __m256i, y: __m256i) -> __m256i {
        let t = _mm256_mul_epu32(x, y);
        let m = _mm256_mul_epu32(t, _mm256_set1_epi32(Mod::R as i32));
        _mm256_srli_epi64::<32>(_mm256_add_epi64(
            t,
            _mm256_mul_epu32(m, _mm256_set1_epi32(Mod::MOD as i32)),
        ))
    }
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn pairs(first: __m256i, second: __m256i) -> __m256i {
        let factor = _mm256_shuffle_epi32::<0xa0>(second);
        let shift = _mm256_blend_epi32::<0xaa>(_mm256_setzero_si256(), second);
        add_mod_256(
            montgomery_mul_256(
                first,
                factor,
                _mm256_set1_epi32(Mod::R as i32),
                _mm256_set1_epi32(Mod::MOD as i32),
            ),
            shift,
            _mm256_set1_epi32((2 * Mod::MOD) as i32),
        )
    }
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn avx2<const FULL: bool>(node: &CompositeLeaf, range: Range<usize>) -> Affine {
        let p = _mm256_set1_epi32(Mod::MOD as i32);
        let p2 = _mm256_add_epi32(p, p);
        let inv = _mm256_set1_epi32(Mod::R as i32);
        let one = _mm256_set1_epi32(ONE as i32);
        let zero = _mm256_setzero_si256();
        let mut a = [zero; 2];
        let mut b = [zero; 2];
        for base in [0, 8] {
            let lanes = _mm256_add_epi32(
                _mm256_setr_epi32(0, 8, 4, 12, 2, 10, 6, 14),
                _mm256_set1_epi32((base / 8) as i32),
            );
            let mask = if FULL {
                _mm256_set1_epi32(-1)
            } else {
                _mm256_and_si256(
                    _mm256_cmpgt_epi32(lanes, _mm256_set1_epi32(range.start as i32 - 1)),
                    _mm256_cmpgt_epi32(_mm256_set1_epi32(range.end as i32), lanes),
                )
            };
            a[base / 8] = _mm256_blendv_epi8(
                one,
                _mm256_loadu_si256(node.mul.as_ptr().add(base).cast()),
                mask,
            );
            b[base / 8] = _mm256_blendv_epi8(
                zero,
                _mm256_loadu_si256(node.add.as_ptr().add(base).cast()),
                mask,
            );
        }
        if !FULL && (range.end <= 8 || range.start >= 8) {
            if range.start >= 8 {
                for i in 0..2 {
                    a[i] = _mm256_srli_epi64::<32>(a[i]);
                    b[i] = _mm256_srli_epi64::<32>(b[i]);
                }
            }
            let mut x = even(a[0], a[1]);
            let mut y = add_mod_256(even(b[0], a[1]), b[1], p2);
            let rx = _mm256_permute2x128_si256::<1>(x, x);
            let ry = _mm256_permute2x128_si256::<1>(y, y);
            x = even(x, rx);
            y = add_mod_256(even(y, rx), ry, p2);
            let rx = _mm256_shuffle_epi32::<0xee>(x);
            let ry = _mm256_shuffle_epi32::<0xee>(y);
            x = even(x, rx);
            y = add_mod_256(even(y, rx), ry, p2);
            return (
                _mm256_extract_epi32::<0>(x) as u32,
                _mm256_extract_epi32::<0>(y) as u32,
            );
        }
        if !FULL {
            let x = montgomery_mul_256(a[0], a[1], inv, p);
            let y = add_mod_256(montgomery_mul_256(b[0], a[1], inv, p), b[1], p2);
            let low = _mm256_unpacklo_epi32(x, y);
            let high = _mm256_unpackhi_epi32(x, y);
            let first = _mm256_permute2x128_si256::<0x20>(low, high);
            let second = _mm256_permute2x128_si256::<0x31>(low, high);
            let quads = pairs(first, second);
            let halves = pairs(quads, _mm256_permute2x128_si256::<1>(quads, quads));
            let last = _mm256_castsi256_si128(halves);
            return compose(
                (
                    _mm_cvtsi128_si32(last) as u32,
                    _mm_extract_epi32::<1>(last) as u32,
                ),
                (
                    _mm_extract_epi32::<2>(last) as u32,
                    _mm_extract_epi32::<3>(last) as u32,
                ),
            );
        }
        reduce256(a, b)
    }
    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn reduce256(a: [__m256i; 2], b: [__m256i; 2]) -> Affine {
        let p = _mm256_set1_epi32(Mod::MOD as i32);
        let p2 = _mm256_add_epi32(p, p);
        let inv = _mm256_set1_epi32(Mod::R as i32);
        let mut x = montgomery_mul_256(a[0], a[1], inv, p);
        let mut y = add_mod_256(montgomery_mul_256(b[0], a[1], inv, p), b[1], p2);
        for step in 0..3 {
            let (rx, ry) = match step {
                0 => (
                    _mm256_permute2x128_si256::<1>(x, x),
                    _mm256_permute2x128_si256::<1>(y, y),
                ),
                1 => (
                    _mm256_shuffle_epi32::<0xee>(x),
                    _mm256_shuffle_epi32::<0xee>(y),
                ),
                _ => (
                    _mm256_shuffle_epi32::<0x55>(x),
                    _mm256_shuffle_epi32::<0x55>(y),
                ),
            };
            x = montgomery_mul_256(x, rx, inv, p);
            y = add_mod_256(montgomery_mul_256(y, rx, inv, p), ry, p2);
        }
        (
            _mm256_extract_epi32::<0>(x) as u32,
            _mm256_extract_epi32::<0>(y) as u32,
        )
    }
    #[inline]
    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    unsafe fn avx512<const FULL: bool>(node: &CompositeLeaf, range: Range<usize>) -> Affine {
        let p = _mm512_set1_epi32(Mod::MOD as i32);
        let p2 = _mm512_add_epi32(p, p);
        let inv = _mm512_set1_epi32(Mod::R as i32);
        let mask = if FULL {
            u16::MAX
        } else {
            PREFIX_MASK[range.end] ^ PREFIX_MASK[range.start]
        };
        let mut a = _mm512_mask_blend_epi32(
            mask,
            _mm512_set1_epi32(ONE as i32),
            _mm512_loadu_si512(node.mul.as_ptr().cast()),
        );
        let mut b = _mm512_maskz_mov_epi32(mask, _mm512_loadu_si512(node.add.as_ptr().cast()));
        for shift in [8, 4, 2, 1] {
            let indices: [i32; 16] = std::array::from_fn(|i| (i ^ shift) as i32);
            let index = _mm512_loadu_si512(indices.as_ptr().cast());
            let ra = _mm512_permutexvar_epi32(index, a);
            let rb = _mm512_permutexvar_epi32(index, b);
            a = montgomery_mul_512(a, ra, inv, p);
            b = add_mod_512(montgomery_mul_512(b, ra, inv, p), rb, p2);
        }
        (
            _mm_cvtsi128_si32(_mm512_castsi512_si128(a)) as u32,
            _mm_cvtsi128_si32(_mm512_castsi512_si128(b)) as u32,
        )
    }
}

#[codesnip::entry("DaryAffineSumKernel")]
pub use self::dary_affine_sum_kernel::DaryAffineSumKernel;

#[codesnip::entry(
    "DaryAffineSumKernel",
    include("DaryLazySegmentTree", "_DarySumVector")
)]
#[allow(unsafe_op_in_unsafe_fn)] // The generic tree checks the selected ISA.
mod dary_affine_sum_kernel {
    #[cfg(target_arch = "x86_64")]
    use super::dary_sum_vector;
    use super::{
        DaryLazyKernel, MIntBase, MIntConvert, SumLeaf, dary_affine_arithmetic::*, dary_sum_leaf,
    };
    #[repr(C, align(64))]
    #[derive(Clone, Debug)]
    pub struct SumBranch {
        values: [u32; B],
        mul: [u32; B],
        add: [u32; B],
    }

    /// Affine updates and sums modulo 998244353, with Montgomery residues in [0, 2p).
    #[derive(Clone, Debug)]
    pub struct DaryAffineSumKernel;

    impl DaryLazyKernel for DaryAffineSumKernel {
        const B: usize = B;
        const PAIRED_UPDATES: bool = true;
        type Key = M;
        type EncodedKey = u32;
        type Output = M;
        type Action = (M, M);
        type State = ();
        fn new_state(_len: usize) {}
        fn encode(key: M) -> u32 {
            <Mod as MIntConvert<_>>::from(key.inner())
        }
        fn decode(value: u32) -> M {
            M::new_unchecked(Mod::mod_inner(value))
        }
        type Summary = u32;
        type Tag = Affine;
        type Prepared = Affine;
        type Change = u32;
        type Leaf = SumLeaf;
        type Branch = SumBranch;

        fn unit() -> u32 {
            0
        }
        fn tag_unit() -> Affine {
            (ONE, 0)
        }
        fn combine(left: u32, right: u32) -> u32 {
            mod_add(left, right)
        }
        fn compose(old: Affine, later: Affine) -> Affine {
            compose(old, later)
        }
        fn prepare(
            _state: &mut (),
            (a, b): (M, M),
            _range: Range<usize>,
            _len: usize,
            _branches: &[Vec<Self::Branch>],
        ) -> Affine {
            (
                <Mod as MIntConvert<_>>::from(a.inner()),
                <Mod as MIntConvert<_>>::from(b.inner()),
            )
        }
        fn map(_state: &(), value: u32, tag: Affine, len: usize) -> u32 {
            if tag == (ONE, 0) {
                return value;
            }
            // Multiplication by an integer preserves the Montgomery representation.
            let offset = ((tag.1 as u64 * (len as u64 % Mod::MOD as u64)) % Mod::MOD as u64) as u32;
            mod_add(mod_mul(value, tag.0), offset)
        }
        fn repair(value: u32, change: u32) -> u32 {
            mod_add(value, change)
        }
        fn leaf(keys: impl Iterator<Item = u32>) -> SumLeaf {
            dary_sum_leaf::leaf(keys)
        }
        fn branch(values: &[u32]) -> SumBranch {
            SumBranch {
                values: Self::leaf(values.iter().copied()).0,
                mul: [ONE; B],
                add: [0; B],
            }
        }
        unsafe fn fold_leaf<const ISA: u8>(
            _state: &(),
            node: &SumLeaf,
            range: Range<usize>,
        ) -> u32 {
            #[cfg(target_arch = "x86_64")]
            if ISA != 0 {
                return dary_sum_vector::sum::<ISA>(&node.0, range);
            }
            (node.0[range].iter().map(|&v| v as u64).sum::<u64>() % Mod::MOD as u64) as u32
        }
        unsafe fn fold_branch<const ISA: u8>(
            _state: &(),
            node: &SumBranch,
            range: Range<usize>,
        ) -> u32 {
            #[cfg(target_arch = "x86_64")]
            if ISA != 0 {
                return dary_sum_vector::sum::<ISA>(&node.values, range);
            }
            (node.values[range].iter().map(|&v| v as u64).sum::<u64>() % Mod::MOD as u64) as u32
        }
        fn tag(node: &SumBranch, lane: usize) -> Affine {
            (node.mul[lane], node.add[lane])
        }
        unsafe fn edit_leaf<const ISA: u8>(
            _state: &mut (),
            node: &mut SumLeaf,
            len: usize,
            range: Range<usize>,
            action: &Affine,
            carry: Affine,
        ) -> u32 {
            #[cfg(target_arch = "x86_64")]
            if ISA != 0 {
                return dary_sum_vector::edit_sum_leaf::<ISA>(node, len, range, *action, carry);
            }
            if carry != (ONE, 0) {
                for value in &mut node.0[..len] {
                    *value = Self::map(&(), *value, carry, 1);
                }
            }
            let mut delta = 0i64;
            for value in &mut node.0[range] {
                let old = *value;
                *value = Self::map(&(), old, *action, 1);
                delta += i64::from(*value) - i64::from(old);
            }
            delta.rem_euclid(i64::from(2 * Mod::MOD)) as u32
        }
        unsafe fn edit_branch<const ISA: u8>(
            _state: &mut (),
            node: &mut SumBranch,
            len: usize,
            width: usize,
            full: Range<usize>,
            edges: &[Option<(usize, u32)>; 2],
            action: &Affine,
            carry: Affine,
        ) -> u32 {
            #[cfg(target_arch = "x86_64")]
            if ISA != 0 {
                return edit_sum_branch::<ISA>(node, len, width, full, edges, *action, carry);
            }
            if carry != (ONE, 0) {
                let count = ((len - 1) >> width.trailing_zeros()) + 1;
                for lane in 0..count {
                    let span = width.min(len - lane * width);
                    node.values[lane] = Self::map(&(), node.values[lane], carry, span);
                    let tag = compose(Self::tag(node, lane), carry);
                    node.mul[lane] = tag.0;
                    node.add[lane] = tag.1;
                }
            }
            let mut delta = 0i64;
            for lane in full {
                let old = node.values[lane];
                let span = width.min(len - lane * width);
                node.values[lane] = Self::map(&(), old, *action, span);
                let tag = compose(Self::tag(node, lane), *action);
                node.mul[lane] = tag.0;
                node.add[lane] = tag.1;
                delta += i64::from(node.values[lane]) - i64::from(old);
            }
            for &(lane, change) in edges.iter().flatten() {
                node.values[lane] = mod_add(node.values[lane], change);
                node.mul[lane] = ONE;
                node.add[lane] = 0;
                delta += i64::from(change);
            }
            delta.rem_euclid(i64::from(2 * Mod::MOD)) as u32
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    unsafe fn edit_sum_branch<const ISA: u8>(
        node: &mut SumBranch,
        len: usize,
        width: usize,
        full: Range<usize>,
        edges: &[Option<(usize, u32)>; 2],
        action: Affine,
        carry: Affine,
    ) -> u32 {
        let tags = Some((&mut node.mul, &mut node.add));
        let mut delta = if full.is_empty() && carry == (ONE, 0) {
            0
        } else if ISA == 1 {
            dary_sum_vector::avx2::edit_sum(&mut node.values, tags, len, width, full, action, carry)
        } else {
            dary_sum_vector::avx512::edit_sum(
                &mut node.values,
                tags,
                len,
                width,
                full,
                action,
                carry,
            )
        };
        for &(lane, change) in edges.iter().flatten() {
            node.values[lane] = mod_add(node.values[lane], change);
            node.mul[lane] = ONE;
            node.add[lane] = 0;
            delta = mod_add(delta, change);
        }
        delta
    }
}

#[codesnip::entry("DaryAffinePointKernel")]
pub use self::dary_affine_point_kernel::DaryAffinePointKernel;

#[codesnip::entry(
    "DaryAffinePointKernel",
    include("DaryDualSegmentTree", "_DarySumVector")
)]
#[allow(unsafe_op_in_unsafe_fn)] // The generic tree checks the selected ISA.
mod dary_affine_point_kernel {
    #[cfg(target_arch = "x86_64")]
    use super::dary_sum_vector;
    use super::{
        DaryDualKernel, MIntBase, MIntConvert, SumLeaf, dary_affine_arithmetic::*, dary_sum_leaf,
    };
    /// Affine range updates and point access modulo 998244353.
    #[derive(Clone, Debug)]
    pub struct DaryAffinePointKernel;
    #[repr(C, align(64))]
    #[derive(Clone, Debug)]
    pub struct AffineTagBlock {
        mul: [u32; B],
        add: [u32; B],
    }

    impl DaryDualKernel for DaryAffinePointKernel {
        const B: usize = B;
        type Key = M;
        type Value = u32;
        type Output = M;
        type Action = (M, M);
        type Tag = Affine;
        type Leaf = SumLeaf;
        type Branch = AffineTagBlock;
        fn encode(key: M) -> u32 {
            <Mod as MIntConvert<_>>::from(key.inner())
        }
        fn decode(value: u32) -> M {
            M::new_unchecked(Mod::mod_inner(value))
        }
        fn prepare(action: (M, M)) -> Affine {
            (
                <Mod as MIntConvert<_>>::from(action.0.inner()),
                <Mod as MIntConvert<_>>::from(action.1.inner()),
            )
        }
        fn unit() -> Affine {
            (ONE, 0)
        }
        fn compose(old: Affine, later: Affine) -> Affine {
            compose(old, later)
        }
        fn apply(value: u32, tag: Affine) -> u32 {
            if tag == (ONE, 0) {
                value
            } else {
                mod_add(mod_mul(value, tag.0), tag.1)
            }
        }
        fn leaf(values: impl Iterator<Item = u32>) -> SumLeaf {
            dary_sum_leaf::leaf(values)
        }
        fn branch() -> AffineTagBlock {
            AffineTagBlock {
                mul: [ONE; B],
                add: [0; B],
            }
        }
        fn get(node: &SumLeaf, lane: usize) -> u32 {
            node.0[lane]
        }
        fn tag(node: &AffineTagBlock, lane: usize) -> Affine {
            (node.mul[lane], node.add[lane])
        }
        unsafe fn edit_leaf<const ISA: u8>(
            node: &mut SumLeaf,
            len: usize,
            range: Range<usize>,
            action: Affine,
            carry: Affine,
        ) {
            Self::edit::<ISA>(&mut node.0, None, len, range, action, carry);
        }
        unsafe fn set_leaf<const ISA: u8>(
            node: &mut SumLeaf,
            len: usize,
            lane: usize,
            value: u32,
            carry: Affine,
        ) {
            Self::edit::<ISA>(&mut node.0, None, len, 0..0, (ONE, 0), carry);
            node.0[lane] = value;
        }
        unsafe fn edit_branch<const ISA: u8>(
            node: &mut AffineTagBlock,
            len: usize,
            full: Range<usize>,
            edges: &[Option<usize>; 2],
            action: Affine,
            carry: Affine,
        ) {
            Self::edit::<ISA>(&mut node.add, Some(&mut node.mul), len, full, action, carry);
            for &lane in edges.iter().flatten() {
                node.mul[lane] = ONE;
                node.add[lane] = 0;
            }
        }
    }
    impl DaryAffinePointKernel {
        unsafe fn edit<const ISA: u8>(
            values: &mut [u32; B],
            mut multipliers: Option<&mut [u32; B]>,
            len: usize,
            range: Range<usize>,
            action: Affine,
            carry: Affine,
        ) {
            if carry == (ONE, 0) && range.is_empty() {
                return;
            }
            #[cfg(target_arch = "x86_64")]
            if ISA != 0 {
                dary_sum_vector::edit_affine::<ISA>(values, multipliers, len, range, action, carry);
                return;
            }
            let combined = compose(carry, action);
            for (lane, value) in values[..len].iter_mut().enumerate() {
                let tag = if range.contains(&lane) {
                    combined
                } else {
                    carry
                };
                *value = Self::apply(*value, tag);
                if let Some(mul) = &mut multipliers {
                    mul[lane] = mod_mul(mul[lane], tag.0);
                }
            }
        }
    }
}

#[codesnip::entry("DaryAssignCompositeKernel")]
pub use self::dary_assign_composite_kernel::DaryAssignCompositeKernel;

#[codesnip::entry(
    "DaryAssignCompositeKernel",
    include("DaryLazySegmentTree", "_DaryCompositeLeaf")
)]
#[allow(unsafe_op_in_unsafe_fn)] // The generic tree checks the selected ISA.
mod dary_assign_composite_kernel {
    #[cfg(target_arch = "x86_64")]
    use super::bitrev_vector;
    use super::{
        CompositeLeaf, DaryLazyKernel, MIntBase, MIntConvert, dary_affine_arithmetic::*,
        dary_composite_leaf,
    };
    #[repr(C, align(64))]
    #[derive(Clone, Debug)]
    pub struct CompositeBranch {
        values: CompositeLeaf,
        tags: [u32; B],
    }

    /// Per-tree power records shared by live assignment tags.
    #[derive(Clone, Debug)]
    pub struct AssignmentPowerStore {
        powers: Vec<Affine>,
        free: Vec<u32>,
        live: Vec<bool>,
        width: usize,
        lasting: bool,
    }

    impl AssignmentPowerStore {
        fn new(len: usize) -> Self {
            let width = len.max(1).ilog2() as usize + 1;
            let mut slots = 0;
            let mut count = len.div_ceil(B);
            while count > 1 {
                count = count.div_ceil(B);
                slots += count * B;
            }
            // A sweep leaves more than slots free records, even when every tag is live.
            let capacity = 2 * slots + 2;
            assert!(u32::try_from(capacity).is_ok());
            Self {
                powers: vec![(ONE, 0); capacity * width],
                free: (1..capacity as u32).rev().collect(),
                live: vec![false; capacity],
                width,
                lasting: false,
            }
        }

        fn insert(&mut self, value: Affine, len: usize, tags: impl Iterator<Item = u32>) -> u32 {
            if self.free.is_empty() {
                self.live.fill(false);
                for tag in tags {
                    self.live[tag as usize] = true;
                }
                self.free.extend(
                    (1..self.live.len())
                        .rev()
                        .filter(|&i| !self.live[i])
                        .map(|i| i as u32),
                );
            }
            let tag = self.free.pop().expect("live tag capacity");
            self.lasting = false;
            let start = tag as usize * self.width;
            let record = &mut self.powers[start..][..=len.ilog2() as usize];
            let mut power = value;
            record[0] = power;
            for next in &mut record[1..] {
                let (a, b) = power;
                power = (mod_mul(a, a), mod_mul(mod_add(a, ONE), b));
                *next = power;
            }
            tag
        }

        fn retain(&mut self) {
            self.lasting = true;
        }

        fn finish(&mut self, tag: u32) {
            if !self.lasting {
                self.free.push(tag);
            }
        }

        fn step(&self, tag: u32, shift: usize) -> Affine {
            self.powers[tag as usize * self.width + shift]
        }

        #[inline]
        fn power(&self, tag: u32, mut count: usize) -> Affine {
            if count == 0 {
                return (ONE, 0);
            }
            let start = tag as usize * self.width;
            let mut value = self.powers[start + count.trailing_zeros() as usize];
            count &= count - 1;
            while count != 0 {
                value = compose(value, self.powers[start + count.trailing_zeros() as usize]);
                count &= count - 1;
            }
            value
        }
    }

    /// Range assignment and ordered affine composition modulo 998244353.
    /// Cached powers are reclaimed by scanning stored tags; this cost is amortized across updates.
    #[derive(Clone, Debug)]
    pub struct DaryAssignCompositeKernel;

    impl DaryLazyKernel for DaryAssignCompositeKernel {
        const FOLD_UPWARD: bool = false;
        const B: usize = B;
        type Key = (M, M);
        type EncodedKey = Affine;
        type Output = (M, M);
        type Action = (M, M);
        type State = AssignmentPowerStore;
        fn new_state(len: usize) -> AssignmentPowerStore {
            AssignmentPowerStore::new(len)
        }

        fn encode((a, b): (M, M)) -> Affine {
            (
                <Mod as MIntConvert<_>>::from(a.inner()),
                <Mod as MIntConvert<_>>::from(b.inner()),
            )
        }
        fn decode((a, b): Affine) -> (M, M) {
            (
                M::new_unchecked(Mod::mod_inner(a)),
                M::new_unchecked(Mod::mod_inner(b)),
            )
        }
        type Summary = Affine;
        type Tag = u32;
        type Prepared = u32;
        type Change = Affine;
        type Leaf = CompositeLeaf;
        type Branch = CompositeBranch;

        fn unit() -> Affine {
            (ONE, 0)
        }
        fn tag_unit() -> Self::Tag {
            0
        }
        fn combine(left: Affine, right: Affine) -> Affine {
            compose(left, right)
        }
        fn compose(old: Self::Tag, later: Self::Tag) -> Self::Tag {
            if later != 0 { later } else { old }
        }
        fn prepare(
            state: &mut AssignmentPowerStore,
            action: (M, M),
            range: Range<usize>,
            _len: usize,
            branches: &[Vec<CompositeBranch>],
        ) -> u32 {
            state.insert(
                Self::encode(action),
                range.len(),
                branches
                    .iter()
                    .flatten()
                    .flat_map(|node| node.tags.iter().copied()),
            )
        }
        fn finish(state: &mut AssignmentPowerStore, action: u32) {
            state.finish(action);
        }

        fn map(state: &AssignmentPowerStore, value: Affine, tag: u32, len: usize) -> Affine {
            if tag == 0 {
                value
            } else {
                state.power(tag, len)
            }
        }
        fn constant(state: &AssignmentPowerStore, tag: u32, len: usize) -> Option<Affine> {
            (tag != 0).then(|| state.power(tag, len))
        }
        fn repair(_value: Affine, change: Affine) -> Affine {
            change
        }
        fn leaf(keys: impl Iterator<Item = Affine>) -> CompositeLeaf {
            dary_composite_leaf::leaf(keys)
        }
        fn branch(values: &[Affine]) -> CompositeBranch {
            CompositeBranch {
                values: Self::leaf(values.iter().copied()),
                tags: [0; B],
            }
        }
        unsafe fn fold_leaf<const ISA: u8>(
            _state: &AssignmentPowerStore,
            node: &CompositeLeaf,
            range: Range<usize>,
        ) -> Affine {
            dary_composite_leaf::fold::<ISA>(node, range)
        }
        unsafe fn fold_branch<const ISA: u8>(
            state: &AssignmentPowerStore,
            node: &CompositeBranch,
            range: Range<usize>,
        ) -> Affine {
            Self::fold_leaf::<ISA>(state, &node.values, range)
        }
        fn tag(node: &CompositeBranch, lane: usize) -> Self::Tag {
            node.tags[lane]
        }
        unsafe fn edit_leaf<const ISA: u8>(
            state: &mut AssignmentPowerStore,
            node: &mut CompositeLeaf,
            len: usize,
            range: Range<usize>,
            action: &u32,
            carry: u32,
        ) -> Affine {
            let previous = (carry != 0).then(|| state.step(carry, 0));
            let value = state.step(*action, 0);
            node.assign::<ISA>(
                None, len, range, value, previous, *action, carry, None, &[None; 2],
            )
        }
        unsafe fn edit_branch<const ISA: u8>(
            state: &mut AssignmentPowerStore,
            node: &mut CompositeBranch,
            len: usize,
            width: usize,
            full: Range<usize>,
            edges: &[Option<(usize, Affine)>; 2],
            action: &u32,
            carry: u32,
        ) -> Affine {
            let shift = width.trailing_zeros() as usize;
            let wide = len >> shift;
            let short = len & (width - 1);
            let count = wide + (short != 0) as usize;
            let previous = (carry != 0).then(|| {
                if wide != 0 {
                    state.step(carry, shift)
                } else {
                    state.power(carry, short)
                }
            });
            let value = if full.is_empty() {
                (ONE, 0)
            } else {
                state.retain();
                if full.start < wide {
                    state.step(*action, shift)
                } else {
                    state.power(*action, short)
                }
            };
            let tail = if short != 0 && wide != 0 {
                let lane = count - 1;
                let tag = if full.contains(&lane) { *action } else { carry };
                if tag == 0 {
                    None
                } else {
                    Some((lane, state.power(tag, short)))
                }
            } else {
                None
            };
            node.values.assign::<ISA>(
                Some(&mut node.tags),
                count,
                full,
                value,
                previous,
                *action,
                carry,
                tail,
                edges,
            )
        }
    }

    impl CompositeLeaf {
        #[allow(clippy::too_many_arguments)] // Keeps the update and boundary repairs in one reduction.
        unsafe fn assign<const ISA: u8>(
            &mut self,
            mut tags: Option<&mut [u32; B]>,
            len: usize,
            range: Range<usize>,
            value: Affine,
            carry: Option<Affine>,
            action_tag: u32,
            carry_tag: u32,
            tail: Option<(usize, Affine)>,
            edges: &[Option<(usize, Affine)>; 2],
        ) -> Affine {
            #[cfg(target_arch = "x86_64")]
            if ISA != 0 {
                return assign_vector::assign::<ISA>(
                    self, tags, len, range, value, carry, action_tag, carry_tag, tail, edges,
                );
            }
            if let Some((mul, add)) = carry {
                for &lane in &REV[..len] {
                    self.mul[lane] = mul;
                    self.add[lane] = add;
                }
                if let Some(tags) = &mut tags {
                    tags[..len].fill(carry_tag);
                }
            }
            for lane in range.clone() {
                self.mul[REV[lane]] = value.0;
                self.add[REV[lane]] = value.1;
            }
            if let Some(tags) = &mut tags {
                tags[range].fill(action_tag);
            }
            self.repair_edges(tags, tail, edges);
            dary_composite_leaf::refold::<ISA>(self)
        }

        fn repair_edges(
            &mut self,
            mut tags: Option<&mut [u32; B]>,
            tail: Option<(usize, Affine)>,
            edges: &[Option<(usize, Affine)>; 2],
        ) {
            if let Some((lane, (a, b))) = tail {
                self.mul[REV[lane]] = a;
                self.add[REV[lane]] = b;
            }
            for &(lane, (a, b)) in edges.iter().flatten() {
                self.mul[REV[lane]] = a;
                self.add[REV[lane]] = b;
                if let Some(tags) = &mut tags {
                    tags[lane] = 0;
                }
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[allow(clippy::too_many_arguments)] // Each backend receives the same fused update parameters.
    mod assign_vector {
        use super::*;
        use std::arch::x86_64::*;

        fn masks(len: usize, range: Range<usize>) -> [(u16, u16); 2] {
            [
                (
                    PREFIX_MASK[len],
                    PREFIX_MASK[range.end] ^ PREFIX_MASK[range.start],
                ),
                (
                    ((1u32 << len) - 1) as u16,
                    ((1u32 << range.end) - (1u32 << range.start)) as u16,
                ),
            ]
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn expand(mask: u16, base: usize) -> __m256i {
            let bits = _mm256_setr_epi32(1, 2, 4, 8, 16, 32, 64, 128);
            let selected = _mm256_and_si256(_mm256_set1_epi32((mask >> base) as i32), bits);
            _mm256_cmpeq_epi32(selected, bits)
        }

        pub unsafe fn assign<const ISA: u8>(
            node: &mut CompositeLeaf,
            mut tags: Option<&mut [u32; B]>,
            len: usize,
            range: Range<usize>,
            value: Affine,
            carry: Option<Affine>,
            action_tag: u32,
            carry_tag: u32,
            tail: Option<(usize, Affine)>,
            edges: &[Option<(usize, Affine)>; 2],
        ) -> Affine {
            if ISA == 1 || is_x86_feature_detected!("avx2") {
                return avx2(
                    node, tags, len, range, value, carry, action_tag, carry_tag, tail, edges,
                );
            }
            avx512(
                node,
                tags.as_deref_mut(),
                len,
                range,
                value,
                carry,
                action_tag,
                carry_tag,
            );
            node.repair_edges(tags, tail, edges);
            dary_composite_leaf::refold::<ISA>(node)
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn avx2(
            node: &mut CompositeLeaf,
            mut tags: Option<&mut [u32; B]>,
            len: usize,
            range: Range<usize>,
            value: Affine,
            carry: Option<Affine>,
            action_tag: u32,
            carry_tag: u32,
            tail: Option<(usize, Affine)>,
            edges: &[Option<(usize, Affine)>; 2],
        ) -> Affine {
            let [(valid, selected), (tag_valid, tag_selected)] = masks(len, range);
            let a = _mm256_set1_epi32(value.0 as i32);
            let b = _mm256_set1_epi32(value.1 as i32);
            let tag = _mm256_set1_epi32(action_tag as i32);
            let mut all_a = [_mm256_setzero_si256(); 2];
            let mut all_b = all_a;
            for base in [0, 8] {
                let selected = expand(selected, base);
                let pa = node.mul.as_mut_ptr().add(base).cast();
                let pb = node.add.as_mut_ptr().add(base).cast();
                let mut x = _mm256_loadu_si256(pa);
                let mut y = _mm256_loadu_si256(pb);
                if let Some((a, b)) = carry {
                    let valid = expand(valid, base);
                    x = _mm256_blendv_epi8(x, _mm256_set1_epi32(a as i32), valid);
                    y = _mm256_blendv_epi8(y, _mm256_set1_epi32(b as i32), valid);
                }
                x = _mm256_blendv_epi8(x, a, selected);
                y = _mm256_blendv_epi8(y, b, selected);
                for &(lane, (a, b)) in tail.iter().chain(edges.iter().flatten()) {
                    let selected = expand(1 << REV[lane], base);
                    x = _mm256_blendv_epi8(x, _mm256_set1_epi32(a as i32), selected);
                    y = _mm256_blendv_epi8(y, _mm256_set1_epi32(b as i32), selected);
                }
                _mm256_storeu_si256(pa, x);
                _mm256_storeu_si256(pb, y);
                all_a[base / 8] = x;
                all_b[base / 8] = y;
                if let Some(tags) = &mut tags {
                    let tag_selected = expand(tag_selected, base);
                    let p = tags.as_mut_ptr().add(base).cast();
                    let mut old = _mm256_loadu_si256(p);
                    if carry.is_some() {
                        let tag_valid = expand(tag_valid, base);
                        old =
                            _mm256_blendv_epi8(old, _mm256_set1_epi32(carry_tag as i32), tag_valid);
                    }
                    _mm256_storeu_si256(p, _mm256_blendv_epi8(old, tag, tag_selected));
                }
            }
            if let Some(tags) = tags {
                for &(lane, _) in edges.iter().flatten() {
                    tags[lane] = 0;
                }
            }
            bitrev_vector::reduce256(all_a, all_b)
        }
        #[inline]
        #[target_feature(enable = "avx512f")]
        unsafe fn avx512(
            node: &mut CompositeLeaf,
            tags: Option<&mut [u32; B]>,
            len: usize,
            range: Range<usize>,
            value: Affine,
            carry: Option<Affine>,
            action_tag: u32,
            carry_tag: u32,
        ) {
            let [(valid, selected), (tag_valid, tag_selected)] = masks(len, range);
            let pa = node.mul.as_mut_ptr().cast();
            let pb = node.add.as_mut_ptr().cast();
            let mut a = _mm512_loadu_si512(pa);
            let mut b = _mm512_loadu_si512(pb);
            if let Some((ca, cb)) = carry {
                a = _mm512_mask_blend_epi32(valid, a, _mm512_set1_epi32(ca as i32));
                b = _mm512_mask_blend_epi32(valid, b, _mm512_set1_epi32(cb as i32));
            }
            _mm512_storeu_si512(
                pa,
                _mm512_mask_blend_epi32(selected, a, _mm512_set1_epi32(value.0 as i32)),
            );
            _mm512_storeu_si512(
                pb,
                _mm512_mask_blend_epi32(selected, b, _mm512_set1_epi32(value.1 as i32)),
            );
            if let Some(tags) = tags {
                let p = tags.as_mut_ptr().cast();
                let mut old = _mm512_loadu_si512(p);
                if carry.is_some() {
                    old = _mm512_mask_blend_epi32(
                        tag_valid,
                        old,
                        _mm512_set1_epi32(carry_tag as i32),
                    );
                }
                _mm512_storeu_si512(
                    p,
                    _mm512_mask_blend_epi32(
                        tag_selected,
                        old,
                        _mm512_set1_epi32(action_tag as i32),
                    ),
                );
            }
        }
    }
}

#[codesnip::entry("DaryInvertibleCompositeKernel")]
pub use self::dary_invertible_composite_kernel::{
    AffineFraction, DaryInvertibleCompositeKernel, InvertibleAffine, InvertibleAffineBlock,
    InvertibleAffineValue,
};

#[codesnip::entry(
    "DaryInvertibleCompositeKernel",
    include("DarySegmentTree", "_DaryAffineArithmetic", "montgomery_simd")
)]
#[allow(unsafe_op_in_unsafe_fn)] // The generic tree checks the selected ISA.
mod dary_invertible_composite_kernel {
    #[cfg(target_arch = "x86_64")]
    use super::montgomery_simd;
    use super::{
        DarySegmentGroup, DarySegmentKernel, MIntBase, MIntConvert, dary_affine_arithmetic::*,
    };
    #[inline]
    fn canonical(x: u32) -> u32 {
        if x >= Mod::MOD { x - Mod::MOD } else { x }
    }
    // Two products reduce below 3p; one subtraction of 2p restores [0, 2p).
    #[inline]
    fn mul_sum(a: u32, b: u32, c: u32, d: u32) -> u32 {
        let product = a as u64 * b as u64 + c as u64 * d as u64;
        let reduced = ((product + (product as u32).wrapping_mul(Mod::R) as u64 * Mod::MOD as u64)
            >> 32) as u32;
        reduced.min(reduced.wrapping_sub(2 * Mod::MOD))
    }
    fn invert_affine(x: u32) -> u32 {
        Mod::mod_inv(canonical(x))
    }
    fn invert_affine_products<const N: usize>(products: &mut [u32]) {
        let mut before = [ONE; N];
        let mut product = ONE;
        for (i, &x) in products.iter().enumerate() {
            before[i] = product;
            product = mod_mul(product, x);
        }
        let mut inverse = invert_affine(product);
        for i in (0..products.len()).rev() {
            let next = mod_mul(inverse, products[i]);
            products[i] = mod_mul(inverse, before[i]);
            inverse = next;
        }
    }
    fn invert_affine_batch(values: &mut [u32]) {
        if values.is_empty() {
            return;
        }
        if values.len() <= 32 {
            invert_affine_products::<32>(values);
            return;
        }
        let mut prefixes = vec![0; values.len()];
        #[cfg(target_arch = "x86_64")]
        {
            use super::{SimdBackend, simd_backend};
            unsafe {
                match simd_backend() {
                    SimdBackend::Avx512 => {
                        invertible_vector::invert512(values, &mut prefixes);
                        return;
                    }
                    SimdBackend::Avx2 => {
                        invertible_vector::invert256(values, &mut prefixes);
                        return;
                    }
                    _ => {}
                }
            }
        }
        let mut product = [ONE; 16];
        for (chunk, prefix) in values.chunks(16).zip(prefixes.chunks_mut(16)) {
            for i in 0..chunk.len() {
                prefix[i] = product[i];
                product[i] = mod_mul(product[i], chunk[i]);
            }
        }
        invert_affine_products::<16>(&mut product);
        for (chunk, prefix) in values.chunks_mut(16).zip(prefixes.chunks(16)).rev() {
            for i in 0..chunk.len() {
                let value = mod_mul(product[i], prefix[i]);
                product[i] = mod_mul(product[i], chunk[i]);
                chunk[i] = value;
            }
        }
    }
    /// An affine map modulo 998244353 with a nonzero multiplier.
    #[derive(Clone, Copy, Debug)]
    pub struct InvertibleAffine(u32, u32, u32);
    impl InvertibleAffine {
        pub fn new(a: M, b: M) -> Self {
            assert_ne!(a.inner(), 0);
            Self(<Mod as MIntConvert<u32>>::from(a.inner()), b.inner(), 0)
        }
        /// Caches inverses for later construction and point updates.
        pub fn prepare(keys: &mut [Self]) {
            let mut inverse: Vec<_> = keys.iter().map(|key| key.0).collect();
            invert_affine_batch(&mut inverse);
            for (key, inverse) in keys.iter_mut().zip(inverse) {
                key.2 = inverse;
            }
        }
    }
    /// An affine composition whose final division may be evaluated in a batch.
    #[derive(Clone, Copy, Debug)]
    pub struct AffineFraction(u32, u32, u32);
    impl AffineFraction {
        pub fn apply(self, x: M) -> M {
            M::new_unchecked(canonical(mod_mul(
                mod_sub(mod_mul(self.2, x.inner()), self.1),
                invert_affine(self.0),
            )))
        }
        /// Evaluates each composition at its paired argument, sharing the inversion work.
        pub fn apply_many(values: impl IntoIterator<Item = (Self, M)>) -> Vec<M> {
            let values = values.into_iter();
            let capacity = values.size_hint().0;
            let mut nums = Vec::with_capacity(capacity);
            let mut dens = Vec::with_capacity(capacity);
            for (value, x) in values {
                nums.push(mod_sub(mod_mul(value.2, x.inner()), value.1));
                dens.push(value.0);
            }
            invert_affine_batch(&mut dens);
            nums.into_iter()
                .zip(dens)
                .map(|(a, b)| M::new_unchecked(canonical(mod_mul(a, b))))
                .collect()
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct InvertibleAffineValue(u32, u32, u32, u32);

    #[derive(Clone, Debug)]
    #[repr(C, align(64))]
    pub struct InvertibleAffineBlock {
        a: [u32; B],
        b: [u32; B],
        raw: [u32; B],
    }
    /// Ordered compositions of affine maps with nonzero multipliers modulo 998244353.
    #[derive(Clone, Debug)]
    pub struct DaryInvertibleCompositeKernel;
    impl DarySegmentKernel for DaryInvertibleCompositeKernel {
        const B: usize = B;
        type Key = InvertibleAffine;
        type Value = InvertibleAffineValue;
        type Output = AffineFraction;
        type Change = (u32, u32);
        type Block = InvertibleAffineBlock;
        fn encode(InvertibleAffine(a, b, c): InvertibleAffine) -> InvertibleAffineValue {
            let inverse = if c == 0 { invert_affine(a) } else { c };
            InvertibleAffineValue(inverse, mod_sub(0, mod_mul(inverse, b)), ONE, a)
        }
        fn decode(v: InvertibleAffineValue) -> AffineFraction {
            AffineFraction(v.0, v.1, v.2)
        }
        fn unit() -> InvertibleAffineValue {
            InvertibleAffineValue(ONE, 0, ONE, ONE)
        }
        fn combine(l: InvertibleAffineValue, r: InvertibleAffineValue) -> InvertibleAffineValue {
            let a = if l.0 == ONE {
                r.0
            } else if r.0 == ONE {
                l.0
            } else {
                mod_mul(l.0, r.0)
            };
            let b = mod_add(
                if l.0 == ONE { r.1 } else { mod_mul(l.0, r.1) },
                if r.2 == ONE { l.1 } else { mod_mul(l.1, r.2) },
            );
            let d = if l.2 == ONE {
                r.2
            } else if r.2 == ONE {
                l.2
            } else {
                mod_mul(l.2, r.2)
            };
            InvertibleAffineValue(a, b, d, 0)
        }
        fn block(input: impl Iterator<Item = InvertibleAffineValue>) -> InvertibleAffineBlock {
            let mut node = InvertibleAffineBlock {
                a: [ONE; B],
                b: [0; B],
                raw: [ONE; B],
            };
            let mut invalid = false;
            for (i, v) in input.enumerate() {
                (node.a[i], node.b[i]) = Self::inverse_coefficients(v);
                if v.3 != 0 {
                    node.raw[i] = Self::forward_multiplier(v);
                } else {
                    invalid = true;
                }
            }
            for i in 1..B {
                node.b[i] = mod_add(node.b[i - 1], mod_mul(node.a[i - 1], node.b[i]));
                node.a[i] = mod_mul(node.a[i - 1], node.a[i]);
            }
            if invalid {
                node.raw[B - 1] |= 1 << 31;
            }
            node
        }
        fn get(node: &InvertibleAffineBlock, lane: usize) -> InvertibleAffineValue {
            unsafe { Self::fold::<0>(node, lane..lane + 1) }
        }
        fn repair_value(v: InvertibleAffineValue, (a, b): (u32, u32)) -> InvertibleAffineValue {
            InvertibleAffineValue(
                mod_mul(a, v.0),
                mod_add(
                    mod_mul(a, v.1),
                    if v.2 == ONE { b } else { mod_mul(b, v.2) },
                ),
                v.2,
                0,
            )
        }
        unsafe fn fold<const ISA: u8>(
            node: &InvertibleAffineBlock,
            r: Range<usize>,
        ) -> InvertibleAffineValue {
            if r.is_empty() {
                return Self::unit();
            }
            if r.start == 0 {
                return InvertibleAffineValue(node.a[r.end - 1], node.b[r.end - 1], ONE, 0);
            }
            InvertibleAffineValue(
                node.a[r.end - 1],
                mod_sub(node.b[r.end - 1], node.b[r.start - 1]),
                node.a[r.start - 1],
                0,
            )
        }
        unsafe fn update<const ISA: u8>(
            node: &mut InvertibleAffineBlock,
            lane: usize,
            value: InvertibleAffineValue,
        ) -> Option<(u32, u32)> {
            let old = Self::raw_a(node, lane);
            let delta = Self::forward_multiplier(value);
            let (a, b) = Self::inverse_coefficients(value);
            let shift = mul_sum(node.a[lane], b, mod_sub(ONE, a), node.b[lane]);
            Self::transform::<ISA>(node, lane, a, shift);
            Self::set_raw(node, lane, mod_mul(delta, old));
            Some((a, shift))
        }
        unsafe fn replace<const ISA: u8>(
            node: &mut InvertibleAffineBlock,
            lane: usize,
            value: InvertibleAffineValue,
        ) -> Option<(u32, u32)> {
            let old = Self::raw_a(node, lane);
            let new = Self::forward_multiplier(value);
            let (inverse, inverse_b) = Self::inverse_coefficients(value);
            let a = mod_mul(old, inverse);
            let (before_a, before_b) = if lane == 0 {
                (ONE, 0)
            } else {
                (node.a[lane - 1], node.b[lane - 1])
            };
            let shift = mod_add(
                before_b,
                mul_sum(before_a, inverse_b, mod_sub(0, a), node.b[lane]),
            );
            Self::transform::<ISA>(node, lane, a, shift);
            Self::set_raw(node, lane, new);
            Some((a, shift))
        }
        unsafe fn repair<const ISA: u8>(
            node: &mut InvertibleAffineBlock,
            lane: usize,
            (a, b): (u32, u32),
        ) -> Option<(u32, u32)> {
            let shift = if lane == 0 {
                b
            } else {
                mul_sum(node.a[lane - 1], b, mod_sub(ONE, a), node.b[lane - 1])
            };
            Self::transform::<ISA>(node, lane, a, shift);
            node.raw[B - 1] |= 1 << 31;
            Some((a, shift))
        }
    }
    impl DarySegmentGroup for DaryInvertibleCompositeKernel {
        fn inverse(
            InvertibleAffineValue(a, b, d, _): InvertibleAffineValue,
        ) -> InvertibleAffineValue {
            InvertibleAffineValue(d, mod_sub(0, b), a, 0)
        }
    }
    impl DaryInvertibleCompositeKernel {
        #[inline]
        fn inverse_coefficients(value: InvertibleAffineValue) -> Affine {
            if value.2 == ONE {
                (value.0, value.1)
            } else {
                let inverse = invert_affine(value.2);
                (mod_mul(value.0, inverse), mod_mul(value.1, inverse))
            }
        }

        fn forward_multiplier(v: InvertibleAffineValue) -> u32 {
            let ia = if v.3 != 0 { v.3 } else { invert_affine(v.0) };
            if v.2 == ONE { ia } else { mod_mul(v.2, ia) }
        }
        fn raw_a(node: &InvertibleAffineBlock, lane: usize) -> u32 {
            if node.raw[B - 1] >> 31 == 0 {
                node.raw[lane]
            } else {
                Self::forward_multiplier(Self::get(node, lane))
            }
        }
        fn set_raw(node: &mut InvertibleAffineBlock, lane: usize, a: u32) {
            let invalid = node.raw[B - 1] & (1 << 31);
            node.raw[lane] = a;
            node.raw[B - 1] |= invalid;
        }
        unsafe fn transform<const ISA: u8>(
            node: &mut InvertibleAffineBlock,
            lane: usize,
            a: u32,
            b: u32,
        ) {
            #[cfg(target_arch = "x86_64")]
            if ISA == 1 {
                invertible_vector::edit256(node, lane, a, b);
                return;
            } else if ISA == 2 {
                invertible_vector::edit512(node, lane, a, b);
                return;
            }
            for i in lane..B {
                node.a[i] = mod_mul(a, node.a[i]);
                node.b[i] = mod_add(mod_mul(a, node.b[i]), b);
            }
        }
    }
    #[cfg(target_arch = "x86_64")]
    mod invertible_vector {
        use super::montgomery_simd::*;
        use super::*;
        use std::arch::x86_64::*;
        #[target_feature(enable = "avx2")]
        pub unsafe fn edit256(node: &mut InvertibleAffineBlock, lane: usize, a: u32, b: u32) {
            let p = _mm256_set1_epi32(Mod::MOD as i32);
            let p2 = _mm256_add_epi32(p, p);
            let ar = _mm256_set1_epi32(a.wrapping_mul(Mod::R) as i32);
            let av = _mm256_set1_epi32(a as i32);
            let bv = _mm256_set1_epi32(b as i32);
            for base in [0, 8] {
                let lanes = _mm256_add_epi32(
                    _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7),
                    _mm256_set1_epi32(base as i32),
                );
                let mask = _mm256_cmpgt_epi32(lanes, _mm256_set1_epi32(lane as i32 - 1));
                let pa = node.a.as_mut_ptr().add(base).cast();
                let pb = node.b.as_mut_ptr().add(base).cast();
                let x = _mm256_loadu_si256(pa);
                let y = _mm256_loadu_si256(pb);
                let nx = montgomery_mul_256_fixed(x, av, ar, p);
                let ny = add_mod_256(montgomery_mul_256_fixed(y, av, ar, p), bv, p2);
                _mm256_storeu_si256(pa, _mm256_blendv_epi8(x, nx, mask));
                _mm256_storeu_si256(pb, _mm256_blendv_epi8(y, ny, mask));
            }
        }
        #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
        pub unsafe fn edit512(node: &mut InvertibleAffineBlock, lane: usize, a: u32, b: u32) {
            let p = _mm512_set1_epi32(Mod::MOD as i32);
            let ar = _mm512_set1_epi32(a.wrapping_mul(Mod::R) as i32);
            let av = _mm512_set1_epi32(a as i32);
            let mask = u16::MAX << lane;
            let pa = node.a.as_mut_ptr().cast();
            let pb = node.b.as_mut_ptr().cast();
            let x = _mm512_loadu_si512(pa);
            let y = _mm512_loadu_si512(pb);
            let nx = montgomery_mul_512_fixed(x, av, ar, p);
            let ny = add_mod_512(
                montgomery_mul_512_fixed(y, av, ar, p),
                _mm512_set1_epi32(b as i32),
                _mm512_add_epi32(p, p),
            );
            _mm512_storeu_si512(pa, _mm512_mask_blend_epi32(mask, x, nx));
            _mm512_storeu_si512(pb, _mm512_mask_blend_epi32(mask, y, ny));
        }
        macro_rules! invert_batches {
            ($name:ident, $features:literal, $lanes:literal, $chains:literal,
             $splat:ident, $load:ident, $store:ident, $mul:ident) => {
                #[target_feature(enable = $features)]
                pub unsafe fn $name(values: &mut [u32], prefix: &mut [u32]) {
                    const WIDTH: usize = $lanes * $chains;
                    let full = values.len() / WIDTH * WIDTH;
                    let tail = values.len() - full;
                    let p = $splat(Mod::MOD as i32);
                    let inv = $splat(Mod::R as i32);
                    let mut product = [$splat(ONE as i32); $chains];
                    for base in (0..full).step_by(WIDTH) {
                        for (chain, product) in product.iter_mut().enumerate() {
                            let at = base + chain * $lanes;
                            let x = $load(values.as_ptr().add(at).cast());
                            $store(prefix.as_mut_ptr().add(at).cast(), *product);
                            *product = $mul(*product, x, inv, p);
                        }
                    }

                    let mut last = [ONE; WIDTH];
                    last[..tail].copy_from_slice(&values[full..]);
                    let mut before = [0; WIDTH];
                    let mut totals = [0; WIDTH];
                    for (chain, product) in product.iter().copied().enumerate() {
                        let at = chain * $lanes;
                        $store(before.as_mut_ptr().add(at).cast(), product);
                        let x = $load(last.as_ptr().add(at).cast());
                        $store(totals.as_mut_ptr().add(at).cast(), $mul(product, x, inv, p));
                    }
                    invert_affine_products::<WIDTH>(&mut totals);
                    for i in 0..tail {
                        let value = values[full + i];
                        values[full + i] = mod_mul(totals[i], before[i]);
                        totals[i] = mod_mul(totals[i], value);
                    }
                    for (chain, product) in product.iter_mut().enumerate() {
                        *product = $load(totals.as_ptr().add(chain * $lanes).cast());
                    }

                    for base in (0..full).step_by(WIDTH).rev() {
                        for (chain, product) in product.iter_mut().enumerate() {
                            let at = base + chain * $lanes;
                            let x = $load(values.as_ptr().add(at).cast());
                            let before = $load(prefix.as_ptr().add(at).cast());
                            let out = $mul(*product, before, inv, p);
                            *product = $mul(*product, x, inv, p);
                            $store(values.as_mut_ptr().add(at).cast(), out);
                        }
                    }
                }
            };
        }

        invert_batches!(
            invert256,
            "avx2",
            8,
            2,
            _mm256_set1_epi32,
            _mm256_loadu_si256,
            _mm256_storeu_si256,
            montgomery_mul_256
        );
        invert_batches!(
            invert512,
            "avx512f,avx512dq,avx512cd,avx512bw,avx512vl",
            16,
            2,
            _mm512_set1_epi32,
            _mm512_loadu_si512,
            _mm512_storeu_si512,
            montgomery_mul_512
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AffineFraction, DaryInvertibleCompositeKernel, InvertibleAffine, dary_affine_arithmetic::M,
    };
    use crate::{data_structure::DarySegmentTree, tools::Xorshift};

    #[test]
    fn test_invertible_composite() {
        let modulus = 998_244_353u64;
        let mut rng = Xorshift::default();
        for n in (0..=65).chain([255, 256, 257, 4095, 4096, 4097]) {
            let mut values: Vec<_> = (0..n)
                .map(|i| {
                    (
                        if i % 3 == 0 {
                            1
                        } else {
                            rng.rand(modulus - 1) + 1
                        },
                        if i % 3 == 0 {
                            modulus - 1
                        } else {
                            rng.rand(modulus)
                        },
                    )
                })
                .collect();
            let mut keys: Vec<_> = values
                .iter()
                .map(|&(a, b)| InvertibleAffine::new(M::from(a), M::from(b)))
                .collect();
            InvertibleAffine::prepare(&mut keys);
            let mut tree =
                DarySegmentTree::<DaryInvertibleCompositeKernel>::from_keys(keys.into_iter());
            for step in 0..1000 {
                if n != 0 {
                    let index = match step % 7 {
                        0 => 0,
                        1 => n - 1,
                        _ => rng.rand(n as u64) as usize,
                    };
                    let a = if step % 11 == 0 {
                        modulus - 1
                    } else {
                        rng.rand(modulus - 1) + 1
                    };
                    let b = if step % 11 == 0 { 0 } else { rng.rand(modulus) };
                    let mut keys = [InvertibleAffine::new(M::from(a), M::from(b))];
                    if step % 2 == 0 {
                        InvertibleAffine::prepare(&mut keys);
                    }
                    match step % 4 {
                        0 | 1 => {
                            tree.set(index, keys[0]);
                            values[index] = (a, b);
                        }
                        2 => {
                            tree.update(index, keys[0]);
                            let old = values[index];
                            values[index] = (old.0 * a % modulus, (old.1 * a + b) % modulus);
                        }
                        _ => {
                            tree.clear(index);
                            values[index] = (1, 0);
                        }
                    }
                    for x in [0, 1, modulus - 1] {
                        assert_eq!(
                            tree.get(index).apply(M::from(x)).inner() as u64,
                            (values[index].0 * x + values[index].1) % modulus
                        );
                    }
                }
                let (left, right) = match step % 11 {
                    0 => (0, n),
                    1 => (n, n),
                    _ => {
                        let left = rng.rand(n as u64 + 1) as usize;
                        let right = rng.rand(n as u64 + 1) as usize;
                        (left.min(right), left.max(right))
                    }
                };
                let (a, b) = values[left..right].iter().fold((1, 0), |(a, b), &(c, d)| {
                    (a * c % modulus, (b * c + d) % modulus)
                });
                let compositions = [tree.fold(left..right), tree.fold_by_prefix(left..right)];
                let arguments = compositions
                    .into_iter()
                    .flat_map(|value| [0, 1, modulus - 1].map(move |x| (value, M::from(x))));
                for ((value, x), result) in
                    arguments.clone().zip(AffineFraction::apply_many(arguments))
                {
                    let expected = (a * x.inner() as u64 + b) % modulus;
                    assert_eq!(result.inner() as u64, expected);
                    assert_eq!(value.apply(x).inner() as u64, expected);
                }
                if step % 127 == 0 {
                    let cloned = tree.clone();
                    let expected = values.iter().fold(13, |x, &(a, b)| (a * x + b) % modulus);
                    assert_eq!(
                        cloned.fold_all().apply(M::from(13u32)).inner() as u64,
                        expected
                    );
                }
            }
        }
    }
}
