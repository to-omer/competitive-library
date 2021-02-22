use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::string::Zarray;

#[verify::verify("https://judge.yosupo.jp/problem/zalgorithm")]
pub fn zalgorithm(reader: impl Read, writer: impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let z = Zarray::new(&s);
    echo(writer, (0..s.len()).map(|i| z[i]), ' ').ok();
}
