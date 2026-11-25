use super::{DotProduct, MInt, MIntBase};

pub trait MIntDotProduct: MIntBase + Sized {
    fn try_matrix_product(
        _a: &[Vec<MInt<Self>>],
        _b: &[Vec<MInt<Self>>],
    ) -> Option<Vec<Vec<MInt<Self>>>> {
        None
    }

    /// Returns the inverse of two for odd moduli greater than one.
    fn inverse_two() -> Option<MInt<Self>>;

    fn dot_product(x: &[MInt<Self>], y: &[MInt<Self>]) -> MInt<Self> {
        assert_eq!(x.len(), y.len());
        x.iter()
            .zip(y)
            .fold(MInt::new_unchecked(Self::mod_zero()), |sum, (&x, &y)| {
                sum + x * y
            })
    }

    /// Computes the inner product of `x` and the reversed `y`.
    fn dot_product_rev(x: &[MInt<Self>], y: &[MInt<Self>]) -> MInt<Self> {
        assert_eq!(x.len(), y.len());
        const CHUNK_SIZE: usize = if cfg!(all(
            target_feature = "avx2",
            not(target_feature = "avx512f")
        )) {
            16
        } else {
            8
        };
        let (x, x_rem) = x.as_chunks::<CHUNK_SIZE>();
        let (y_rem, y) = y.as_rchunks::<CHUNK_SIZE>();
        let mut accum = [MInt::new_unchecked(Self::mod_zero()); CHUNK_SIZE];
        for (x, y) in x.iter().zip(y.iter().rev()) {
            for i in 0..CHUNK_SIZE {
                accum[i] += x[i] * y[CHUNK_SIZE - 1 - i];
            }
        }
        let mut sum = accum.into_iter().sum();
        for (&x, &y) in x_rem.iter().zip(y_rem.iter().rev()) {
            sum += x * y;
        }
        sum
    }

    /// Computes one inner product per column.
    fn dot_product_batch<const N: usize>(
        x: &[[MInt<Self>; N]],
        y: &[[MInt<Self>; N]],
    ) -> [MInt<Self>; N] {
        assert_eq!(x.len(), y.len());
        let mut sums = std::array::from_fn(|_| MInt::new_unchecked(Self::mod_zero()));
        for (x, y) in x.iter().zip(y) {
            for i in 0..N {
                sums[i] += x[i] * y[i];
            }
        }
        sums
    }

    fn add_scaled_assign(x: &mut [MInt<Self>], y: &[MInt<Self>], a: &MInt<Self>) {
        assert_eq!(x.len(), y.len());
        for (x, y) in x.iter_mut().zip(y) {
            *x += *a * *y;
        }
    }
}

impl<M> DotProduct for MInt<M>
where
    M: MIntDotProduct,
{
    #[inline]
    fn try_matrix_product(a: &[Vec<Self>], b: &[Vec<Self>]) -> Option<Vec<Vec<Self>>> {
        M::try_matrix_product(a, b)
    }

    #[inline]
    fn inverse_two() -> Option<Self> {
        M::inverse_two()
    }

    #[inline]
    fn dot_product(x: &[Self], y: &[Self]) -> Self {
        assert_eq!(x.len(), y.len());
        M::dot_product(x, y)
    }

    #[inline]
    fn dot_product_batch<const N: usize>(x: &[[Self; N]], y: &[[Self; N]]) -> [Self; N] {
        M::dot_product_batch(x, y)
    }

    #[inline]
    fn add_scaled_assign(x: &mut [Self], y: &[Self], a: &Self) {
        M::add_scaled_assign(x, y, a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::{MIntConvert, mint_basic, montgomery};

    #[test]
    fn inverse_two() {
        fn check<M>()
        where
            M: MIntDotProduct + MIntConvert<u64>,
        {
            let modulus = M::mod_into();
            let inverse = MInt::<M>::inverse_two().map(u64::from);
            if modulus > 1 && modulus % 2 == 1 {
                let value = inverse.unwrap();
                assert!(value < modulus);
                assert_eq!(2 * u128::from(value) % u128::from(modulus), 1);
            } else {
                assert_eq!(inverse, None);
            }
        }

        check::<mint_basic::Modulo2>();
        check::<mint_basic::Modulo998244353>();
        check::<mint_basic::Modulo1000000007>();
        check::<mint_basic::Modulo1000000009>();
        for modulus in [1, 2, 3, 9, u32::MAX - 1, u32::MAX] {
            mint_basic::DynModuloU32::set_mod(modulus);
            check::<mint_basic::DynModuloU32>();
        }
        for modulus in [1, 2, 3, 9, u64::MAX - 1, u64::MAX] {
            mint_basic::DynModuloU64::set_mod(modulus);
            check::<mint_basic::DynModuloU64>();
        }
        mint_basic::DynModuloU32::set_mod(1_000_000_007);
        mint_basic::DynModuloU64::set_mod(1_000_000_007);

        enum Modulo<const P: u32> {}
        impl<const P: u32> montgomery::MontgomeryReduction32 for Modulo<P> {
            const MOD: u32 = P;
        }
        check::<Modulo<1>>();
        check::<Modulo<2>>();
        check::<Modulo<9>>();
        check::<montgomery::Modulo998244353>();
        check::<Modulo<2_147_483_647>>();
    }
}
