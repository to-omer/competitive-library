#[doc(no_inline)]
pub use competitive::num::mint_basic::DynMIntU32;
use competitive::{math::MemorizedFactorial, prelude::*};

#[verify::library_checker("binomial_coefficient_prime_mod")]
pub fn binomial_coefficient_prime_mod(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t, m: u32, nk: [(usize, usize); t]);
    DynMIntU32::set_mod(m);
    let max_n = nk.iter().map(|(n, _)| n).max().cloned().unwrap_or_default();
    let f = MemorizedFactorial::new(max_n);
    for (n, k) in nk {
        let ans: DynMIntU32 = f.combination(n, k);
        iter_print!(writer, ans);
    }
}
