use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/many_aplusb")]
pub fn many_aplusb(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t, ab: [(usize, usize); t]);
    for (a, b) in ab.into_iter() {
        writeln!(writer, "{}", a + b).ok();
    }
}
