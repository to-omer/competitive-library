use competitive::data_structure::OfflineLiChaoTree;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { l: i32, r: i32, a: i32, b: i64 }
        1 => Get { x: i32 }
    }
}

#[verify::library_checker("segment_add_get_min")]
pub fn segment_add_get_min(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut tree = OfflineLiChaoTree::new();
    for (l, r, a, b) in scanner.iter::<(i32, i32, i32, i64)>().take(n) {
        tree.add_segment(l..r, (a, b));
    }
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { l, r, a, b } => {
                tree.add_segment(l..r, (a, b));
            }
            Query::Get { x } => {
                tree.query_min(x);
            }
        }
    }
    for result in tree.execute() {
        if let Some(value) = result {
            writeln!(writer, "{value}").ok();
        } else {
            writeln!(writer, "INFINITY").ok();
        }
    }
}
