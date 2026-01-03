use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::library_checker("compositional_inverse_of_formal_power_series")]
pub fn compositional_inverse_of_formal_power_series(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [MInt998244353; n]);
    let f = Fps998244353::from_vec(a);
    let g = f.compositional_inverse(n);
    iter_print!(writer, @it g.data);
}
