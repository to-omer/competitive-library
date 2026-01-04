use super::*;

impl<M> MIntBase for M
where
    M: MontgomeryReduction32,
{
    type Inner = u32;
    fn get_mod() -> Self::Inner {
        <Self as MontgomeryReduction32>::MOD
    }
    fn mod_zero() -> Self::Inner {
        0
    }
    fn mod_one() -> Self::Inner {
        Self::N1
    }
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        let z = x + y;
        let m = Self::get_mod();
        if z >= m { z - m } else { z }
    }
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        if x < y {
            x + Self::get_mod() - y
        } else {
            x - y
        }
    }
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        Self::reduce(x as u64 * y as u64)
    }
    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        Self::mod_mul(x, Self::mod_inv(y))
    }
    fn mod_neg(x: Self::Inner) -> Self::Inner {
        if x == 0 { 0 } else { Self::get_mod() - x }
    }
    fn mod_inv(x: Self::Inner) -> Self::Inner {
        let p = Self::get_mod() as i32;
        let (mut a, mut b) = (x as i32, p);
        let (mut u, mut x) = (1, 0);
        while a != 0 {
            let k = b / a;
            x -= k * u;
            b -= k * a;
            std::mem::swap(&mut x, &mut u);
            std::mem::swap(&mut b, &mut a);
        }
        Self::reduce((if x < 0 { x + p } else { x }) as u64 * Self::N3 as u64)
    }
    fn mod_inner(x: Self::Inner) -> Self::Inner {
        Self::reduce(x as u64)
    }
}
impl<M> MIntConvert<u32> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: u32) -> Self::Inner {
        Self::reduce(x as u64 * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> u32 {
        Self::reduce(x as u64)
    }
    fn mod_into() -> u32 {
        <Self as MIntBase>::get_mod()
    }
}
impl<M> MIntConvert<u64> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: u64) -> Self::Inner {
        Self::reduce(x % Self::get_mod() as u64 * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> u64 {
        Self::reduce(x as u64) as u64
    }
    fn mod_into() -> u64 {
        <Self as MIntBase>::get_mod() as u64
    }
}
impl<M> MIntConvert<usize> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: usize) -> Self::Inner {
        Self::reduce(x as u64 % Self::get_mod() as u64 * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> usize {
        Self::reduce(x as u64) as usize
    }
    fn mod_into() -> usize {
        <Self as MIntBase>::get_mod() as usize
    }
}
impl<M> MIntConvert<i32> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: i32) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as i32;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as i32) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> i32 {
        Self::reduce(x as u64) as i32
    }
    fn mod_into() -> i32 {
        <Self as MIntBase>::get_mod() as i32
    }
}
impl<M> MIntConvert<i64> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: i64) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as i64;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as i64) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> i64 {
        Self::reduce(x as u64) as i64
    }
    fn mod_into() -> i64 {
        <Self as MIntBase>::get_mod() as i64
    }
}
impl<M> MIntConvert<isize> for M
where
    M: MontgomeryReduction32,
{
    fn from(x: isize) -> Self::Inner {
        let x = x % <Self as MIntBase>::get_mod() as isize;
        let x = if x < 0 {
            (x + <Self as MIntBase>::get_mod() as isize) as u64
        } else {
            x as u64
        };
        Self::reduce(x * Self::N2 as u64)
    }
    fn into(x: Self::Inner) -> isize {
        Self::reduce(x as u64) as isize
    }
    fn mod_into() -> isize {
        <Self as MIntBase>::get_mod() as isize
    }
}
/// m is prime, n = 2^32
pub trait MontgomeryReduction32 {
    /// m
    const MOD: u32;
    /// (-m)^{-1} mod n
    const R: u32 = {
        let m = Self::MOD;
        let mut r = 0;
        let mut t = 0;
        let mut i = 0;
        while i < 32 {
            if t % 2 == 0 {
                t += m;
                r += 1 << i;
            }
            t /= 2;
            i += 1;
        }
        r
    };
    /// n^1 mod m
    const N1: u32 = ((1u64 << 32) % Self::MOD as u64) as _;
    /// n^2 mod m
    const N2: u32 = (Self::N1 as u64 * Self::N1 as u64 % Self::MOD as u64) as _;
    /// n^3 mod m
    const N3: u32 = (Self::N1 as u64 * Self::N2 as u64 % Self::MOD as u64) as _;
    /// n^{-1}x = (x + (xr mod n)m) / n
    fn reduce(x: u64) -> u32 {
        let m: u32 = Self::MOD;
        let r = Self::R;
        let mut x = ((x + r.wrapping_mul(x as u32) as u64 * m as u64) >> 32) as u32;
        if x >= m {
            x -= m;
        }
        x
    }
}
macro_rules! define_montgomery_reduction_32 {
    ($([$name:ident, $m:expr, $mint_name:ident $(,)?]),* $(,)?) => {
        $(
            pub enum $name {}
            impl MontgomeryReduction32 for $name {
                const MOD: u32 = $m;
            }
            pub type $mint_name = MInt<$name>;
        )*
    };
}
define_montgomery_reduction_32!(
    [Modulo998244353, 998_244_353, MInt998244353],
    [Modulo2113929217, 2_113_929_217, MInt2113929217],
    [Modulo1811939329, 1_811_939_329, MInt1811939329],
    [Modulo2013265921, 2_013_265_921, MInt2013265921],
);

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)] // SIMD intrinsics and raw pointers are confined here
pub mod simd32 {
    use std::arch::x86_64::*;

    #[target_feature(enable = "avx2")]
    unsafe fn my256_mullo_epu32(a: __m256i, b: __m256i) -> __m256i {
        _mm256_mullo_epi32(a, b)
    }

    #[target_feature(enable = "avx2")]
    unsafe fn my256_mulhi_epu32(a: __m256i, b: __m256i) -> __m256i {
        let a13 = _mm256_shuffle_epi32(a, 0xF5);
        let b13 = _mm256_shuffle_epi32(b, 0xF5);
        let prod02 = _mm256_mul_epu32(a, b);
        let prod13 = _mm256_mul_epu32(a13, b13);
        let t0 = _mm256_unpacklo_epi32(prod02, prod13);
        let t1 = _mm256_unpackhi_epi32(prod02, prod13);
        _mm256_unpackhi_epi64(t0, t1)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn montgomery_mul_256(
        a: __m256i,
        b: __m256i,
        r_vec: __m256i,
        mod_vec: __m256i,
    ) -> __m256i {
        let hi = my256_mulhi_epu32(a, b);
        let lo = my256_mullo_epu32(a, b);
        let lo = my256_mullo_epu32(lo, r_vec);
        let lo = my256_mulhi_epu32(lo, mod_vec);
        _mm256_sub_epi32(_mm256_add_epi32(hi, mod_vec), lo)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn add_mod_256(a: __m256i, b: __m256i, mod_vec: __m256i, sign: __m256i) -> __m256i {
        let sum = _mm256_add_epi32(a, b);
        let sum_x = _mm256_xor_si256(sum, sign);
        let mod_x = _mm256_xor_si256(mod_vec, sign);
        let gt = _mm256_cmpgt_epi32(sum_x, mod_x);
        let eq = _mm256_cmpeq_epi32(sum, mod_vec);
        let mask = _mm256_or_si256(gt, eq);
        let sub = _mm256_and_si256(mod_vec, mask);
        _mm256_sub_epi32(sum, sub)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn sub_mod_256(a: __m256i, b: __m256i, mod_vec: __m256i, sign: __m256i) -> __m256i {
        let diff = _mm256_sub_epi32(a, b);
        let a_x = _mm256_xor_si256(a, sign);
        let b_x = _mm256_xor_si256(b, sign);
        let mask = _mm256_cmpgt_epi32(b_x, a_x);
        let add = _mm256_and_si256(mod_vec, mask);
        _mm256_add_epi32(diff, add)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn montgomery_mul_256_canon(
        a: __m256i,
        b: __m256i,
        r_vec: __m256i,
        mod_vec: __m256i,
        sign: __m256i,
    ) -> __m256i {
        let x = montgomery_mul_256(a, b, r_vec, mod_vec);
        add_mod_256(x, _mm256_setzero_si256(), mod_vec, sign)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn montgomery_add_256(
        a: __m256i,
        b: __m256i,
        mod2_vec: __m256i,
        sign: __m256i,
    ) -> __m256i {
        let sum = _mm256_add_epi32(a, b);
        let sum_x = _mm256_xor_si256(sum, sign);
        let mod_x = _mm256_xor_si256(mod2_vec, sign);
        let gt = _mm256_cmpgt_epi32(sum_x, mod_x);
        let eq = _mm256_cmpeq_epi32(sum, mod2_vec);
        let mask = _mm256_or_si256(gt, eq);
        let sub = _mm256_and_si256(mod2_vec, mask);
        _mm256_sub_epi32(sum, sub)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn montgomery_sub_256(
        a: __m256i,
        b: __m256i,
        mod2_vec: __m256i,
        sign: __m256i,
    ) -> __m256i {
        let diff = _mm256_sub_epi32(a, b);
        let a_x = _mm256_xor_si256(a, sign);
        let b_x = _mm256_xor_si256(b, sign);
        let mask = _mm256_cmpgt_epi32(b_x, a_x);
        let add = _mm256_and_si256(mod2_vec, mask);
        _mm256_add_epi32(diff, add)
    }

    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    unsafe fn my512_mullo_epu32(a: __m512i, b: __m512i) -> __m512i {
        _mm512_mullo_epi32(a, b)
    }

    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    unsafe fn my512_mulhi_epu32(a: __m512i, b: __m512i) -> __m512i {
        let a13 = _mm512_shuffle_epi32(a, 0xF5);
        let b13 = _mm512_shuffle_epi32(b, 0xF5);
        let prod02 = _mm512_mul_epu32(a, b);
        let prod13 = _mm512_mul_epu32(a13, b13);
        let t0 = _mm512_unpacklo_epi32(prod02, prod13);
        let t1 = _mm512_unpackhi_epi32(prod02, prod13);
        _mm512_unpackhi_epi64(t0, t1)
    }

    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn montgomery_mul_512(
        a: __m512i,
        b: __m512i,
        r_vec: __m512i,
        mod_vec: __m512i,
    ) -> __m512i {
        let hi = my512_mulhi_epu32(a, b);
        let lo = my512_mullo_epu32(a, b);
        let lo = my512_mullo_epu32(lo, r_vec);
        let lo = my512_mulhi_epu32(lo, mod_vec);
        _mm512_sub_epi32(_mm512_add_epi32(hi, mod_vec), lo)
    }

    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn add_mod_512(a: __m512i, b: __m512i, mod_vec: __m512i) -> __m512i {
        let sum = _mm512_add_epi32(a, b);
        let mask = !_mm512_cmp_epu32_mask(sum, mod_vec, _MM_CMPINT_LT);
        _mm512_mask_sub_epi32(sum, mask, sum, mod_vec)
    }

    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn sub_mod_512(a: __m512i, b: __m512i, mod_vec: __m512i) -> __m512i {
        let diff = _mm512_sub_epi32(a, b);
        let mask = _mm512_cmp_epu32_mask(a, b, _MM_CMPINT_LT);
        _mm512_mask_add_epi32(diff, mask, diff, mod_vec)
    }

    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn montgomery_mul_512_canon(
        a: __m512i,
        b: __m512i,
        r_vec: __m512i,
        mod_vec: __m512i,
    ) -> __m512i {
        let x = montgomery_mul_512(a, b, r_vec, mod_vec);
        add_mod_512(x, _mm512_setzero_si512(), mod_vec)
    }

    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn montgomery_add_512(a: __m512i, b: __m512i, mod2_vec: __m512i) -> __m512i {
        let sum = _mm512_add_epi32(a, b);
        let mask = !_mm512_cmp_epu32_mask(sum, mod2_vec, _MM_CMPINT_LT);
        _mm512_mask_sub_epi32(sum, mask, sum, mod2_vec)
    }

    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn montgomery_sub_512(a: __m512i, b: __m512i, mod2_vec: __m512i) -> __m512i {
        let diff = _mm512_sub_epi32(a, b);
        let mask = _mm512_cmp_epu32_mask(a, b, _MM_CMPINT_LT);
        _mm512_mask_add_epi32(diff, mask, diff, mod2_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::montgomery::MInt998244353 as M;
    use crate::tools::Xorshift;

    #[test]
    fn test_mint998244353() {
        let mut rng = Xorshift::default();
        const Q: usize = 1000;
        assert_eq!(0, MInt998244353::zero().inner());
        assert_eq!(1, MInt998244353::one().inner());
        assert_eq!(
            Modulo998244353::reduce(Modulo998244353::N3 as u64),
            Modulo998244353::N2
        );
        assert_eq!(
            Modulo998244353::reduce(Modulo998244353::N2 as u64),
            Modulo998244353::N1
        );
        assert_eq!(Modulo998244353::reduce(Modulo998244353::N1 as u64), 1);
        for _ in 0..Q {
            let x = rng.random(..MInt998244353::get_mod());
            assert_eq!(x, MInt998244353::new(x).inner());
            assert_eq!((-M::new(x)).inner(), (-MInt998244353::new(x)).inner());
            assert_eq!(x, MInt998244353::new(x).inv().inv().inner());
            assert_eq!(M::new(x).inv().inner(), MInt998244353::new(x).inv().inner());
        }

        for _ in 0..Q {
            let x = rng.random(..MInt998244353::get_mod());
            let y = rng.random(..MInt998244353::get_mod());
            assert_eq!(
                (M::new(x) + M::new(y)).inner(),
                (MInt998244353::new(x) + MInt998244353::new(y)).inner()
            );
            assert_eq!(
                (M::new(x) - M::new(y)).inner(),
                (MInt998244353::new(x) - MInt998244353::new(y)).inner()
            );
            assert_eq!(
                (M::new(x) * M::new(y)).inner(),
                (MInt998244353::new(x) * MInt998244353::new(y)).inner()
            );
            assert_eq!(
                (M::new(x) / M::new(y)).inner(),
                (MInt998244353::new(x) / MInt998244353::new(y)).inner()
            );
            assert_eq!(
                M::new(x).pow(y as usize).inner(),
                MInt998244353::new(x).pow(y as usize).inner()
            );
        }

        for _ in 0..Q {
            let x = rng.rand64();
            assert_eq!(
                M::from(x as u32).inner(),
                MInt998244353::from(x as u32).inner()
            );
            assert_eq!(M::from(x).inner(), MInt998244353::from(x).inner());
            assert_eq!(
                M::from(x as usize).inner(),
                MInt998244353::from(x as usize).inner()
            );
            assert_eq!(
                M::from(x as i32).inner(),
                MInt998244353::from(x as i32).inner()
            );
            assert_eq!(
                M::from(x as i64).inner(),
                MInt998244353::from(x as i64).inner()
            );
            assert_eq!(
                M::from(x as isize).inner(),
                MInt998244353::from(x as isize).inner()
            );
        }
    }
}
