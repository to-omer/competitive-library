use crate::prelude::*;
pub use crate::string::SuffixArray;

#[verify_attr::verify("https://judge.yosupo.jp/problem/suffixarray")]
pub fn suffixarray(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let sa = SuffixArray::new(s);
    echo!(writer, (1..sa.len()).map(|i| sa[i]), " ");
}
