use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AddMulOperation,
    algorithm::{BitwiseandConvolve, BitwiseorConvolve},
    math::ConvolveSteps,
    num::montgomery::MInt998244353,
};

#[verify::library_checker("bitwise_and_convolution")]
pub fn bitwise_and_convolution(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [MInt998244353; 1 << n], b: [MInt998244353; 1 << n]);
    let c = BitwiseandConvolve::<AddMulOperation<_>>::convolve(a, b);
    iter_print!(writer, @it c);
}

#[verify::library_checker("bitwise_and_convolution")]
pub fn bitwise_or_convolution(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, mut a: [MInt998244353; 1 << n], mut b: [MInt998244353; 1 << n]);
    a.reverse();
    b.reverse();
    let mut c = BitwiseorConvolve::<AddMulOperation<_>>::convolve(a, b);
    c.reverse();
    iter_print!(writer, @it c);
}
