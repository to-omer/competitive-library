use competitive::prelude::*;
use competitive::{math::Fps998244353, num::Zero as _, num::montgomery::MInt998244353 as M};

#[verify::library_checker("sqrt_of_formal_power_series_sparse")]
pub fn sqrt_of_formal_power_series_sparse(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, k);
    let mut a = vec![M::zero(); n];
    for _ in 0..k {
        sc!(i, a_i: M);
        a[i] = a_i;
    }
    let f = Fps998244353::from_vec(a);
    if let Some(g) = f.sqrt(n) {
        pp!(@it g.data);
    } else {
        pp!("-1");
    }
}
