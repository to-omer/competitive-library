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
            #[cfg(target_arch = "x86_64")]
            {
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
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                let x = *x as u128;
                let mut bit = 0u128;
                for i in 0u32..63 {
                    if y >> i & 1 != 0 {
                        bit ^= x << i;
                    }
                }
                let hi = (bit >> 64) as u64;
                let lo = (bit & ((1 << 64) - 1)) as u64;
                let hi = (hi << 1) | (lo >> 63);
                let lo = lo & !(!(0u64) << 63);
                lo ^ hi ^ (hi << 1)
            }
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
    #[cfg(verus_keep_ghost)]
    use vstd::{arithmetic::div_mod::*, prelude::*};

    pub enum Mersenne61Add {}
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    impl Magma for Mersenne61Add {
        type T = u64;
        #[cfg_attr(verus_keep_ghost, verus_spec(result =>
            requires (*x as int) + (*y as int) < 2 * (Mersenne61::MOD as int),
            ensures
                result < Mersenne61::MOD,
                result == ((*x as int) + (*y as int)) % (Mersenne61::MOD as int),
        ))]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let mut z = x + y;
            if z >= Mersenne61::MOD {
                z -= Mersenne61::MOD
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                lemma_fundamental_div_mod_converse(
                    (*x as int) + (*y as int), (Mersenne61::MOD as int),
                    if (*x as int) + (*y as int) >= Mersenne61::MOD { 1int } else { 0int },
                    z as int,
                );
                #[verifier::proof_note("For all u64 operands with sum below twice 2^61 - 1, addition returns the canonical residue modulo 2^61 - 1 without overflow")]
                assert(z < Mersenne61::MOD && z == ((*x as int) + (*y as int)) % (Mersenne61::MOD as int));
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
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    impl Invertible for Mersenne61Add {
        #[cfg_attr(verus_keep_ghost, verus_spec(result =>
            requires *x < Mersenne61::MOD,
            ensures result < Mersenne61::MOD, ((*x as int) + result as int) % (Mersenne61::MOD as int) == 0,
        ))]
        fn inverse(x: &Self::T) -> Self::T {
            #[cfg(verus_keep_ghost)]
            proof! {
                let inverse = if *x == 0 { 0int } else { (Mersenne61::MOD as int) - (*x as int) };
                lemma_small_mod(0, Mersenne61::MOD as nat);
                lemma_mod_self_0((Mersenne61::MOD as int));
                #[verifier::proof_note("For every residue below 2^61 - 1, additive inversion returns a canonical residue whose sum with the input is zero modulo 2^61 - 1")]
                assert(0 <= inverse < Mersenne61::MOD && ((*x as int) + inverse) % (Mersenne61::MOD as int) == 0);
            }
            if *x == 0 { 0 } else { Mersenne61::MOD - x }
        }
    }

    pub enum Mersenne61 {}
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    impl Mersenne61 {
        pub const MOD: u64 = 0x1fff_ffff_ffff_ffff; // 2^61 - 1
    }
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    impl Magma for Mersenne61 {
        type T = u64;
        #[cfg_attr(verus_keep_ghost, verus_spec(result =>
            requires *x < Self::MOD, *y < Self::MOD,
            ensures result < Self::MOD, result == ((*x as int) * (*y as int)) % (Self::MOD as int),
        ))]
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            #[cfg(verus_keep_ghost)]
            proof! {
                assert(Self::MOD == (1u64 << 61) - 1) by(bit_vector);
                assert(0 <= (*x as int) * (*y as int) < (Self::MOD as int) * (Self::MOD as int))
                    by(nonlinear_arith) requires *x < Self::MOD, *y < Self::MOD, *x >= 0, *y >= 0;
                assert((*x as int) * (*y as int) <= u128::MAX) by(nonlinear_arith)
                    requires *x < Self::MOD, *y < Self::MOD, Self::MOD == 0x1fff_ffff_ffff_ffff;
            }
            let z = *x as u128 * *y as u128;
            #[cfg(verus_keep_ghost)]
            proof! {
                let radix = ((Self::MOD as int) + 1) as u128;
                assert(z >> 61 < (Self::MOD as u128)) by(bit_vector)
                    requires z < (Self::MOD as u128) * (Self::MOD as u128);
                assert(z >> 61 == z / radix) by(bit_vector)
                    requires radix == ((Self::MOD as int) + 1) as u128;
                let hi = (z >> 61) as u64;
                let lo = (#[verifier::truncate] (z as u64)) & Self::MOD;
                assert(lo as u128 == z % radix) by(bit_vector)
                    requires lo == (#[verifier::truncate] (z as u64)) & Self::MOD,
                        radix == ((Self::MOD as int) + 1) as u128;
                lemma_fundamental_div_mod(z as int, radix as int);
                assert(hi < Self::MOD && lo <= Self::MOD);
                assert(z == hi as int * radix as int + lo) by(nonlinear_arith)
                    requires z == radix as int * (z / radix) + z % radix,
                        hi == z / radix, lo == z % radix;
                lemma_mod_self_0((Self::MOD as int));
                lemma_small_mod(0, Self::MOD as nat);
                vstd::arithmetic::mul::lemma_mul_basics(hi as int);
                lemma_mul_mod_noop_right(hi as int, (Self::MOD as int), (Self::MOD as int));
                lemma_add_mod_noop(hi as int * (Self::MOD as int), hi as int + lo as int, (Self::MOD as int));
                assert(z == hi as int * (Self::MOD as int) + (hi as int + lo as int))
                    by(nonlinear_arith) requires z == hi as int * radix as int + lo,
                        radix == (Self::MOD as int) + 1;
                #[verifier::proof_note("For all residues below 2^61 - 1, the u128 product fits and folding its high 61-bit part into its low part preserves the residue; final addition yields the canonical product modulo 2^61 - 1")]
                assert(z as int % (Self::MOD as int) == (hi as int + lo as int) % (Self::MOD as int));
            }
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

    #[cfg(verus_keep_ghost)]
    mod verification {
        use super::*;

        verus! {
            // Verus represents the uninhabited operation markers as opaque types.
            #[verifier::external_type_specification]
            #[verifier::external_body]
            pub struct ExMersenne61Add(Mersenne61Add);

            #[verifier::external_type_specification]
            #[verifier::external_body]
            pub struct ExMersenne61(Mersenne61);
        }
    }
}
