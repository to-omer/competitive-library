pub use crate::algebra::MaxOperation;
pub use crate::graph::{AdjacencyGraphAbstraction, TreeGraphScanner};
use crate::prelude::*;
pub use crate::tree::ReRooting;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_B")]
pub fn grl_5_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, (graph, _, w): { TreeGraphScanner::<usize, u64>::new(n) });
    let re = ReRooting::new(&graph, MaxOperation::new(), |d, _vid, eid_opt| {
        d + eid_opt.map_or(0, |eid| w[eid])
    });
    echo(writer, re.dp, '\n').ok();
}
