pub trait ConvolveSteps {
    /// Whether transform multiplication computes modulo x^n - 1 in the coefficient ring.
    const CYCLIC: bool = false;

    type T;
    type F;
    fn length(t: &Self::T) -> usize;
    fn transform(t: Self::T, len: usize) -> Self::F;
    fn inverse_transform(f: Self::F, len: usize) -> Self::T;
    fn multiply(f: &mut Self::F, g: &Self::F);
    fn square(t: Self::T, len: usize) -> Self::T
    where
        Self::T: Clone,
    {
        let mut f = Self::transform(t.clone(), len);
        let g = Self::transform(t, len);
        Self::multiply(&mut f, &g);
        Self::inverse_transform(f, len)
    }
    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        let len = (Self::length(&a) + Self::length(&b)).saturating_sub(1);
        let mut a = Self::transform(a, len);
        let b = Self::transform(b, len);
        Self::multiply(&mut a, &b);
        Self::inverse_transform(a, len)
    }
}
