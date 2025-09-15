#[doc(no_inline)]
pub use competitive::data_structure::LineSet;
use competitive::prelude::*;

#[verify::library_checker("line_add_get_min")]
pub fn line_add_get_min(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut cht = LineSet::new();
    for (a, b) in scanner.iter::<(i64, i64)>().take(n) {
        cht.insert(a, b);
    }
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, a: i64, b: i64);
            cht.insert(a, b);
        } else {
            scan!(scanner, q: i64);
            writeln!(writer, "{}", cht.query_min(q).unwrap()).ok();
        }
    }
}
