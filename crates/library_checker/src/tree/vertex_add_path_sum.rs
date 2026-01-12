use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::TreeGraphScanner,
    tree::HeavyLightDecomposition,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { p: usize, x: i64 }
        1 => Sum { u: usize, v: usize }
    }
}

#[verify::library_checker("vertex_add_path_sum")]
pub fn vertex_add_path_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n], (mut graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(n);
    for (i, a) in a.iter().cloned().enumerate() {
        bit.update(hld.vidx[i], a);
    }
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { p, x } => {
                bit.update(hld.vidx[p], x);
            }
            Query::Sum { u, v } => {
                writeln!(
                    writer,
                    "{}",
                    hld.query::<AdditiveOperation<_>, _>(u, v, false, |l, r| bit.fold(l, r))
                )
                .ok();
            }
        }
    }
}
