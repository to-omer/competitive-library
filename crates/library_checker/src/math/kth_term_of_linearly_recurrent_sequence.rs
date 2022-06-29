use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    math::Fps998244353,
    num::{montgomery::MInt998244353, One},
};

#[verify::verify("https://judge.yosupo.jp/problem/kth_term_of_linearly_recurrent_sequence")]
pub fn kth_term_of_linearly_recurrent_sequence(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, d, k, a: [MInt998244353; d], c: [MInt998244353; d]);
    let q = Fps998244353::one() - (Fps998244353::from_vec(c) << 1);
    iter_print!(writer, q.kth_term_of_linearly_recurrence(a, k));
}
