use competitive::prelude::*;

#[verify::library_checker("many_aplusb")]
pub fn many_aplusb(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t);
    for (a, b) in scanner.iter::<(usize, usize)>().take(t) {
        writeln!(writer, "{}", a + b).ok();
    }
}
