use crate::scan;
pub use crate::string::SuffixArray;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/suffixarray")]
pub fn suffixarray(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: chars);
    let sa = SuffixArray::new(s);
    for i in 1..sa.len() {
        write!(writer, "{}{}", if i == 1 { "" } else { " " }, sa[i]).ok();
    }
    writeln!(writer).ok();
}
