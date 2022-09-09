pub trait ConvolveSteps {
    type T;
    type F;
    fn length(t: &Self::T) -> usize;
    fn transform(t: Self::T, len: usize) -> Self::F;
    fn inverse_transform(f: Self::F, len: usize) -> Self::T;
    fn multiply(f: &mut Self::F, g: &Self::F);
    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        let len = (Self::length(&a) + Self::length(&b)).saturating_sub(1);
        let mut a = Self::transform(a, len);
        let b = Self::transform(b, len);
        Self::multiply(&mut a, &b);
        Self::inverse_transform(a, len)
    }
}
