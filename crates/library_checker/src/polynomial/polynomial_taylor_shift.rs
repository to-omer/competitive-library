use competitive::prelude::*;
use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::library_checker("polynomial_taylor_shift")]
pub fn polynomial_taylor_shift(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, c: MInt998244353, a: [MInt998244353; n]);
    let a = Fps998244353::from_vec(a);
    let res = a.taylor_shift(c);
    pp!(@it res);
}
