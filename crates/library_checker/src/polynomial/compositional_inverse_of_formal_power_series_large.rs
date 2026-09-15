use competitive::prelude::*;
use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::library_checker("compositional_inverse_of_formal_power_series_large")]
pub fn compositional_inverse_of_formal_power_series_large(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [MInt998244353; n]);
    let f = Fps998244353::from_vec(a);
    let g = f.compositional_inverse(n);
    pp!(@it g.data);
}
