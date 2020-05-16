pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/many_aplusb")]
pub fn many_aplusb(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let t: usize = scanner.scan();
    for _ in 0..t {
        let a: usize = scanner.scan();
        let b: usize = scanner.scan();
        writeln!(writer, "{}", a + b)?;
    }
    Ok(())
}
