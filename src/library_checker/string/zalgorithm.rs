pub use crate::string::Zarray;
use crate::tools::{read_all, Scanner};
use crate::{echo, scan};
use std::io::{Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/zalgorithm")]
pub fn zalgorithm(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: chars);
    let z = Zarray::new(&s);
    echo!(writer, (0..s.len()).map(|i| z[i]), " ");
}
