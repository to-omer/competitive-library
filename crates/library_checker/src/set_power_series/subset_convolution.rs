use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AddMulOperation,
    math::{ConvolveSteps, SubsetConvolve},
    num::montgomery::MInt998244353,
};

#[verify::library_checker("subset_convolution")]
pub fn subset_convolution(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [MInt998244353; 1 << n], b: [MInt998244353; 1 << n]);
    let c = SubsetConvolve::<AddMulOperation<_>>::convolve(a, b);
    iter_print!(writer, @it c);
}
