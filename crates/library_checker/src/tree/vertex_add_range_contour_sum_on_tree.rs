use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::TreeGraphScanner,
    tree::ContourQueryRange,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { p: usize, x: i64 }
        1 => Sum { v: usize, l: usize, r: usize }
    }
}

#[verify::library_checker("vertex_add_range_contour_sum_on_tree")]
pub fn vertex_add_range_contour_sum_on_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, mut a: [i64; n], (graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let cq = graph.contour_query_range();
    let mut raw = vec![0; cq.len()];
    for (v, &x) in a.iter().enumerate() {
        cq.for_each_index(v, |i| raw[i] += x);
    }
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::from_slice(&raw);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { p, x } => {
                a[p] += x;
                cq.for_each_index(p, |i| bit.update(i, x));
            }
            Query::Sum { v, l, r } => {
                let mut ans = if l == 0 && 0 < r { a[v] } else { 0 };
                cq.for_each_contour_range(v, l, r, |start, end| {
                    ans += bit.fold(start, end);
                });
                writeln!(writer, "{ans}").ok();
            }
        }
    }
}
