use competitive::prelude::*;
pub use competitive::string::SuffixArray;

#[verify::verify("https://judge.yosupo.jp/problem/suffixarray")]
pub fn suffixarray(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let n = s.len();
    let sa = SuffixArray::new(s);
    echo(writer, (1..=n).map(|i| sa[i]), ' ').ok();
}
