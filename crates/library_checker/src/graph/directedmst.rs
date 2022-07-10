use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, graph::EdgeListGraphScanner};

#[verify::library_checker("directedmst")]
pub fn directedmst(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, s, (graph, w): @EdgeListGraphScanner::<usize, i64>::new(n, m));
    let res = graph
        .minimum_spanning_arborescence::<AdditiveOperation<_>, _>(s, |u| w[u])
        .unwrap();
    iter_print!(writer, res.0; @it res.1);
}
