use competitive::prelude::*;
use competitive::{algorithm::SliceSortExt, graph::EdgeListGraphScanner};

#[verify::library_checker("minimum_spanning_tree")]
pub fn minimum_spanning_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, (graph, w): @EdgeListGraphScanner::<usize, u32>::new(n, m));
    let mut edges: Vec<_> = (0..m).map(|eid| (w[eid], eid as u32)).collect();
    edges.radix_sort_by_key(|&(weight, _)| weight);
    let span = graph
        .minimum_spanning_tree_from_sorted_edges(edges.into_iter().map(|(_, eid)| eid as usize));
    let ans = (0..m).filter(|&eid| span[eid]);
    let total: u64 = ans.clone().map(|eid| u64::from(w[eid])).sum();
    pp!(total; @it ans);
}
