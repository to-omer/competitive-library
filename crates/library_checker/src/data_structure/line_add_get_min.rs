use competitive::data_structure::OfflineLiChaoTree;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { a: i32, b: i64 }
        1 => Get { x: i32 }
    }
}

#[verify::library_checker("line_add_get_min")]
pub fn line_add_get_min(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut tree = OfflineLiChaoTree::new();
    for (a, b) in sv!([(i32, i64); iter n]) {
        tree.add_line((a, b));
    }
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { a, b } => {
                tree.add_line((a, b));
            }
            Query::Get { x } => {
                tree.query_min(x);
            }
        }
    }
    pp!(@lf @it tree.execute().into_iter().map(Option::unwrap));
}
