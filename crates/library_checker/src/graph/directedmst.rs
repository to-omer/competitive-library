use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, graph::EdgeListGraphScanner};

#[verify::library_checker("directedmst")]
pub fn directedmst(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, s, (graph, w): @EdgeListGraphScanner::<usize, i64>::new(n, m));
    let res = graph
        .minimum_spanning_arborescence::<AdditiveOperation<_>, _>(s, |u| w[u])
        .unwrap();
    pp!(res.0; @it res.1);
}
