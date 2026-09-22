use competitive::prelude::*;
use competitive::{math::Fps998244353, num::montgomery::MInt998244353 as M};

#[verify::library_checker("find_linear_recurrence")]
pub fn find_linear_recurrence(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [M; n]);
    let c = Fps998244353::berlekamp_massey(&a);
    pp!(c.length() - 1; @it c.iter().skip(1).map(|x| -x));
}
