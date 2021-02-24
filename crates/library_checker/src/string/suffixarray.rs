use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::string::SuffixArray;

#[verify::verify("https://judge.yosupo.jp/problem/suffixarray")]
pub fn suffixarray(reader: impl Read, writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let n = s.len();
    let sa = SuffixArray::new(s);
    echo(writer, (1..=n).map(|i| sa[i]), ' ').ok();
}
