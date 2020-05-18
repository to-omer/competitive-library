pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/aplusb")]
pub fn aplusb(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let a: usize = scanner.scan();
    let b: usize = scanner.scan();
    writeln!(writer, "{}", a + b)
}
