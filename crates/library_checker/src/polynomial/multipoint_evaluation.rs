use competitive::prelude::*;
use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::library_checker("multipoint_evaluation")]
pub fn multipoint_evaluation(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, c: [MInt998244353; n], p: [MInt998244353; m]);
    let f = Fps998244353::from_vec(c);
    let res = f.multipoint_evaluation(&p);
    pp!(@it res);
}
