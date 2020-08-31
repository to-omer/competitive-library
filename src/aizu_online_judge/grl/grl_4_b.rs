pub use crate::graph::GraphScanner;
use crate::prelude::*;

#[verify_attr::verify(
    "https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/4/GRL_4_B",
    judge = "judge_grl_4_b"
)]
pub fn grl_4_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): {GraphScanner::<usize, ()>::new(vs, es, true)});
    for u in graph.topological_sort().into_iter() {
        writeln!(writer, "{}", u).ok();
    }
}

pub fn judge_grl_4_b(
    input: &mut impl Read,
    _output: &mut impl Read,
    result: &mut impl Read,
) -> bool {
    let (s_in, s_res) = (read_all(input), read_all(result));
    let (mut scanner_in, mut scanner_res) = (Scanner::new(&s_in), Scanner::new(&s_res));
    scan!(scanner_in, vs, es, edges: [(usize, usize); es]);
    let mut ord = vec![!0usize; vs];
    let mut is_ac = true;
    for (i, u) in scanner_res.iter::<usize>().take(vs).enumerate() {
        is_ac &= ord[u] == !0usize;
        ord[u] = i;
    }
    for (u, v) in edges.into_iter() {
        is_ac &= ord[u] < ord[v];
    }
    is_ac
}
