use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AddMulOperation,
    math::{BitwisexorConvolve, ConvolveSteps},
    num::montgomery::MInt998244353,
};

#[verify::library_checker("bitwise_xor_convolution")]
pub fn bitwise_xor_convolution(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [MInt998244353; 1 << n], b: [MInt998244353; 1 << n]);
    let c = BitwisexorConvolve::<AddMulOperation<_>>::convolve(a, b);
    iter_print!(writer, @it c);
}
