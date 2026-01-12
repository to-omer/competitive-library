#[doc(no_inline)]
use competitive::algorithm::number_of_increasing_sequences_between_998244353;
use competitive::prelude::*;

#[verify::library_checker("number_of_increasing_sequences_between_two_sequences")]
pub fn number_of_increasing_sequences_between_two_sequences(
    reader: impl Read,
    mut writer: impl Write,
) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, _m, a: [usize; n], b: [usize; n]);
    let ans = number_of_increasing_sequences_between_998244353(&a, &b);
    writeln!(writer, "{}", ans).ok();
}
