use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::TreeGraphScanner,
    tree::ContourQueryRange,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { v: usize, l: usize, r: usize, x: i64 }
        1 => Get { v: usize }
    }
}

#[verify::library_checker("vertex_get_range_contour_add_on_tree")]
pub fn vertex_get_range_contour_add_on_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, mut a: [i64; n], (graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let cq = graph.contour_query_range();
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(cq.len() + 1);

    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { v, l, r, x } => {
                cq.for_each_contour_range(v, l, r, |start, end| {
                    bit.update(start, x);
                    bit.update(end, -x);
                });
                if l == 0 && 0 < r {
                    a[v] += x;
                }
            }
            Query::Get { v } => {
                let mut ans = a[v];
                cq.for_each_index(v, |i| ans += bit.accumulate(i));
                writeln!(writer, "{ans}").ok();
            }
        }
    }
}
