pub use crate::scan;
pub use crate::string::suffix_array::SuffixArray;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/suffixarray")]
pub fn suffixarray(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: chars);
    let sa = SuffixArray::new(s);
    for i in 1..sa.len() {
        write!(writer, "{}{}", if i == 1 { "" } else { " " }, sa[i])?;
    }
    writeln!(writer, "")?;
    Ok(())
}
