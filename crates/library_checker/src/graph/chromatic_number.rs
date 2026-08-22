use competitive::{algorithm, prelude::*};

#[verify::library_checker("chromatic_number")]
pub fn chromatic_number(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, uv: [(usize, usize); m]);
    iter_print!(writer, algorithm::chromatic_number(n, &uv));
}
