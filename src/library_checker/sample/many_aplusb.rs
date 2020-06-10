pub use crate::scan;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/many_aplusb")]
pub fn many_aplusb(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t, ab: [(usize, usize); t]);
    for (a, b) in ab.into_iter() {
        writeln!(writer, "{}", a + b)?;
    }
    Ok(())
}
