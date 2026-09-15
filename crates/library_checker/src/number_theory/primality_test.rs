use competitive::math::miller_rabin;
use competitive::prelude::*;

#[verify::library_checker("primality_test")]
pub fn primality_test(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(q);
    for _ in 0..q {
        sc!(n: u64);
        let ans = if miller_rabin(n) { "Yes" } else { "No" };
        pp!(ans);
    }
}
