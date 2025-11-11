use super::{magma::*, operations::BitXorOperation, ring::*};

#[codesnip::entry("Gf2_63")]
pub use self::gf2_63::Gf2_63;
#[codesnip::entry("Gf2_63", include("BitXorOperation", "ring"))]
mod gf2_63 {
    use super::*;
    pub enum Gf2_63 {}
    impl Gf2_63 {
        pub const MOD: u64 = 1 << 63;
    }
    impl Magma for Gf2_63 {
        type T = u64;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            use core::arch::x86_64::{_mm_clmulepi64_si128, _mm_extract_epi64, _mm_set_epi64x};
            unsafe {
                let a = _mm_set_epi64x(0, *x as i64);
                let b = _mm_set_epi64x(0, *y as i64);
                let c = _mm_clmulepi64_si128(a, b, 0);
                let lo = _mm_extract_epi64(c, 0) as u64;
                let hi = _mm_extract_epi64(c, 1) as u64;
                let hi = (hi << 1) | (lo >> 63);
                let lo = lo & !(!(0u64) << 63);
                lo ^ hi ^ (hi << 1)
            }
            // {
            //     let x = *x as u128;
            //     let mut bit = 0u128;
            //     for i in 0u32..63 {
            //         if y >> i & 1 != 0 {
            //             bit ^= x << i;
            //         }
            //     }
            //     let hi = (bit >> 64) as u64;
            //     let lo = (bit & ((1 << 64) - 1)) as u64;
            //     let hi = hi << 1 | lo >> 63;
            //     let lo = lo & !(!(0u64) << 63);
            //     lo ^ hi ^ (hi << 1)
            // }
        }
    }
    impl Unital for Gf2_63 {
        fn unit() -> Self::T {
            1
        }
    }
    impl Associative for Gf2_63 {}
    impl Commutative for Gf2_63 {}
    impl SemiRing for Gf2_63 {
        type T = u64;
        type Additive = BitXorOperation<u64>;
        type Multiplicative = Self;
    }
}

#[codesnip::entry("Mersenne61")]
pub use self::mersenne61::Mersenne61;
#[codesnip::entry("Mersenne61", include("ring"))]
mod mersenne61 {
    use super::*;
    pub enum Mersenne61Add {}
    impl Magma for Mersenne61Add {
        type T = u64;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let mut z = x + y;
            if z >= Mersenne61::MOD {
                z -= Mersenne61::MOD
            }
            z
        }
    }
    impl Unital for Mersenne61Add {
        fn unit() -> Self::T {
            0
        }
    }
    impl Associative for Mersenne61Add {}
    impl Commutative for Mersenne61Add {}
    impl Invertible for Mersenne61Add {
        fn inverse(x: &Self::T) -> Self::T {
            if *x == 0 {
                0
            } else {
                Mersenne61::MOD - x
            }
        }
    }

    pub enum Mersenne61 {}
    impl Mersenne61 {
        pub const MOD: u64 = (1 << 61) - 1;
    }
    impl Magma for Mersenne61 {
        type T = u64;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let z = *x as u128 * *y as u128;
            Mersenne61Add::operate(&((z >> 61) as _), &(z as u64 & Self::MOD))
        }
    }
    impl Unital for Mersenne61 {
        fn unit() -> Self::T {
            1
        }
    }
    impl Associative for Mersenne61 {}
    impl Commutative for Mersenne61 {}
    impl SemiRing for Mersenne61 {
        type T = u64;
        type Additive = Mersenne61Add;
        type Multiplicative = Self;
    }
}
