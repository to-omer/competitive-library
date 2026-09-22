use competitive::prelude::*;
use competitive::{math::Fps998244353, num::montgomery::MInt998244353 as M};

#[verify::library_checker("division_of_polynomials")]
pub fn division_of_polynomials(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, f: [M; n], g: [M; m]);
    let f = Fps998244353::from_vec(f);
    let g = Fps998244353::from_vec(g);
    let (q, r) = f.div_rem(g);
    pp!(q.length(), r.length(); @it q.data; @it r.data);
}
