use competitive::prelude::*;
use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::library_checker("find_linear_recurrence")]
pub fn find_linear_recurrence(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [MInt998244353; n]);
    let c = Fps998244353::berlekamp_massey(&a);
    iter_print!(writer, c.length() - 1; @it c.iter().skip(1).map(|x| -x));
}
