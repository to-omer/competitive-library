use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AddMulOperation,
    math::{ConvolveSteps, GcdConvolve},
    num::montgomery::MInt998244353,
};

#[verify::library_checker("gcd_convolution")]
pub fn gcd_convolution(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, mut a: [MInt998244353; n], mut b: [MInt998244353; n]);
    a.insert(0, Default::default());
    b.insert(0, Default::default());
    let c = GcdConvolve::<AddMulOperation<_>>::convolve(a, b);
    iter_print!(writer, @it &c[1..]);
}
