use super::{DotProduct, MInt, MIntBase};

pub trait MIntDotProduct: MIntBase + Sized {
    fn try_matrix_product(
        _a: &[Vec<MInt<Self>>],
        _b: &[Vec<MInt<Self>>],
    ) -> Option<Vec<Vec<MInt<Self>>>> {
        None
    }

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
    fn dot_product(x: &[Self], y: &[Self]) -> Self {
        assert_eq!(x.len(), y.len());
        M::dot_product(x, y)
    }

    #[inline]
    fn add_scaled_assign(x: &mut [Self], y: &[Self], a: &Self) {
        M::add_scaled_assign(x, y, a);
    }
}
