use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    math::Fps998244353, math::MemorizedFactorial, num::montgomery::MInt998244353,
};

#[verify::library_checker("polynomial_taylor_shift")]
pub fn polynomial_taylor_shift(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, c: MInt998244353, a: [MInt998244353; n]);
    let f = MemorizedFactorial::new(n);
    let a = Fps998244353::from_vec(a);
    let res = a.taylor_shift(c, &f);
    iter_print!(writer, @it res);
}
