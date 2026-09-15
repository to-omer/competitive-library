use competitive::prelude::*;
use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::library_checker("division_of_polynomials")]
pub fn division_of_polynomials(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, f: [MInt998244353; n], g: [MInt998244353; m]);
    let f = Fps998244353::from_vec(f);
    let g = Fps998244353::from_vec(g);
    let (q, r) = f.div_rem(g);
    pp!(q.length(), r.length());
    pp!(@it q.data);
    pp!(@it r.data);
}
