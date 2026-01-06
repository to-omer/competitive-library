use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::library_checker("division_of_polynomials")]
pub fn division_of_polynomials(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, f: [MInt998244353; n], g: [MInt998244353; m]);
    let f = Fps998244353::from_vec(f);
    let g = Fps998244353::from_vec(g);
    let (q, r) = f.div_rem(g);
    writeln!(writer, "{} {}", q.length(), r.length()).ok();
    iter_print!(writer, @it q.data);
    iter_print!(writer, @it r.data);
}
