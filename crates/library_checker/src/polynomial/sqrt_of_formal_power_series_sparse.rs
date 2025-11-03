use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::Fps998244353, num::montgomery::MInt998244353, num::Zero as _};

#[verify::library_checker("sqrt_of_formal_power_series_sparse")]
pub fn sqrt_of_formal_power_series_sparse(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, k);
    let mut a = vec![MInt998244353::zero(); n];
    for _ in 0..k {
        scan!(scanner, i, a_i: MInt998244353);
        a[i] = a_i;
    }
    let f = Fps998244353::from_vec(a);
    if let Some(g) = f.sqrt(n) {
        iter_print!(writer, @it g.data);
    } else {
        iter_print!(writer, "-1");
    }
}
