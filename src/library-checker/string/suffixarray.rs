// verify-helper: PROBLEM https://judge.yosupo.jp/problem/suffixarray

use competitive_library::input;
use competitive_library::string::suffix_array::SuffixArray;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, s: chars };
    let sa = SuffixArray::new(s);
    for i in 1..sa.len() {
        write!(out, "{}{}", if i == 1 { "" } else { " " }, sa[i])?;
    }
    writeln!(out, "")?;

    Ok(())
}
