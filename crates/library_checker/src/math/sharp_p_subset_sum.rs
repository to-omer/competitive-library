use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    math::{Fps998244353, MemorizedFactorial},
    num::{One, Zero, montgomery::MInt998244353},
};

#[verify::library_checker("sharp_p_subset_sum")]
pub fn sharp_p_subset_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, t, s: [usize; n]);
    let f = MemorizedFactorial::new(t);
    let mut c = vec![MInt998244353::zero(); t + 1];
    for s in s {
        c[s] += MInt998244353::one();
    }
    let a = Fps998244353::from_vec(c).count_subset_sum(t + 1, |x| f.inv(x));
    iter_print!(writer, @it a.data[1..]);
}
