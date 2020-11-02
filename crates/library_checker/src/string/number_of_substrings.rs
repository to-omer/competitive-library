use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::string::SuffixArray;

#[verify::verify("https://judge.yosupo.jp/problem/number_of_substrings")]
pub fn number_of_substrings(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let mut ans = s.len() * (s.len() + 1) / 2;
    let sa = SuffixArray::new(s);
    let lcp = sa.longest_common_prefix_array();
    for x in lcp {
        ans -= x;
    }
    writeln!(writer, "{}", ans).ok();
}