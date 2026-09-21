use super::{
    AddMulOperation, BlackBoxMatrix, BlackBoxMatrixImpl, ConvolveSteps, DotProduct,
    FormalPowerSeries, Fps, MInt, MIntConvert, MIntDotProduct, One, Xorshift, Zero,
};

pub trait BlackBoxMIntMatrix<M>: BlackBoxMatrix<AddMulOperation<MInt<M>>>
where
    M: MIntDotProduct<Inner = u32>
        + MIntConvert<u32>
        + MIntConvert<u64>
        + MIntConvert<usize>
        + MIntConvert<isize>,
{
    fn minimal_polynomial(&self) -> Vec<MInt<M>> {
        assert_eq!(self.shape().0, self.shape().1);
        let n = self.shape().0;
        let mut rng = Xorshift::new();
        let b: Vec<MInt<M>> = (0..n).map(|_| MInt::from(rng.rand64())).collect();
        let u: Vec<MInt<M>> = (0..n).map(|_| MInt::from(rng.rand64())).collect();
        let a: Vec<MInt<M>> = (0..2 * n)
            .scan(b, |b, _| {
                let a = MInt::dot_product(b, &u);
                *b = self.apply(b);
                Some(a)
            })
            .collect();
        let polynomial: Fps<M> = FormalPowerSeries::berlekamp_massey(&a);
        let mut p = polynomial.data;
        p.reverse();
        p
    }

    fn apply_pow<C>(&self, mut b: Vec<MInt<M>>, k: usize) -> Vec<MInt<M>>
    where
        C: ConvolveSteps<T = Vec<MInt<M>>>,
    {
        assert_eq!(self.shape().0, self.shape().1);
        assert_eq!(self.shape().1, b.len());
        let n = self.shape().0;
        let p = self.minimal_polynomial();
        let polynomial: FormalPowerSeries<MInt<M>, C> = FormalPowerSeries::from_vec(p);
        let f = polynomial.pow_mod(k);
        let mut res = vec![MInt::zero(); n];
        for f in f {
            for j in 0..n {
                res[j] += f * b[j];
            }
            b = self.apply(&b);
        }
        res
    }

    fn black_box_determinant(&self) -> MInt<M> {
        assert_eq!(self.shape().0, self.shape().1);
        let n = self.shape().0;
        let mut rng = Xorshift::new();
        let d: Vec<MInt<M>> = (0..n).map(|_| MInt::from(rng.rand64())).collect();
        let det_d = d.iter().fold(MInt::one(), |s, x| s * x);
        let ad: BlackBoxMatrixImpl<AddMulOperation<MInt<M>>, _> =
            BlackBoxMatrixImpl::new(self.shape(), |v: &[MInt<M>]| {
                let mut w = self.apply(v);
                for (w, d) in w.iter_mut().zip(&d) {
                    *w *= d;
                }
                w
            });
        let p = ad.minimal_polynomial();
        let det_ad = if n % 2 == 0 { p[0] } else { -p[0] };
        det_ad / det_d
    }

    fn black_box_linear_equation(&self, mut b: Vec<MInt<M>>) -> Option<Vec<MInt<M>>> {
        assert_eq!(self.shape().0, self.shape().1);
        assert_eq!(self.shape().1, b.len());
        let n = self.shape().0;
        let p = self.minimal_polynomial();
        if p.is_empty() || p[0].is_zero() {
            return None;
        }
        let p0_inv = p[0].inv();
        let mut x = vec![MInt::zero(); n];
        for p in p.into_iter().skip(1) {
            let p = -p * p0_inv;
            for i in 0..n {
                x[i] += p * b[i];
            }
            b = self.apply(&b);
        }
        Some(x)
    }
}

impl<M, B> BlackBoxMIntMatrix<M> for B
where
    M: MIntDotProduct<Inner = u32>
        + MIntConvert<u32>
        + MIntConvert<u64>
        + MIntConvert<usize>
        + MIntConvert<isize>,
    B: BlackBoxMatrix<AddMulOperation<MInt<M>>>,
{
}
