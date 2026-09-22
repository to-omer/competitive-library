use competitive::prelude::*;
use competitive::{math::Fps998244353, num::montgomery::MInt998244353 as M};

#[verify::library_checker("sqrt_of_formal_power_series")]
pub fn sqrt_of_formal_power_series(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [M; n]);
    let f = Fps998244353::from_vec(a);
    if let Some(g) = f.sqrt(n) {
        pp!(@it g.data);
    } else {
        pp!("-1");
    }
}
