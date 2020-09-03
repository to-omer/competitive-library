use crate::prelude::*;
pub use crate::string::Zarray;

#[verify_attr::verify("https://judge.yosupo.jp/problem/zalgorithm")]
pub fn zalgorithm(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let z = Zarray::new(&s);
    echo(writer, (0..s.len()).map(|i| z[i]), ' ').ok();
}
