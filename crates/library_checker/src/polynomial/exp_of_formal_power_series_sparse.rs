use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::Fps998244353, num::Zero as _, num::montgomery::MInt998244353};

#[verify::library_checker("exp_of_formal_power_series_sparse")]
pub fn exp_of_formal_power_series_sparse(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, k);
    let mut a = vec![MInt998244353::zero(); n];
    for _ in 0..k {
        scan!(scanner, i, a_i: MInt998244353);
        a[i] = a_i;
    }
    let f = Fps998244353::from_vec(a);
    let g = f.exp(n);
    iter_print!(writer, @it g.data);
}
