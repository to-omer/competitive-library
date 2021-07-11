use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    math::Fps998244353,
    num::{mint_basic::MInt998244353, One, Zero},
};

#[verify::verify("https://judge.yosupo.jp/problem/sharp_p_subset_sum")]
pub fn sharp_p_subset_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, t, s: [usize; n]);
    let mut inv = vec![MInt998244353::zero(); t + 1];
    for (i, inv) in inv.iter_mut().enumerate().skip(1) {
        *inv = MInt998244353::from(i).inv();
    }
    let mut c = vec![MInt998244353::zero(); t + 1];
    for s in s {
        c[s] += MInt998244353::one();
    }
    let a = Fps998244353::from_vec(c).count_subset_sum(t + 1, &inv);
    iter_print!(writer, @iter a.data[1..]);
}
