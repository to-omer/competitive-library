#[doc(no_inline)]
pub use competitive::data_structure::LineSet;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { a: i64, b: i64 }
        1 => Get { q: i64 }
    }
}

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
        scan!(scanner, query: Query);
        match query {
            Query::Add { a, b } => {
                cht.insert(a, b);
            }
            Query::Get { q } => {
                writeln!(writer, "{}", cht.query_min(q).unwrap()).ok();
            }
        }
    }
}
