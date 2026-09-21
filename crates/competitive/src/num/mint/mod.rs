//! modint

#[cfg(target_arch = "x86_64")]
use crate::tools::{advise_huge_pages, avx512_enabled, avx512_supported};
use crate::{
    algebra::DotProduct,
    num::{BarrettReduction, One, Zero},
    tools::{FastOutput, FastPrint, RandomSpec, Scan, ScanSource, SerdeByteStr, Xorshift},
};

#[codesnip::entry("MIntBase", include("scanner", "zero_one"))]
pub use mint_base::{MInt, MIntBase, MIntConvert};

#[cfg_attr(nightly, codesnip::entry("MIntBase"))]
mod mint_base;

#[cfg_attr(
    nightly,
    codesnip::entry("MInt", include("MIntBase", "BarrettReduction"))
)]
pub mod mint_basic;

#[cfg_attr(nightly, codesnip::entry("montgomery", include("MIntBase")))]
pub mod montgomery;

#[codesnip::entry("MIntDotProduct")]
pub use mint_dot_product::MIntDotProduct;
#[cfg_attr(
    nightly,
    codesnip::entry(when("MInt", "MIntDotProduct"), include("simd_matrix", "avx_helper"))
)]
mod mint_basic_dot_product;
#[cfg_attr(
    nightly,
    codesnip::entry("MIntDotProduct", include("MIntBase", "ring"))
)]
mod mint_dot_product;
#[cfg_attr(
    nightly,
    codesnip::entry(
        when("montgomery", "MIntDotProduct"),
        include("simd_matrix", "montgomery_simd", "avx_helper")
    )
)]
mod montgomery_dot_product;
#[cfg(target_arch = "x86_64")]
#[cfg_attr(nightly, codesnip::entry("montgomery_simd"))]
pub mod montgomery_simd;

#[cfg(target_arch = "x86_64")]
#[cfg_attr(
    nightly,
    codesnip::entry("simd_matrix", include("MIntBase", "avx_helper", "_huge_pages"))
)]
mod simd_matrix;

#[codesnip::entry(when("MIntBase", "fastio"))]
impl<M> FastPrint for MInt<M>
where
    M: MIntBase<Inner: FastPrint>,
{
    #[inline]
    fn fast_print<W: std::io::Write>(&self, writer: &mut FastOutput<W>) {
        self.inner().fast_print(writer);
    }
}

#[codesnip::entry(when("MIntBase", "coding"))]
impl<M> SerdeByteStr for MInt<M>
where
    M: MIntBase<Inner: SerdeByteStr>,
{
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.inner().serialize(buf)
    }

    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        Self::new_unchecked(M::Inner::deserialize(iter))
    }
}

#[cfg_attr(nightly, codesnip::entry(when("MIntBase", "random_generator")))]
mod random_spec {
    use super::*;
    use std::ops::{RangeFull, RangeTo};

    impl<M> RandomSpec<MInt<M>> for RangeFull
    where
        M: MIntBase,
        RangeTo<M::Inner>: RandomSpec<M::Inner>,
    {
        fn rand(&self, rng: &mut Xorshift) -> MInt<M> {
            MInt::<M>::new_unchecked(rng.random(..M::get_mod()))
        }
    }
}
