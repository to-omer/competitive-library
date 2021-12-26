use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/aplusb")]
pub fn aplusb(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, a, b);
    writeln!(writer, "{}", a + b).ok();
}
