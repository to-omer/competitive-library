use competitive::data_structure::OfflineLiChaoTree;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { l: i32, r: i32, a: i32, b: i64 }
        1 => Get { x: i32 }
    }
}

#[verify::library_checker("segment_add_get_min")]
pub fn segment_add_get_min(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q);
    let mut tree = OfflineLiChaoTree::new();
    for (l, r, a, b) in sv!([(i32, i32, i32, i64); iter n]) {
        tree.add_segment(l..r, (a, b));
    }
    for _ in 0..q {
        sc!(query: Query);
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
            pp!(value);
        } else {
            pp!("INFINITY");
        }
    }
}
